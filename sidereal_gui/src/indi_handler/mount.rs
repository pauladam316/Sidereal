use super::TELEMETRY_TIMES;
use crate::{
    app::Message,
    gui::tabs::mount::Message as MountMessage,
    model::{SiderealError, SiderealResult},
};
use iced::futures::{Sink, SinkExt, StreamExt};
use indi::client::active_device::ActiveDevice;
use std::{collections::HashMap, time::Instant};

use super::CONNECTED_DEVICES;

/// Move the mount in a specific direction
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
            "Mount device not available. Please ensure the mount is connected to the INDI server."
                .to_owned(),
        )),
    }
}

/// Stop all mount movement
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

/// Watch for mount coordinate updates and send them to the UI
/// This function runs until the connection is lost
pub async fn watch_coordinates<S>(mount: ActiveDevice, output: &mut S)
where
    S: Sink<Message> + Unpin,
{
    // Get the parameter we care about
    let param_notify = match mount.get_parameter("EQUATORIAL_EOD_COORD").await {
        Ok(p) => p,
        Err(_) => match mount.get_parameter("EQUATORIAL_COORD").await {
            Ok(p) => p,
            Err(_) => {
                // If we can't get coordinates, just return
                return;
            }
        },
    };

    // Subscribe to parameter changes
    let mut changes = param_notify.subscribe().await;

    // Event loop - just process data, timeout is handled by generic param_watcher
    loop {
        match changes.next().await {
            Some(Ok(param_arc)) => {
                if let Ok(map) = param_arc.get_values::<HashMap<String, indi::Number>>() {
                    if let (Some(ra), Some(dec)) = (map.get("RA"), map.get("DEC")) {
                        // Update telemetry time
                        {
                            let mut telemetry = TELEMETRY_TIMES.write().await;
                            telemetry.insert("mount".to_string(), Instant::now());
                        }

                        let _ = output
                            .send(Message::Mount(MountMessage::CoordsUpdated {
                                ra_hours: ra.value.into(),
                                dec_deg: dec.value.into(),
                            }))
                            .await;
                    }
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
