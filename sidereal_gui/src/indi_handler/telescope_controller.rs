use super::TELEMETRY_TIMES;
use crate::{
    app::Message,
    gui::tabs::telescope::Message as TelescopeMessage,
    model::{SiderealError, SiderealResult},
};
use iced::futures::{Sink, SinkExt, StreamExt};
use indi::client::active_device::ActiveDevice;
use std::{collections::HashMap, time::Instant};

use super::CONNECTED_DEVICES;

/// Control heater 1 (enable/disable)
pub async fn set_heater1(enabled: bool) -> SiderealResult<()> {
    let devices = CONNECTED_DEVICES.read().await;
    match &devices.telescope_controller {
        Some(device) => {
            let switch_name = if enabled { "HEATER1_ON" } else { "HEATER1_OFF" };
            device
                .change("HEATER1", vec![(switch_name, true)])
                .await
                .map_err(|e| {
                    SiderealError::ServerError(format!("Heater1 control failed: {:?}", e))
                })?;
            Ok(())
        }
        None => Err(SiderealError::ServerError(
            "Telescope Controller device not available. Please ensure the device is connected to the INDI server.".to_owned(),
        )),
    }
}

/// Control heater 2 (enable/disable)
pub async fn set_heater2(enabled: bool) -> SiderealResult<()> {
    let devices = CONNECTED_DEVICES.read().await;
    match &devices.telescope_controller {
        Some(device) => {
            let switch_name = if enabled { "HEATER2_ON" } else { "HEATER2_OFF" };
            device
                .change("HEATER2", vec![(switch_name, true)])
                .await
                .map_err(|e| {
                    SiderealError::ServerError(format!("Heater2 control failed: {:?}", e))
                })?;
            Ok(())
        }
        None => Err(SiderealError::ServerError(
            "Telescope Controller device not available. Please ensure the device is connected to the INDI server.".to_owned(),
        )),
    }
}

/// Control heater 3 (enable/disable)
pub async fn set_heater3(enabled: bool) -> SiderealResult<()> {
    let devices = CONNECTED_DEVICES.read().await;
    match &devices.telescope_controller {
        Some(device) => {
            let switch_name = if enabled { "HEATER3_ON" } else { "HEATER3_OFF" };
            device
                .change("HEATER3", vec![(switch_name, true)])
                .await
                .map_err(|e| {
                    SiderealError::ServerError(format!("Heater3 control failed: {:?}", e))
                })?;
            Ok(())
        }
        None => Err(SiderealError::ServerError(
            "Telescope Controller device not available. Please ensure the device is connected to the INDI server.".to_owned(),
        )),
    }
}

/// Control lens cap (open/close)
pub async fn set_lens_cap(open: bool) -> SiderealResult<()> {
    let devices = CONNECTED_DEVICES.read().await;
    match &devices.telescope_controller {
        Some(device) => {
            let switch_name = if open {
                "LENS_CAP_OPEN"
            } else {
                "LENS_CAP_CLOSE"
            };
            device
                .change("LENS_CAP", vec![(switch_name, true)])
                .await
                .map_err(|e| {
                    SiderealError::ServerError(format!("Lens cap control failed: {:?}", e))
                })?;
            Ok(())
        }
        None => Err(SiderealError::ServerError(
            "Telescope Controller device not available. Please ensure the device is connected to the INDI server.".to_owned(),
        )),
    }
}

/// Control flat light (on/off)
pub async fn set_flat_light(on: bool) -> SiderealResult<()> {
    let devices = CONNECTED_DEVICES.read().await;
    match &devices.telescope_controller {
        Some(device) => {
            let switch_name = if on {
                "FLAT_LIGHT_ON"
            } else {
                "FLAT_LIGHT_OFF"
            };
            device
                .change("FLAT_LIGHT", vec![(switch_name, true)])
                .await
                .map_err(|e| {
                    SiderealError::ServerError(format!("Flat light control failed: {:?}", e))
                })?;
            Ok(())
        }
        None => Err(SiderealError::ServerError(
            "Telescope Controller device not available. Please ensure the device is connected to the INDI server.".to_owned(),
        )),
    }
}

/// Watch for telemetry updates and send them to the UI
/// This function runs until the connection is lost
pub async fn watch_telemetry<S>(device: ActiveDevice, output: &mut S)
where
    S: Sink<Message> + Unpin,
{
    // Get the telemetry parameter
    let param_notify = match device.get_parameter("TELEMETRY").await {
        Ok(p) => p,
        Err(_) => {
            // If we can't get telemetry, just return
            return;
        }
    };

    // Subscribe to telemetry updates
    let mut changes = param_notify.subscribe().await;

    // Event loop - just process data, timeout is handled by generic param_watcher
    loop {
        match changes.next().await {
            Some(Ok(param_arc)) => {
                if let Ok(map) = param_arc.get_values::<HashMap<String, indi::Number>>() {
                    // Update telemetry time
                    {
                        let mut telemetry = TELEMETRY_TIMES.write().await;
                        telemetry.insert("telescope_controller".to_string(), Instant::now());
                    }

                    // Extract telemetry values - convert Sexagesimal to f64
                    let ambient_temp = map.get("AMBIENT_TEMP").map(|n| n.value.into());
                    let heater1_temp = map.get("HEATER1_TEMP").map(|n| n.value.into());
                    let heater2_temp = map.get("HEATER2_TEMP").map(|n| n.value.into());
                    let heater3_temp = map.get("HEATER3_TEMP").map(|n| n.value.into());

                    // Extract states - convert Sexagesimal to f64 then to u8
                    let lens_cap_state = map.get("LENS_CAP_REAL_STATE").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });
                    let flat_light_state = map.get("FLAT_LIGHT_REAL_STATE").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });
                    let heater1_state = map.get("HEATER1_REAL_STATE").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });
                    let heater2_state = map.get("HEATER2_REAL_STATE").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });
                    let heater3_state = map.get("HEATER3_REAL_STATE").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });

                    // Extract manual override states
                    let lens_cap_manual = map.get("LENS_CAP_MANUAL_STATE").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });
                    let flat_light_manual = map.get("FLAT_LIGHT_MANUAL_STATE").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });
                    let heater1_manual = map.get("HEATER1_MANUAL_STATE").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });
                    let heater2_manual = map.get("HEATER2_MANUAL_STATE").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });
                    let heater3_manual = map.get("HEATER3_MANUAL_STATE").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });

                    let _ = output
                        .send(Message::Telescope(TelescopeMessage::TelemetryUpdate {
                            ambient_temp: ambient_temp.unwrap_or(0.0),
                            heater1_temp: heater1_temp.unwrap_or(0.0),
                            heater2_temp: heater2_temp.unwrap_or(0.0),
                            heater3_temp: heater3_temp.unwrap_or(0.0),
                            lens_cap_open: lens_cap_state.map(|s| s != 0).unwrap_or(false),
                            flat_light_on: flat_light_state.map(|s| s != 0).unwrap_or(false),
                            heater1_on: heater1_state.map(|s| s != 0).unwrap_or(false),
                            heater2_on: heater2_state.map(|s| s != 0).unwrap_or(false),
                            heater3_on: heater3_state.map(|s| s != 0).unwrap_or(false),
                            lens_cap_manual_override: lens_cap_manual
                                .map(|s| s != 0)
                                .unwrap_or(false),
                            flat_light_manual_override: flat_light_manual
                                .map(|s| s != 0)
                                .unwrap_or(false),
                            heater1_manual_override: heater1_manual
                                .map(|s| s != 0)
                                .unwrap_or(false),
                            heater2_manual_override: heater2_manual
                                .map(|s| s != 0)
                                .unwrap_or(false),
                            heater3_manual_override: heater3_manual
                                .map(|s| s != 0)
                                .unwrap_or(false),
                        }))
                        .await;
                }
            }
            Some(Err(_)) => {
                // Stream error - connection lost
                break;
            }
            None => {
                // Stream ended - connection lost
                break;
            }
        }
    }
}
