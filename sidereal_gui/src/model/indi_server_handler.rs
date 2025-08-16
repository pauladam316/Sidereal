use crate::{
    app::{ConnectedDevices, Message},
    gui::tabs::mount::Message as MountMessage,
    model::{SiderealError, SiderealResult},
};
use iced::{
    futures::{Sink, SinkExt, Stream, StreamExt},
    stream,
};
use indi::client::active_device::ActiveDevice;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    net::TcpStream,
    sync::RwLock,
    time::{self, Instant},
};
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

pub struct ConnectedDevicesOne {
    pub mount: Option<ActiveDevice>,
    pub camera: Option<ActiveDevice>,
    pub focuser: Option<ActiveDevice>,
}

type SharedConnected = Arc<RwLock<ConnectedDevicesOne>>;
pub static CONNECTED_DEVICES: Lazy<SharedConnected> =
    Lazy::new(|| Arc::new(RwLock::new(ConnectedDevicesOne::default())));

impl Default for ConnectedDevicesOne {
    fn default() -> Self {
        Self {
            mount: None,
            camera: None,
            focuser: None,
        }
    }
}

// INDI interface bitmasks (common values)
const IF_TELESCOPE: u32 = 0x0001; // mount
const IF_CCD: u32 = 0x0002; // camera
const IF_FOCUSER: u32 = 0x0008; // focuser

//TODO: run this every time the device list changes
pub async fn find_connected_devices<S>(
    mut out: S, // e.g., the sender from `stream::channel`
) -> SiderealResult<()>
where
    S: Sink<Message> + Unpin,
{
    let start = Instant::now();

    // Wait until INDI_CLIENT is set
    let client: Arc<IndiClientInner> = loop {
        if let Some(c) = INDI_CLIENT.read().await.as_ref().cloned() {
            break c;
        }
        time::sleep(Duration::from_millis(100)).await;
    };

    loop {
        // ---- 1) Scan under locks: collect *names* only ----
        let (mount_name, camera_name, focuser_name) = {
            let devices = client.get_devices();
            let map = devices.lock().await;

            let mut mount_name: Option<String> = None;
            let mut camera_name: Option<String> = None;
            let mut focuser_name: Option<String> = None;

            for (name, dev_mx) in map.iter() {
                let dev = dev_mx.lock().await;
                let params = dev.get_parameters();

                // DRIVER_INFO.DRIVER_INTERFACE -> capability mask
                let mut iface_mask = 0u32;
                if let Some(driver_info_mx) = params.get("DRIVER_INFO") {
                    let driver_info = driver_info_mx.lock().await;
                    if let Ok(tv) = driver_info.get_values::<HashMap<String, indi::Text>>() {
                        if let Some(iface) = tv.get("DRIVER_INTERFACE") {
                            if let Ok(mask) = iface.value.parse::<u32>() {
                                iface_mask = mask;
                            }
                        }
                    }
                }

                // CONNECTION.CONNECT (if available)
                let mut _is_connected = true;
                if let Some(conn_mx) = params.get("CONNECTION") {
                    let conn = conn_mx.lock().await;
                    if let Ok(sw) = conn.get_values::<HashMap<String, indi::Switch>>() {
                        if let Some(connect_sw) = sw.get("CONNECT") {
                            _is_connected = connect_sw.value == indi::SwitchState::On;
                        }
                    }
                }
                //todo: do something if we can't connect
                // if !is_connected {
                //     println!("couldn't connect");
                //     continue;
                // }

                if mount_name.is_none() && (iface_mask & IF_TELESCOPE) != 0 {
                    mount_name = Some(name.clone());
                } else if camera_name.is_none() && (iface_mask & IF_CCD) != 0 {
                    camera_name = Some(name.clone());
                } else if focuser_name.is_none() && (iface_mask & IF_FOCUSER) != 0 {
                    focuser_name = Some(name.clone());
                }

                if mount_name.is_some() && camera_name.is_some() && focuser_name.is_some() {
                    break;
                }
            }

            (mount_name, camera_name, focuser_name)
            // all guards dropped here
        };

        // ---- 2) Resolve names to ActiveDevice (no locks held) ----
        let mut result = ConnectedDevicesOne::default();

        if let Some(n) = mount_name.clone() {
            let dev = client
                .get_device::<()>(&n)
                .await
                .map_err(|e| SiderealError::ServerError(format!("get_device({n}): {:?}", e)))?;
            result.mount = Some(dev);
        }
        if let Some(n) = camera_name.clone() {
            let dev = client
                .get_device::<()>(&n)
                .await
                .map_err(|e| SiderealError::ServerError(format!("get_device({n}): {:?}", e)))?;
            result.camera = Some(dev);
        }
        if let Some(n) = focuser_name.clone() {
            let dev = client
                .get_device::<()>(&n)
                .await
                .map_err(|e| SiderealError::ServerError(format!("get_device({n}): {:?}", e)))?;
            result.focuser = Some(dev);
        }

        // ---- 3) If we found anything, write to the shared cache and return ----
        if result.mount.is_some() || result.camera.is_some() || result.focuser.is_some() {
            // update global cache if you have one
            *CONNECTED_DEVICES.write().await = result;
            // send a one-shot MountMessage with just the names
            out.send(Message::ConnectedDeviceChange(ConnectedDevices {
                mount: mount_name,
                camera: camera_name,
                focuser: focuser_name,
            }))
            .await
            .ok(); // ignore send errors

            return Ok(());
        }

        if start.elapsed() > Duration::from_secs(10) {
            return Err(SiderealError::ServerError(
                "No mount, camera, or focuser found within 10s".into(),
            ));
        }

        time::sleep(Duration::from_millis(100)).await;
    }
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

pub fn param_watcher() -> impl Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        // 1) Get a clone to the client (wait until connected)
        let client: Arc<IndiClientInner> = loop {
            if let Some(c) = INDI_CLIENT.read().await.as_ref().cloned() {
                break c;
            }
            time::sleep(Duration::from_millis(100)).await;
        };

        if let Err(e) = find_connected_devices(&mut output).await {
            let _ = output.send(Message::IndiError(format!("{e:?}"))).await;
            return;
        }

        // 2) Find the mount
        let mount = match find_mount(&client).await {
            Ok(m) => m,
            Err(e) => {
                println!("Couldn't find mount");
                let _ = output.send(Message::IndiError(e)).await;
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
                        .send(Message::IndiError(format!("get_parameter: {:?}", e)))
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
                                .send(Message::Mount(MountMessage::CoordsUpdated {
                                    ra_hours: ra.value.into(), // hours
                                    dec_deg: dec.value.into(), // degrees
                                }))
                                .await;
                        }
                    }
                }
                Err(e) => {
                    let _ = output
                        .send(Message::IndiError(format!("stream error: {e}")))
                        .await;
                }
            }
        }
    })
}
