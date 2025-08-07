use once_cell::sync::Lazy;
use protos::protos::{planetarium_client::PlanetariumClient, SetLocationRequest};
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::{
    io,
    process::{Child, Command, Stdio},
};
use tokio::sync::Mutex;
use tonic::transport::Channel;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::{
    config::GLOBAL_CONFIG,
    model::{SiderealError, SiderealResult},
};

/// A global place to store our planetarium child handle.
static PLANETARIUM_PROCESS: Lazy<Mutex<Option<Child>>> = Lazy::new(|| Mutex::new(None));
static PLANETARIUM_CLIENT: Lazy<Mutex<Option<PlanetariumClient<Channel>>>> =
    Lazy::new(|| Mutex::new(None));

/// Spawn & detach the process, returning its Child handle.
fn spawn_and_detach(path: &str) -> io::Result<Child> {
    let mut binding = Command::new(path);
    let cmd = binding
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(unix)]
    {
        cmd.before_exec(|| {
            unsafe {
                libc::setsid();
            }
            Ok(())
        });
    }

    #[cfg(windows)]
    {
        const DETACHED: u32 = 0x00000008;
        const NEW_GROUP: u32 = 0x00000200;
        cmd.creation_flags(DETACHED | NEW_GROUP);
    }

    cmd.spawn()
}

/// Launches “planetarium” only if our tracked process has exited (or wasn’t started yet).
pub async fn launch_planetarium() -> io::Result<()> {
    let mut planetarium_lock = PLANETARIUM_PROCESS.lock().await;
    let mut client_lock = PLANETARIUM_CLIENT.lock().await;
    // If we have a child, see if it's still running
    if let Some(child) = planetarium_lock.as_mut() {
        match child.try_wait()? {
            None => {
                // Still running
                return Ok(());
            }
            Some(_status) => {
                // It exited; we’ll drop it and spawn a new one
                *planetarium_lock = None;
            }
        }
    }

    // Spawn and store the new handle
    let child = spawn_and_detach("planetarium")?;
    *client_lock = Some(
        PlanetariumClient::connect("http://[::1]:50051")
            .await
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?,
    );

    *planetarium_lock = Some(child);
    Ok(())
}

pub async fn set_location() -> SiderealResult<()> {
    let guard = GLOBAL_CONFIG.write().await;
    let mut client_lock = PLANETARIUM_CLIENT.lock().await;
    if let Some(client) = client_lock.as_mut() {
        let request = SetLocationRequest {
            latitude: guard.location.latitude,
            longitude: guard.location.longitude,
            altitude: guard.location.altitude,
        };
        let response =
            client
                .set_location(request)
                .await
                .map_err(|e| SiderealError::ServerError {
                    reason: e.to_string(),
                })?;
        println!("{}", response.into_inner().description);
    }

    Ok(())
}
