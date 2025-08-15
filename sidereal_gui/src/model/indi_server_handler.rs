use crate::{
    gui::tabs::mount::Message as MountMessage,
    model::{SiderealError, SiderealResult},
};
use iced::{
    futures::{SinkExt, Stream, StreamExt},
    stream,
};
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{net::TcpStream, sync::RwLock, time};

type IndiClientInner = indi::client::Client;
type SharedIndiClient = Arc<RwLock<Option<Arc<IndiClientInner>>>>;

pub static INDI_CLIENT: Lazy<SharedIndiClient> = Lazy::new(|| Arc::new(RwLock::new(None)));

pub(crate) async fn connect_to_server(ip_addr: &str) -> SiderealResult<()> {
    println!("Connecting");
    let stream = TcpStream::connect(ip_addr)
        .await
        .map_err(|e| SiderealError::ServerError(format!("{:?}", e)))?;

    let client = indi::client::new(stream, None, None)
        .map_err(|e| SiderealError::ServerError(format!("{:?}", e)))?;

    {
        let mut guard = INDI_CLIENT.write().await;
        *guard = Some(Arc::new(client));
    }

    println!("Connected");
    Ok(())
}

async fn find_mount(
    client: &Arc<IndiClientInner>,
) -> Result<indi::client::active_device::ActiveDevice, String> {
    use std::time::Instant;

    let start = Instant::now();
    loop {
        // 1) Take a snapshot of the candidate mount name while holding locks
        let mount_name = {
            let devices = client.get_devices(); // Arc<Mutex<HashMap<..>>>
            let map = devices.lock().await;

            let mut found: Option<String> = None;

            // NOTE: no `.await` after this point until we drop `map` and any inner guards
            for (name, dev_mx) in map.iter() {
                let dev = dev_mx.lock().await;
                let params = dev.get_parameters();

                if let Some(driver_info_mx) = params.get("DRIVER_INFO") {
                    let driver_info = driver_info_mx.lock().await;

                    if let Ok(tv) = driver_info.get_values::<HashMap<String, indi::Text>>() {
                        if let Some(iface) = tv.get("DRIVER_INTERFACE") {
                            if let Ok(mask) = iface.value.parse::<u32>() {
                                if (mask & 1) != 0 {
                                    found = Some(name.clone());
                                    break;
                                }
                            }
                        }
                    }
                }
                // drop `dev` / `driver_info` guards at end of loop iteration
            }

            // `map` guard (and any inner guards) drop here
            found
        };

        // 2) Do awaited calls only *after* all the above guards are dropped
        if let Some(name) = mount_name {
            return client
                .get_device::<()>(&name)
                .await
                .map_err(|e| format!("get_device({name}): {:?}", e));
        }

        if start.elapsed() > Duration::from_secs(10) {
            return Err("No mount found within 10s".into());
        }
        time::sleep(Duration::from_millis(100)).await;
    }
}

pub fn param_watcher() -> impl Stream<Item = MountMessage> {
    stream::channel(100, |mut output| async move {
        // 1) Get a clone to the client (wait until connected)
        let client: Arc<IndiClientInner> = loop {
            if let Some(c) = INDI_CLIENT.read().await.as_ref().cloned() {
                break c;
            }
            time::sleep(Duration::from_millis(100)).await;
        };
        // 2) Find the mount
        let mount = match find_mount(&client).await {
            Ok(m) => m,
            Err(e) => {
                println!("Couldn't find mount");
                let _ = output.send(MountMessage::IndiError(e)).await;
                return;
            }
        };
        println!("Fouund Mount");
        if let Err(e) = mount.change("CONNECTION", vec![("CONNECT", true)]).await {
            println!("Failed to send CONNECT to mount: {:?}", e);
        } else {
            println!("CONNECT command sent to mount");
        }
        // 3) Get RA/DEC parameter notify (prefer EOD; fallback to J2000)
        let param_notify = match mount.get_parameter("EQUATORIAL_EOD_COORD").await {
            Ok(p) => p,
            Err(_) => match mount.get_parameter("EQUATORIAL_COORD").await {
                Ok(p) => p,
                Err(e) => {
                    println!("failed to find params");
                    let _ = output
                        .send(MountMessage::IndiError(format!("get_parameter: {:?}", e)))
                        .await;
                    return;
                }
            },
        };
        // 4) Subscribe: current snapshot + all future changes
        let mut changes = param_notify.subscribe().await; // BroadcastStream<Arc<Parameter>>

        while let Some(next) = changes.next().await {
            match next {
                Ok(param_arc) => {
                    if let Ok(map) = param_arc.get_values::<HashMap<String, indi::Number>>() {
                        if let (Some(ra), Some(dec)) = (map.get("RA"), map.get("DEC")) {
                            let _ = output
                                .send(MountMessage::CoordsUpdated {
                                    ra_hours: ra.value.into(), // hours
                                    dec_deg: dec.value.into(), // degrees
                                })
                                .await;
                        }
                    }
                }
                Err(e) => {
                    let _ = output
                        .send(MountMessage::IndiError(format!("stream error: {e}")))
                        .await;
                }
            }
        }
    })
}
