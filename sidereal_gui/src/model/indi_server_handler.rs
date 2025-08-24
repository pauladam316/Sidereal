use crate::{
    app::{ConnectedDevices, Message},
    gui::{tabs::mount::Message as MountMessage, widgets::server_status::ServerStatus},
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
    time::{self, interval, Instant},
};
// type IndiClientInner = indi::client::Client;

pub struct IndiClientInstance {
    ip: String,
    client: indi::client::Client,
}
type SharedIndiClient = Arc<RwLock<Option<Arc<IndiClientInstance>>>>;

pub static INDI_CLIENT: Lazy<SharedIndiClient> = Lazy::new(|| Arc::new(RwLock::new(None)));

pub(crate) async fn connect_to_server(ip_addr: String) -> SiderealResult<()> {
    println!("Connecting");
    let stream = TcpStream::connect(ip_addr.clone())
        .await
        .map_err(|e| SiderealError::ServerError(format!("{:?}", e)))?;

    let client = indi::client::new(stream, None, None)
        .map_err(|e| SiderealError::ServerError(format!("{:?}", e)))?;

    {
        let mut guard = INDI_CLIENT.write().await;
        *guard = Some(Arc::new(IndiClientInstance {
            ip: ip_addr.to_owned(),
            client,
        }));
    }

    println!("Connected");
    Ok(())
}

pub struct ServerInstance {
    pub mount: Option<ActiveDevice>,
    pub camera: Option<ActiveDevice>,
    pub focuser: Option<ActiveDevice>,
}

type SharedConnected = Arc<RwLock<ServerInstance>>;
pub static CONNECTED_DEVICES: Lazy<SharedConnected> =
    Lazy::new(|| Arc::new(RwLock::new(ServerInstance::default())));

impl Default for ServerInstance {
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

pub async fn move_mount(direction: String, subdirection: String) -> SiderealResult<()> {
    let devices = CONNECTED_DEVICES.read().await;
    match &devices.mount {
        Some(mount) => match mount
            .change(direction.as_str(), vec![(subdirection.as_str(), true)])
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(SiderealError::ServerError(format!("{:?}", e))),
        },
        None => Err(SiderealError::ServerError(
            "No mount available to command".to_owned(),
        )),
    }
}

pub async fn stop_move() {
    let devices = CONNECTED_DEVICES.read().await;
    if let Some(mount) = &devices.mount {
        if let Err(e) = mount
            .change("TELESCOPE_MOTION_NS", vec![("MOTION_NORTH", false)])
            .await
        {
            println!("{:?}", e);
        }
        if let Err(e) = mount
            .change("TELESCOPE_MOTION_NS", vec![("MOTION_SOUTH", false)])
            .await
        {
            println!("{:?}", e);
        }
        if let Err(e) = mount
            .change("TELESCOPE_MOTION_WE", vec![("MOTION_WEST", false)])
            .await
        {
            println!("{:?}", e);
        }
        if let Err(e) = mount
            .change("TELESCOPE_MOTION_WE", vec![("MOTION_EAST", false)])
            .await
        {
            println!("{:?}", e);
        }
    }
}

//TODO: run this every time the device list changes
pub async fn find_connected_devices<S>(
    mut out: S, // e.g., the sender from `stream::channel`
) -> SiderealResult<()>
where
    S: Sink<Message> + Unpin,
{
    let start = Instant::now();

    // Wait until INDI_CLIENT is set
    let client_instance: Arc<IndiClientInstance> = loop {
        if let Some(c) = INDI_CLIENT.read().await.as_ref().cloned() {
            break c;
        }
        time::sleep(Duration::from_millis(100)).await;
    };

    loop {
        // ---- 1) Scan under locks: collect *names* only ----
        let (mount_name, camera_name, focuser_name) = {
            let devices = client_instance.client.get_devices();
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
        let mut result = ServerInstance::default();

        if let Some(n) = mount_name.clone() {
            let dev = client_instance
                .client
                .get_device::<()>(&n)
                .await
                .map_err(|e| SiderealError::ServerError(format!("get_device({n}): {:?}", e)))?;
            result.mount = Some(dev);
        }
        if let Some(n) = camera_name.clone() {
            let dev = client_instance
                .client
                .get_device::<()>(&n)
                .await
                .map_err(|e| SiderealError::ServerError(format!("get_device({n}): {:?}", e)))?;
            result.camera = Some(dev);
        }
        if let Some(n) = focuser_name.clone() {
            let dev = client_instance
                .client
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
    client_instance: &Arc<IndiClientInstance>,
) -> Result<indi::client::active_device::ActiveDevice, String> {
    use std::time::Instant;

    let start = Instant::now();
    loop {
        // 1) Take a snapshot of the candidate mount name while holding locks
        let mount_name = {
            let devices = client_instance.client.get_devices(); // Arc<Mutex<HashMap<..>>>
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
            return client_instance
                .client
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

async fn tcp_probe(addr: &str) -> bool {
    match time::timeout(Duration::from_secs(2), TcpStream::connect(addr)).await {
        Ok(Ok(_)) => true, // could connect
        _ => false,        // timed out or refused
    }
}

const RECONNECT_DELAY_MS: u64 = 1000; // <-- constant backoff

async fn handle_loss_and_break<S>(output: &mut S, reason: &str)
where
    S: Sink<Message> + Unpin,
{
    let _ = output
        .send(Message::ServerStatus(ServerStatus::ConnectionLost))
        .await;
    println!("{}", format!("connection lost: {reason}"));

    // Clear current client so the outer loop won't reuse a dead handle
    {
        let mut guard = INDI_CLIENT.write().await;
        *guard = None;
    }
}

pub fn param_watcher() -> impl Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        'reconnect: loop {
            // 1) Wait until we have a client; if none, sleep until someone connects initially.
            //    (First connection is expected to be initiated elsewhere via connect_to_server).
            let client_instance: Arc<IndiClientInstance> = loop {
                if let Some(c) = INDI_CLIENT.read().await.as_ref().cloned() {
                    break c;
                }
                time::sleep(Duration::from_millis(100)).await;
            };

            // Keep the server address handy for reconnects
            let server_addr = client_instance.ip.clone();

            // 2) Make sure devices are there
            if let Err(e) = find_connected_devices(&mut output).await {
                let _ = output.send(Message::IndiError(format!("{e:?}"))).await;
                time::sleep(Duration::from_millis(RECONNECT_DELAY_MS)).await;
                continue 'reconnect;
            }

            // 3) Find the mount
            let mount = match find_mount(&client_instance).await {
                Ok(m) => m,
                Err(e) => {
                    let _ = output.send(Message::IndiError(e)).await;
                    time::sleep(Duration::from_millis(RECONNECT_DELAY_MS)).await;
                    continue 'reconnect;
                }
            };

            if let Err(e) = mount.change("CONNECTION", vec![("CONNECT", true)]).await {
                let _ = output
                    .send(Message::IndiError(format!("CONNECT failed: {e:?}")))
                    .await;
                time::sleep(Duration::from_millis(RECONNECT_DELAY_MS)).await;
                continue 'reconnect;
            }

            // 4) Parameter we care about
            let param_notify = match mount.get_parameter("EQUATORIAL_EOD_COORD").await {
                Ok(p) => p,
                Err(_) => match mount.get_parameter("EQUATORIAL_COORD").await {
                    Ok(p) => p,
                    Err(e) => {
                        let _ = output
                            .send(Message::IndiError(format!("get_parameter: {:?}", e)))
                            .await;
                        time::sleep(Duration::from_millis(RECONNECT_DELAY_MS)).await;
                        continue 'reconnect;
                    }
                },
            };

            // 5) Subscribe & start heartbeat
            let mut changes = param_notify.subscribe().await;
            let mut hb = interval(Duration::from_secs(5));
            hb.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

            // 6) Event loop
            let lost = loop {
                tokio::select! {
                    maybe_next = changes.next() => {
                        match maybe_next {
                            Some(Ok(param_arc)) => {
                                if let Ok(map) = param_arc.get_values::<HashMap<String, indi::Number>>() {
                                    if let (Some(ra), Some(dec)) = (map.get("RA"), map.get("DEC")) {
                                        let _ = output
                                            .send(Message::Mount(MountMessage::CoordsUpdated {
                                                ra_hours: ra.value.into(),
                                                dec_deg: dec.value.into(),
                                            }))
                                            .await;
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                handle_loss_and_break(&mut output, &format!("stream error: {e}")).await;
                                break true;
                            }
                            None => {
                                handle_loss_and_break(&mut output, "stream ended").await;
                                break true;
                            }
                        }
                    }

                    _ = hb.tick() => {
                        if !tcp_probe(&server_addr).await {
                            handle_loss_and_break(&mut output, "tcp probe failed").await;
                            break true;
                        }
                    }
                }
            };

            // 7) Reconnect loop (constant backoff) — keep going until we succeed
            if lost {
                loop {
                    match connect_to_server(server_addr.clone()).await {
                        Ok(()) => {
                            let _ = output
                                .send(Message::ServerStatus(ServerStatus::Connected))
                                .await;
                            break;
                        } // success; proceed to 'reconnect which will resubscribe
                        Err(e) => {
                            println!("Reconnect failed: {}, retrying", e);
                            time::sleep(Duration::from_millis(RECONNECT_DELAY_MS)).await;
                        }
                    }
                }
            }

            // Loop back to resubscribe with the fresh client
            continue 'reconnect;
        }
    })
}
