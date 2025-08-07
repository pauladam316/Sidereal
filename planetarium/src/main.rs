// Cargo.toml
// src/main.rs
mod camera;
mod scene;
mod server;
mod star_catalog;
mod starfield;
use crate::starfield::{rotate_starfield_system, SetLocationEvent};
use bevy::ecs::system::ParamSet; // make sure this is in scope
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::mesh::shape::Quad;
use camera::CameraPlugin;
use protos::protos::planetarium_server::{self, PlanetariumServer};
use rand::Rng;
use scene::ScenePlugin;
use starfield::StarfieldPlugin;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

#[derive(Component)]
struct Star;

// new component: stores the unit‐direction of each star
#[derive(Component)]
struct StarDirection(Vec3);

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
        .insert_resource(ClearColor(Color::RgbaLinear {
            red: 0.0 / 255.0,
            green: 0.0 / 255.0,
            blue: 3.0 / 255.0,
            alpha: 255.0,
        }))
        .insert_resource(LocationChannelReceiver(Mutex::new(loc_rx)))
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraPlugin)
        .add_plugin(StarfieldPlugin)
        .add_plugin(ScenePlugin)
        .add_system(location_channel_listener_system)
        .run();
}
fn location_channel_listener_system(
    loc_rx: Res<LocationChannelReceiver>,
    mut ev: EventWriter<SetLocationEvent>,
) {
    // lock *briefly*
    if let Ok(mut receiver) = loc_rx.0.lock() {
        while let Ok(evt) = receiver.try_recv() {
            ev.send(evt);
        }
    }
}
// /// Re‐position each star at “infinite” distance along its direction
// fn star_infinity_system(
//     mut params: ParamSet<(
//         Query<&Transform, With<Camera>>,            // 0: read‐only camera transform
//         Query<(&StarDirection, &mut Transform)>,    // 1: read StarDirection + write star Transform
//     )>,
// ) {
//     // 1) get camera translation
//     let cam_pos = params
//         .p0()               // first query
//         .single()           // there’s only one camera
//         .translation;

//     const INF_DIST: f32 = 10_000.0;
//     // 2) update each star in turn
//     for (star_dir, mut tf) in params.p1().iter_mut() {
//         // star_dir.0 is the Vec3 unit‐direction
//         tf.translation = cam_pos + star_dir.0 * INF_DIST;
//     }
// }

// /// Rotates each quad to always face the camera
// fn billboard_system(
//     camera_q: Query<&GlobalTransform, With<Camera>>,
//     mut stars: Query<&mut Transform, With<Star>>,
// ) {
//     let cam_global = camera_q.single();
//     let cam_rot = cam_global.compute_transform().rotation;
//     for mut star_tf in stars.iter_mut() {
//         star_tf.rotation = cam_rot;
//     }
// }
