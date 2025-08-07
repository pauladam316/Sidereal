// Cargo.toml
// src/main.rs
mod camera;
mod scene;
mod server;
mod star_catalog;
mod starfield;
use bevy::ecs::system::ParamSet; // make sure this is in scope
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::render::mesh::shape::Quad;
use camera::CameraPlugin;
use protos::protos::planetarium_server::{self, PlanetariumServer};
use rand::Rng;
use scene::ScenePlugin;
use starfield::StarfieldPlugin;
#[derive(Component)]
struct Star;

// new component: stores the unit‐direction of each star
#[derive(Component)]
struct StarDirection(Vec3);

fn main() {
    // Launch gRPC server in a separate thread
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            server::run().await.expect("gRPC server failed");
        });
    });

    // Launch Bevy app
    App::new()
        .insert_resource(ClearColor(Color::RgbaLinear {
            red: 0.0 / 255.0,
            green: 0.0 / 255.0,
            blue: 3.0 / 255.0,
            alpha: 255.0,
        }))
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraPlugin)
        .add_plugin(StarfieldPlugin)
        .add_plugin(ScenePlugin)
        .run();
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
