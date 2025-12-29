use super::TELEMETRY_TIMES;
use crate::{
    app::Message,
    gui::tabs::observatory::Message as ObservatoryMessage,
    model::{SiderealError, SiderealResult},
};
use iced::futures::{Sink, SinkExt, StreamExt};
use indi::client::active_device::ActiveDevice;
use std::time::Instant;

use super::CONNECTED_DEVICES;

/// Arm the roof controller system
pub async fn arm_system() -> SiderealResult<()> {
    let devices = CONNECTED_DEVICES.read().await;
    match &devices.roof_controller {
        Some(device) => {
            println!("[Roof Controller] Sending ARM command to INDI driver");
            println!("[Roof Controller] Command: ARM_CONTROL property, switch: ARM = true");
            device
                .change("ARM_CONTROL", vec![("ARM", true)])
                .await
                .map_err(|e| {
                    println!("[Roof Controller] Arm command failed: {:?}", e);
                    SiderealError::ServerError(format!("Arm control failed: {:?}", e))
                })?;
            println!("[Roof Controller] ARM command sent successfully");
            Ok(())
        }
        None => {
            println!("[Roof Controller] Arm command failed: device not available");
            Err(SiderealError::ServerError(
                "Roof Controller device not available. Please ensure the device is connected to the INDI server.".to_owned(),
            ))
        }
    }
}

/// Disarm the roof controller system
pub async fn disarm_system() -> SiderealResult<()> {
    let devices = CONNECTED_DEVICES.read().await;
    match &devices.roof_controller {
        Some(device) => {
            println!("[Roof Controller] Sending DISARM command to INDI driver");
            println!("[Roof Controller] Command: ARM_CONTROL property, switch: DISARM = true");
            device
                .change("ARM_CONTROL", vec![("DISARM", true)])
                .await
                .map_err(|e| {
                    println!("[Roof Controller] Disarm command failed: {:?}", e);
                    SiderealError::ServerError(format!("Disarm control failed: {:?}", e))
                })?;
            println!("[Roof Controller] DISARM command sent successfully");
            Ok(())
        }
        None => {
            println!("[Roof Controller] Disarm command failed: device not available");
            Err(SiderealError::ServerError(
                "Roof Controller device not available. Please ensure the device is connected to the INDI server.".to_owned(),
            ))
        }
    }
}

/// Open the roof
pub async fn open_roof() -> SiderealResult<()> {
    let devices = CONNECTED_DEVICES.read().await;
    match &devices.roof_controller {
        Some(device) => {
            println!("[Roof Controller] Sending OPEN ROOF command to INDI driver");
            println!("[Roof Controller] Command: ROOF_CONTROL property, switch: ROOF_OPEN = true");
            device
                .change("ROOF_CONTROL", vec![("ROOF_OPEN", true)])
                .await
                .map_err(|e| {
                    println!("[Roof Controller] Open roof command failed: {:?}", e);
                    SiderealError::ServerError(format!("Roof open failed: {:?}", e))
                })?;
            println!("[Roof Controller] OPEN ROOF command sent successfully");
            Ok(())
        }
        None => {
            println!("[Roof Controller] Open roof command failed: device not available");
            Err(SiderealError::ServerError(
                "Roof Controller device not available. Please ensure the device is connected to the INDI server.".to_owned(),
            ))
        }
    }
}

/// Close the roof
pub async fn close_roof() -> SiderealResult<()> {
    let devices = CONNECTED_DEVICES.read().await;
    match &devices.roof_controller {
        Some(device) => {
            println!("[Roof Controller] Sending CLOSE ROOF command to INDI driver");
            println!("[Roof Controller] Command: ROOF_CONTROL property, switch: ROOF_CLOSE = true");
            device
                .change("ROOF_CONTROL", vec![("ROOF_CLOSE", true)])
                .await
                .map_err(|e| {
                    println!("[Roof Controller] Close roof command failed: {:?}", e);
                    SiderealError::ServerError(format!("Roof close failed: {:?}", e))
                })?;
            println!("[Roof Controller] CLOSE ROOF command sent successfully");
            Ok(())
        }
        None => {
            println!("[Roof Controller] Close roof command failed: device not available");
            Err(SiderealError::ServerError(
                "Roof Controller device not available. Please ensure the device is connected to the INDI server.".to_owned(),
            ))
        }
    }
}

/// Stop the roof
pub async fn stop_roof() -> SiderealResult<()> {
    let devices = CONNECTED_DEVICES.read().await;
    match &devices.roof_controller {
        Some(device) => {
            println!("[Roof Controller] Sending STOP ROOF command to INDI driver");
            println!("[Roof Controller] Command: ROOF_CONTROL property, switch: ROOF_STOP = true");
            device
                .change("ROOF_CONTROL", vec![("ROOF_STOP", true)])
                .await
                .map_err(|e| {
                    println!("[Roof Controller] Stop roof command failed: {:?}", e);
                    SiderealError::ServerError(format!("Roof stop failed: {:?}", e))
                })?;
            println!("[Roof Controller] STOP ROOF command sent successfully");
            Ok(())
        }
        None => {
            println!("[Roof Controller] Stop roof command failed: device not available");
            Err(SiderealError::ServerError(
                "Roof Controller device not available. Please ensure the device is connected to the INDI server.".to_owned(),
            ))
        }
    }
}

/// Engage the lock
pub async fn engage_lock() -> SiderealResult<()> {
    let devices = CONNECTED_DEVICES.read().await;
    match &devices.roof_controller {
        Some(device) => {
            println!("[Roof Controller] Sending ENGAGE LOCK command to INDI driver");
            println!(
                "[Roof Controller] Command: LOCK_CONTROL property, switch: LOCK_ENGAGE = true"
            );
            device
                .change("LOCK_CONTROL", vec![("LOCK_ENGAGE", true)])
                .await
                .map_err(|e| {
                    println!("[Roof Controller] Engage lock command failed: {:?}", e);
                    SiderealError::ServerError(format!("Lock engage failed: {:?}", e))
                })?;
            println!("[Roof Controller] ENGAGE LOCK command sent successfully");
            Ok(())
        }
        None => {
            println!("[Roof Controller] Engage lock command failed: device not available");
            Err(SiderealError::ServerError(
                "Roof Controller device not available. Please ensure the device is connected to the INDI server.".to_owned(),
            ))
        }
    }
}

/// Disengage the lock
pub async fn disengage_lock() -> SiderealResult<()> {
    let devices = CONNECTED_DEVICES.read().await;
    match &devices.roof_controller {
        Some(device) => {
            println!("[Roof Controller] Sending DISENGAGE LOCK command to INDI driver");
            println!(
                "[Roof Controller] Command: LOCK_CONTROL property, switch: LOCK_DISENGAGE = true"
            );
            device
                .change("LOCK_CONTROL", vec![("LOCK_DISENGAGE", true)])
                .await
                .map_err(|e| {
                    println!("[Roof Controller] Disengage lock command failed: {:?}", e);
                    SiderealError::ServerError(format!("Lock disengage failed: {:?}", e))
                })?;
            println!("[Roof Controller] DISENGAGE LOCK command sent successfully");
            Ok(())
        }
        None => {
            println!("[Roof Controller] Disengage lock command failed: device not available");
            Err(SiderealError::ServerError(
                "Roof Controller device not available. Please ensure the device is connected to the INDI server.".to_owned(),
            ))
        }
    }
}

/// Stop the lock
pub async fn stop_lock() -> SiderealResult<()> {
    let devices = CONNECTED_DEVICES.read().await;
    match &devices.roof_controller {
        Some(device) => {
            println!("[Roof Controller] Sending STOP LOCK command to INDI driver");
            println!("[Roof Controller] Command: LOCK_CONTROL property, switch: LOCK_STOP = true");
            device
                .change("LOCK_CONTROL", vec![("LOCK_STOP", true)])
                .await
                .map_err(|e| {
                    println!("[Roof Controller] Stop lock command failed: {:?}", e);
                    SiderealError::ServerError(format!("Lock stop failed: {:?}", e))
                })?;
            println!("[Roof Controller] STOP LOCK command sent successfully");
            Ok(())
        }
        None => {
            println!("[Roof Controller] Stop lock command failed: device not available");
            Err(SiderealError::ServerError(
                "Roof Controller device not available. Please ensure the device is connected to the INDI server.".to_owned(),
            ))
        }
    }
}

/// Watch telemetry from the roof controller
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
                if let Ok(map) =
                    param_arc.get_values::<std::collections::HashMap<String, indi::Number>>()
                {
                    // Update telemetry time
                    {
                        let mut telemetry = TELEMETRY_TIMES.write().await;
                        telemetry.insert("roof_controller".to_string(), Instant::now());
                    }

                    let h_bridge_current = map.get("H_BRIDGE_CURRENT").map(|n| n.value.into());
                    let voltage_5v = map.get("VOLTAGE_5V").map(|n| n.value.into());
                    let voltage_12v = map.get("VOLTAGE_12V").map(|n| n.value.into());
                    let limit_u1 = map.get("LIMIT_U1").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });
                    let limit_u2 = map.get("LIMIT_U2").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });
                    let limit_l1 = map.get("LIMIT_L1").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });
                    let limit_l2 = map.get("LIMIT_L2").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });
                    let arm_state = map.get("ARM_STATE").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });
                    let lock_state = map.get("LOCK_STATE").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });

                    // Get roof status - try to get it but don't fail if unavailable
                    // We'll use telemetry data for roof state instead of separate ROOF_STATUS parameter
                    let roof_position = None::<Option<f64>>;
                    let roof_is_open = None::<Option<bool>>;
                    let roof_is_closed = None::<Option<bool>>;

                    // Use roof_state from telemetry to determine open/closed
                    let roof_state_val = map.get("ROOF_STATE").map(|n| {
                        let val: f64 = n.value.into();
                        val as u8
                    });
                    let roof_is_open_val =
                        roof_state_val.map(|s| s == 1 || s == 2).unwrap_or(false); // 1=opening, 2=open
                    let roof_is_closed_val =
                        roof_state_val.map(|s| s == 3 || s == 4).unwrap_or(false); // 3=closing, 4=closed
                    let roof_position_val = map
                        .get("POSITION")
                        .map(|n| {
                            let val: f64 = n.value.into();
                            val
                        })
                        .unwrap_or(0.0);

                    let _ = output
                        .send(Message::Observatory(ObservatoryMessage::TelemetryUpdate {
                            is_armed: arm_state.map(|s| s != 0).unwrap_or(false),
                            roof_is_open: roof_is_open_val,
                            roof_is_closed: roof_is_closed_val,
                            roof_position: roof_position_val,
                            lock_engaged: lock_state.map(|s| s == 1).unwrap_or(false),
                            voltage_5v: voltage_5v.unwrap_or(0.0),
                            voltage_12v: voltage_12v.unwrap_or(0.0),
                            actuator_current: h_bridge_current.unwrap_or(0.0),
                            limit_u1: limit_u1.map(|s| s != 0).unwrap_or(false),
                            limit_u2: limit_u2.map(|s| s != 0).unwrap_or(false),
                            limit_l1: limit_l1.map(|s| s != 0).unwrap_or(false),
                            limit_l2: limit_l2.map(|s| s != 0).unwrap_or(false),
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
