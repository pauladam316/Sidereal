// Cargo.toml
// src/main.rs
mod camera;
mod client;
mod colors;
mod events;
mod scene;
mod server;
mod star_catalog;
mod starfield;
mod target;
mod ui;
use crate::events::PlanetariumEvent;
use crate::target::TargetPlugin;
use crate::ui::MenuPlugin;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
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
struct EventChannelReceiver(pub Mutex<Receiver<PlanetariumEvent>>);

fn main() {
    // build the std channel
    let (event_tx, event_rx): (Sender<PlanetariumEvent>, Receiver<PlanetariumEvent>) = channel();

    // spawn gRPC client for testing
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            // wait a bit for server to come up
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            if let Err(e) = client::send_event("hello from client".to_string()).await {
                eprintln!("Client error: {e:?}");
            }
        });
    });

    // spawn gRPC server, handing off loc_tx…
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            server::run(event_tx).await.expect("gRPC server failed");
        });
    });

    App::new()
        .insert_resource(ClearColor(Color::linear_rgba(
            0.0 / 255.0,
            0.0 / 255.0,
            3.0 / 255.0,
            255.0,
        )))
        .insert_resource(EventChannelReceiver(Mutex::new(event_rx)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Planetarium".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default())
        .add_plugins(CameraPlugin)
        .add_plugins(StarfieldPlugin)
        .add_plugins(ScenePlugin)
        .add_plugins(MenuPlugin)
        .add_systems(Update, event_listener_system)
        .add_plugins(TargetPlugin)
        .run();
}
fn event_listener_system(
    event_rx: Res<EventChannelReceiver>,
    mut ev: MessageWriter<PlanetariumEvent>,
) {
    // lock *briefly*
    if let Ok(receiver) = event_rx.0.lock() {
        while let Ok(evt) = receiver.try_recv() {
            ev.write(evt);
        }
    }
}
