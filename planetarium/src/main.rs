// Cargo.toml
// src/main.rs
mod camera;
mod scene;
mod server;
mod star_catalog;
mod starfield;
mod target;
use crate::starfield::SetLocationEvent;
use crate::target::TargetPlugin;

use bevy::prelude::*;
use camera::CameraPlugin;
use scene::ScenePlugin;
use starfield::StarfieldPlugin;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

#[derive(Component)]
struct Star;

// new component: stores the unit‐direction of each star
#[derive(Component)]
struct StarDirection();

#[derive(Resource)]
struct LocationChannelReceiver(pub Mutex<Receiver<SetLocationEvent>>);

fn main() {
    // build the std channel
    let (loc_tx, loc_rx): (Sender<SetLocationEvent>, Receiver<SetLocationEvent>) = channel();

    // spawn gRPC server, handing off loc_tx…
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            server::run(loc_tx).await.expect("gRPC server failed");
        });
    });

    App::new()
        .insert_resource(ClearColor(Color::linear_rgba(
            0.0 / 255.0,
            0.0 / 255.0,
            3.0 / 255.0,
            255.0,
        )))
        .insert_resource(LocationChannelReceiver(Mutex::new(loc_rx)))
        .add_plugins(DefaultPlugins)
        .add_plugins(CameraPlugin)
        .add_plugins(StarfieldPlugin)
        .add_plugins(ScenePlugin)
        .add_systems(Update, location_channel_listener_system)
        .add_plugins(TargetPlugin)
        .run();
}
fn location_channel_listener_system(
    loc_rx: Res<LocationChannelReceiver>,
    mut ev: EventWriter<SetLocationEvent>,
) {
    // lock *briefly*
    if let Ok(receiver) = loc_rx.0.lock() {
        while let Ok(evt) = receiver.try_recv() {
            ev.write(evt);
        }
    }
}
