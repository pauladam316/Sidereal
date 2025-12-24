use crate::{
    app::{ConnectedDevices, Message},
    gui::widgets::server_status::ServerStatus,
    model::{SiderealError, SiderealResult},
};
use iced::{
    futures::{Sink, SinkExt, Stream},
    stream,
};
use indi::client::active_device::ActiveDevice;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{
    net::TcpStream,
    sync::RwLock,
    time::{self, interval},
};

pub mod camera;
pub mod focuser;
pub mod mount;
pub mod telescope_controller;

// INDI interface bitmasks (common values)
const IF_TELESCOPE: u32 = 0x0001; // mount
const IF_CCD: u32 = 0x0002; // camera
const IF_FOCUSER: u32 = 0x0008; // focuser

/// INDI client instance wrapper
pub struct IndiClientInstance {
    pub ip: String,
    pub client: indi::client::Client,
}

type SharedIndiClient = Arc<RwLock<Option<Arc<IndiClientInstance>>>>;

/// Global INDI client instance
pub static INDI_CLIENT: Lazy<SharedIndiClient> = Lazy::new(|| Arc::new(RwLock::new(None)));

/// Container for all connected devices
pub struct ServerInstance {
    pub mount: Option<ActiveDevice>,
    pub camera: Option<ActiveDevice>,
    pub focuser: Option<ActiveDevice>,
    pub telescope_controller: Option<ActiveDevice>,
}

impl Default for ServerInstance {
    fn default() -> Self {
        Self {
            mount: None,
            camera: None,
            focuser: None,
            telescope_controller: None,
        }
    }
}

type SharedConnected = Arc<RwLock<ServerInstance>>;

/// Global cache of connected devices
pub static CONNECTED_DEVICES: Lazy<SharedConnected> =
    Lazy::new(|| Arc::new(RwLock::new(ServerInstance::default())));

/// Track last telemetry time for each device type
/// Uses device name as key for easy extension
pub(crate) type TelemetryTimes = HashMap<String, Instant>;

type SharedTelemetryTimes = Arc<RwLock<TelemetryTimes>>;

/// Global telemetry time tracker
pub(crate) static TELEMETRY_TIMES: Lazy<SharedTelemetryTimes> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Connect to an INDI server
pub async fn connect_to_server(ip_addr: String) -> SiderealResult<()> {
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

    Ok(())
}

/// Discover and connect to all available devices (mount, camera, focuser, telescope controller)
/// This function always succeeds - it just returns what devices are currently available.
/// If no devices are found, it still updates the cache and sends an empty device list.
pub async fn find_connected_devices<S>(mut out: S) -> SiderealResult<()>
where
    S: Sink<Message> + Unpin,
{
    // Wait until INDI_CLIENT is set
    let client_instance: Arc<IndiClientInstance> = loop {
        if let Some(c) = INDI_CLIENT.read().await.as_ref().cloned() {
            break c;
        }
        time::sleep(Duration::from_millis(100)).await;
    };

    // ---- 1) Scan under locks: collect *names* only ----
    let (mount_name, camera_name, focuser_name, telescope_controller_name) = {
        let devices = client_instance.client.get_devices();
        let map = devices.lock().await;

        let mut mount_name: Option<String> = None;
        let mut camera_name: Option<String> = None;
        let mut focuser_name: Option<String> = None;
        let mut telescope_controller_name: Option<String> = None;

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

            // Check for Telescope Controller by device name (it's an AUX device)
            if telescope_controller_name.is_none() && name == "Telescope Controller" {
                telescope_controller_name = Some(name.clone());
            } else if mount_name.is_none() && (iface_mask & IF_TELESCOPE) != 0 {
                mount_name = Some(name.clone());
            } else if camera_name.is_none() && (iface_mask & IF_CCD) != 0 {
                camera_name = Some(name.clone());
            } else if focuser_name.is_none() && (iface_mask & IF_FOCUSER) != 0 {
                focuser_name = Some(name.clone());
            }
        }

        (
            mount_name,
            camera_name,
            focuser_name,
            telescope_controller_name,
        )
        // all guards dropped here
    };

    // ---- 2) Resolve names to ActiveDevice, connect, and verify they're reachable ----
    // Process: get_device -> try to connect -> verify we can get a parameter
    let mut result = ServerInstance::default();
    let mut final_mount_name: Option<String> = None;
    let mut final_camera_name: Option<String> = None;
    let mut final_focuser_name: Option<String> = None;
    let mut final_telescope_controller_name: Option<String> = None;

    // Helper to connect to device and verify it's reachable
    // Reduced timeouts for faster discovery
    async fn connect_and_verify_device(dev: &ActiveDevice) -> bool {
        // Step 1: Try to connect to the device with timeout
        match time::timeout(
            Duration::from_millis(300),
            dev.change("CONNECTION", vec![("CONNECT", true)]),
        )
        .await
        {
            Ok(Ok(_)) => {
                // Connection succeeded, continue to verification
            }
            _ => {
                // Connection failed or timed out - device not reachable
                return false;
            }
        }

        // Step 2: Verify we can actually communicate by getting a parameter
        // Use shorter timeout to avoid hanging on unresponsive devices
        match time::timeout(Duration::from_millis(300), dev.get_parameter("DRIVER_INFO")).await {
            Ok(Ok(_)) => true, // Successfully got parameter - device is reachable
            _ => {
                // DRIVER_INFO might not exist, try CONNECTION as fallback with shorter timeout
                match time::timeout(Duration::from_millis(200), dev.get_parameter("CONNECTION"))
                    .await
                {
                    Ok(Ok(_)) => true,
                    _ => false, // Can't reach device
                }
            }
        }
    }

    // Check all devices in parallel for faster discovery
    let (mount_result, camera_result, focuser_result, telescope_controller_result) = tokio::join!(
        async {
            if let Some(n) = mount_name.clone() {
                match time::timeout(
                    Duration::from_millis(300),
                    client_instance.client.get_device::<()>(&n),
                )
                .await
                {
                    Ok(Ok(dev)) => {
                        if connect_and_verify_device(&dev).await {
                            Some((dev, n))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        },
        async {
            if let Some(n) = camera_name.clone() {
                match time::timeout(
                    Duration::from_millis(300),
                    client_instance.client.get_device::<()>(&n),
                )
                .await
                {
                    Ok(Ok(dev)) => {
                        if connect_and_verify_device(&dev).await {
                            Some((dev, n))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        },
        async {
            if let Some(n) = focuser_name.clone() {
                match time::timeout(
                    Duration::from_millis(300),
                    client_instance.client.get_device::<()>(&n),
                )
                .await
                {
                    Ok(Ok(dev)) => {
                        if connect_and_verify_device(&dev).await {
                            Some((dev, n))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        },
        async {
            if let Some(n) = telescope_controller_name.clone() {
                match time::timeout(
                    Duration::from_millis(300),
                    client_instance.client.get_device::<()>(&n),
                )
                .await
                {
                    Ok(Ok(dev)) => {
                        if connect_and_verify_device(&dev).await {
                            Some((dev, n))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            } else {
                None
            }
        }
    );

    // Process results
    if let Some((dev, name)) = mount_result {
        result.mount = Some(dev);
        final_mount_name = Some(name);
    }
    if let Some((dev, name)) = camera_result {
        result.camera = Some(dev);
        final_camera_name = Some(name);
    }
    if let Some((dev, name)) = focuser_result {
        result.focuser = Some(dev);
        final_focuser_name = Some(name);
    }
    if let Some((dev, name)) = telescope_controller_result {
        result.telescope_controller = Some(dev);
        final_telescope_controller_name = Some(name);
    }

    // ---- 3) Always update the cache and send device change message ----
    // Only send device names for devices that we successfully resolved and verified
    // This ensures the UI only shows devices that are actually available and responding
    // Devices that failed verification will have None values, clearing them from the UI
    //
    // Important: We always send the message, even if all devices are None, to ensure
    // the UI updates when devices become unreachable
    *CONNECTED_DEVICES.write().await = result;

    // Always send the update - this ensures devices drop off when they become unreachable
    let _ = out
        .send(Message::ConnectedDeviceChange(ConnectedDevices {
            mount: final_mount_name,
            camera: final_camera_name,
            focuser: final_focuser_name,
            telescope_controller: final_telescope_controller_name,
        }))
        .await;

    Ok(())
}

/// TCP connection probe for heartbeat checking
pub(crate) async fn tcp_probe(addr: &str) -> bool {
    match time::timeout(Duration::from_secs(2), TcpStream::connect(addr)).await {
        Ok(Ok(_)) => true, // could connect
        _ => false,        // timed out or refused
    }
}

/// Stream that periodically discovers devices and sends updates when devices appear/disappear
/// This allows the UI to stay in sync with device availability
/// Runs every 1 second in the background
pub fn device_discovery_watcher() -> impl Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        let mut discovery_interval = interval(Duration::from_secs(1)); // Check every 1 second
        discovery_interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            // Wait for next interval tick
            discovery_interval.tick().await;

            // Check if we have a client
            if INDI_CLIENT.read().await.is_some() {
                // Discover devices and send update (always succeeds)
                // With parallel checks, this should complete in <2 seconds even with timeouts
                // Increased timeout to 3 seconds to be safe
                let _ = time::timeout(Duration::from_secs(3), find_connected_devices(&mut output))
                    .await;
            }
        }
    })
}

// Helper to create a Sink that forwards to a channel
struct ChannelSink {
    tx: tokio::sync::mpsc::UnboundedSender<Message>,
}

impl Sink<Message> for ChannelSink {
    type Error = ();

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        let _ = self.tx.send(item);
        Ok(())
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }
}

/// Configuration for a device watcher
struct DeviceWatcherConfig {
    /// Device identifier (e.g., "mount", "telescope_controller")
    device_id: &'static str,
    /// Function to get the device from CONNECTED_DEVICES
    get_device: fn(&ServerInstance) -> Option<ActiveDevice>,
    /// Function to set the device to None in CONNECTED_DEVICES
    clear_device: fn(&mut ServerInstance),
    /// Function to get/set the device name in ConnectedDevices message
    get_connected_name: fn(&ConnectedDevices) -> Option<String>,
    set_connected_name: fn(&mut ConnectedDevices, Option<String>),
    /// Function to spawn the watcher task
    spawn_watcher: fn(
        ActiveDevice,
        tokio::sync::mpsc::UnboundedSender<Message>,
    ) -> tokio::task::JoinHandle<()>,
}

/// Generic param watcher that handles all devices
/// Checks timeout before dispatching to device-specific handlers
/// Drops devices if no telemetry received for 2 seconds
pub fn param_watcher() -> impl Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        const DATA_TIMEOUT: Duration = Duration::from_secs(2);
        let mut timeout_check = interval(Duration::from_millis(500));
        timeout_check.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        // Device watcher configurations
        let device_configs: Vec<DeviceWatcherConfig> = vec![
            DeviceWatcherConfig {
                device_id: "mount",
                get_device: |devices| devices.mount.clone(),
                clear_device: |devices| devices.mount = None,
                get_connected_name: |cd| cd.mount.clone(),
                set_connected_name: |cd, name| cd.mount = name,
                spawn_watcher: |device, tx| {
                    tokio::spawn(async move {
                        if device
                            .change("CONNECTION", vec![("CONNECT", true)])
                            .await
                            .is_ok()
                        {
                            let mut channel_sink = ChannelSink { tx };
                            mount::watch_coordinates(device, &mut channel_sink).await;
                        }
                    })
                },
            },
            DeviceWatcherConfig {
                device_id: "telescope_controller",
                get_device: |devices| devices.telescope_controller.clone(),
                clear_device: |devices| devices.telescope_controller = None,
                get_connected_name: |cd| cd.telescope_controller.clone(),
                set_connected_name: |cd, name| cd.telescope_controller = name,
                spawn_watcher: |device, tx| {
                    tokio::spawn(async move {
                        if device
                            .change("CONNECTION", vec![("CONNECT", true)])
                            .await
                            .is_ok()
                        {
                            let mut channel_sink = ChannelSink { tx };
                            telescope_controller::watch_telemetry(device, &mut channel_sink).await;
                        }
                    })
                },
            },
        ];

        // Track active watcher tasks by device ID
        let mut device_tasks: HashMap<String, Option<tokio::task::JoinHandle<()>>> = device_configs
            .iter()
            .map(|config| (config.device_id.to_string(), None))
            .collect();

        // Channel for device handlers to send messages
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

        loop {
            tokio::select! {
                // Forward messages from device handlers to output
                msg = rx.recv() => {
                    if let Some(msg) = msg {
                        let _ = output.send(msg).await;
                    }
                }

                // Timeout check
                _ = timeout_check.tick() => {
                    // Wait until we have a client
                    let _client_instance: Arc<IndiClientInstance> = loop {
                        if let Some(c) = INDI_CLIENT.read().await.as_ref().cloned() {
                            break c;
                        }
                        time::sleep(Duration::from_millis(100)).await;
                    };

                    // Process each device
                    for config in &device_configs {
                        let (device, should_drop) = {
                            let devices = CONNECTED_DEVICES.read().await;
                            let telemetry = TELEMETRY_TIMES.read().await;

                            let device = (config.get_device)(&devices);
                            let should_drop = if let Some(last) = telemetry.get(config.device_id) {
                                last.elapsed() > DATA_TIMEOUT
                            } else {
                                false
                            };

                            (device, should_drop)
                        };

                        let task_key = config.device_id.to_string();
                        let task = device_tasks.get_mut(&task_key).unwrap();

                        if should_drop {
                            // Drop device due to timeout
                            if let Some(handle) = task.take() {
                                handle.abort();
                            }

                            // Note: We don't have easy access to device names here
                            // device_discovery_watcher will send the correct state within 1 second

                            {
                                let mut devices = CONNECTED_DEVICES.write().await;
                                (config.clear_device)(&mut devices);
                            }
                            {
                                let mut telemetry = TELEMETRY_TIMES.write().await;
                                telemetry.remove(config.device_id);
                            }

                            // Build ConnectedDevices message - set dropped device to None, keep others
                            // Since we don't have easy access to other device names here,
                            // we'll send None for all and let device_discovery_watcher send the correct state
                            // This is acceptable since discovery runs every second
                            let mut connected_devices = ConnectedDevices {
                                mount: None,
                                camera: None,
                                focuser: None,
                                telescope_controller: None,
                            };
                            (config.set_connected_name)(&mut connected_devices, None);
                            let _ = output.send(Message::ConnectedDeviceChange(connected_devices)).await;
                        } else if let Some(device) = device {
                            // Start watcher if not already running
                            if task.is_none() || task.as_ref().unwrap().is_finished() {
                                let device_clone = device.clone();
                                let tx_clone = tx.clone();
                                *task = Some((config.spawn_watcher)(device_clone, tx_clone));
                            }
                        }
                    }
                }
            }
        }
    })
}

/// Separate thread that looks for INDI server disconnects
/// Does 5 retries with the same logic, then goes back to disconnected
pub fn server_disconnect_watcher() -> impl Stream<Item = Message> {
    stream::channel(100, |mut output| async move {
        let mut check_interval = interval(Duration::from_secs(1));
        check_interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
        const MAX_RETRIES: u32 = 5;
        const RECONNECT_DELAY_MS: u64 = 1000;

        loop {
            check_interval.tick().await;

            // Check if we have a client
            let (has_client, server_addr) = {
                let client = INDI_CLIENT.read().await;
                if let Some(c) = client.as_ref() {
                    (true, Some(c.ip.clone()))
                } else {
                    (false, None)
                }
            };

            if has_client {
                if let Some(addr) = server_addr {
                    // Check if server is still reachable
                    if !tcp_probe(&addr).await {
                        // Server disconnected - try to reconnect
                        let _ = output
                            .send(Message::ServerStatus(ServerStatus::ConnectionLost))
                            .await;

                        // Clear client
                        {
                            let mut guard = INDI_CLIENT.write().await;
                            *guard = None;
                        }

                        // Try to reconnect up to MAX_RETRIES times
                        let mut retries = 0;
                        while retries < MAX_RETRIES {
                            match connect_to_server(addr.clone()).await {
                                Ok(()) => {
                                    let _ = output
                                        .send(Message::ServerStatus(ServerStatus::Connected))
                                        .await;
                                    break;
                                }
                                Err(_) => {
                                    retries += 1;
                                    if retries < MAX_RETRIES {
                                        time::sleep(Duration::from_millis(RECONNECT_DELAY_MS))
                                            .await;
                                    }
                                }
                            }
                        }

                        // If we exhausted retries, stay disconnected
                        if retries >= MAX_RETRIES {
                            let _ = output
                                .send(Message::ServerStatus(ServerStatus::Disconnected))
                                .await;
                        }
                    }
                }
            }
        }
    })
}
