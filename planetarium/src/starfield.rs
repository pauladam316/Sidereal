

use std::f64::consts::PI;
use std::{env, fs};
use std::path::PathBuf;
use std::str::FromStr;
use chrono::{DateTime, Utc};
use bevy::prelude::*;
use bevy::render::mesh::shape::Quad;
use rand::Rng;
use bevy::ecs::system::ParamSet;

use crate::star_catalog::parse_catalog;

#[derive(Component)]
pub struct Star;

#[derive(Component)]
pub struct StarDirection(pub Vec3);

pub struct StarfieldPlugin;

impl Plugin for StarfieldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(spawn_starfield)
            .add_system(star_infinity_system.before(billboard_system))
            .add_system(billboard_system);
    }
}


/// Compute the Julian Date (JD) from a UTC time.
fn julian_date(time: DateTime<Utc>) -> f64 {
    // Unix epoch JD = 2440587.5
    let unix_seconds = time.timestamp() as f64;
    let subseconds   = time.timestamp_subsec_nanos() as f64 / 1e9;
    2440587.5 + (unix_seconds + subseconds) / 86400.0
}

/// Given:
/// - `time`:  current UTC
/// - `lat`:   observer latitude, radians (φ)
/// - `lon`:   observer longitude, radians **east** of Greenwich (λ)
/// - `ra`:    star Right Ascension, radians (α)
/// - `dec`:   star Declination, radians (δ)
///
/// returns a **unit** vector in the **local horizon** frame:
/// - X points East  
/// - Y points North  
/// - Z points Zenith (Up)
pub fn star_direction(
    time: DateTime<Utc>,
    lat: f64,
    lon: f64,
    ra: f64,
    dec: f64,
) -> Vec3 {
    // 1) Julian centuries since J2000.0
    let jd = julian_date(time);
    let t  = (jd - 2451545.0) / 36525.0;

    // 2) Greenwich Mean Sidereal Time in degrees
    //    (IAU 1982 expression)
    let gmst_deg = 280.46061837
        + 360.98564736629 * (jd - 2451545.0)
        + 0.000387933 * t*t
        - t*t*t / 38710000.0;
    // wrap to [0,360)
    let gmst_deg = gmst_deg.rem_euclid(360.0);
    let gmst_rad = gmst_deg.to_radians();    

    // 3) Local Sidereal Time (radians)
    //    add your longitude (east positive)
    let lst = (gmst_rad + lon).rem_euclid(2.0*PI);

    // 4) Hour Angle = LST − RA
    // 1) Hour Angle = LST − RA
    let ha = (lst - ra).rem_euclid(2.0 * PI);

    // 2) compute the horizon components:
    let east  = dec.cos() * ha.sin();                                // +X
    let north = dec.cos() * ha.cos() * lat.sin() - dec.sin() * lat.cos(); // +Z
    let up    = dec.cos() * ha.cos() * lat.cos() + dec.sin() * lat.sin(); // +Y

    // 3) reorder for Bevy: X=east, Y=up, Z=north
    Vec3::new(east  as f32,
              up    as f32,
              -north as f32)
        .normalize()
}


pub fn magnitude_to_scale(mag: f32) -> f32 {
    const MIN_MAG: f32 = -4.0;     // brightest in our clamp
    const MAX_MAG: f32 = 10.0;     // faintest
    const OUT_MIN: f32 = 30.0;     // smallest size
    const OUT_MAX: f32 = 10_000.0; // largest size

    // 1) clamp into [MIN_MAG, MAX_MAG]
    let m = mag.clamp(MIN_MAG, MAX_MAG);

    // 2) normalized fraction: 1 @ MIN_MAG → 0 @ MAX_MAG
    let t = (MAX_MAG - m) / (MAX_MAG - MIN_MAG);

    // 3) exponential interpolation
    OUT_MIN * (OUT_MAX / OUT_MIN).powf(t)

}

fn asset_base() -> PathBuf {
    let exe_path = env::current_exe().expect("failed to get current exe path");
    let exe_dir = exe_path
        .parent()
        .expect("executable must live in a directory");
    exe_dir.to_path_buf()
}

/// Spawns stars and a point light
fn spawn_starfield(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut rng = rand::thread_rng();
    let texture = asset_server.load("star.png");
    let quad = meshes.add(Mesh::from(Quad::new(Vec2::splat(1.0))));
    let path = asset_base().join("assets").join("BSC5");//PathBuf::from_str("./assets/BSC5").unwrap();

    let (hdr, stars) = parse_catalog(path).unwrap();

    let now = Utc::now();
    let lat = 40.7128_f64.to_radians();   // e.g. New York City
    let lon = (-74.0060_f64).to_radians(); // west long → negative

    for star in stars {
        
        let dir: Vec3 = star_direction(now, lat, lon, star.ra, star.dec).normalize();
        // direction & position
        // let dir = Vec3::new(
        //     rng.gen_range(-1.0..1.0),
        //     rng.gen_range(-1.0..1.0),
        //     rng.gen_range(-1.0..1.0),
        // )
        // .normalize();
        let position = dir * 100_000.0;

        let scale = magnitude_to_scale(star.magnitudes[0]);

        // 1) random blend factor
        let t: f32 = rng.gen_range(0.0..1.0);
        // 2) color endpoints in linear space
        let orange = Vec3::new(1.0, 0.8, 0.6);
        let blue   = Vec3::new(0.6, 0.8, 1.0);
        // 3) mix them
        let mix = orange.lerp(blue, t);
        // 4) make a Color and crank up brightness
        let star_color = Color::rgb_linear(mix.x, mix.y, mix.z) * 100.0;

        // 5) each star needs its own material
        let star_material = materials.add(StandardMaterial {
            base_color_texture: Some(texture.clone()),
            base_color:         star_color,
            emissive:           star_color,
            alpha_mode:         AlphaMode::Add,
            unlit:              true,
            ..default()
        });

        // 6) spawn!
        commands.spawn((
            PbrBundle {
                mesh:     quad.clone(),
                material: star_material,
                transform: Transform {
                    translation: position,
                    rotation:    Quat::IDENTITY,
                    scale:       Vec3::splat(scale),
                },
                ..default()
            },
            Star,
            StarDirection(dir),
        ));
    }
}

/// Moves each star to a far position along its unit direction
fn star_infinity_system(
    mut params: ParamSet<(
        Query<&Transform, With<Camera>>, // p0
        Query<(&StarDirection, &mut Transform)>, // p1
    )>,
) {
    let cam_pos = params.p0().single().translation;

    for (dir, mut tf) in params.p1().iter_mut() {
        tf.translation = cam_pos + dir.0 * 100_000.0;
    }
}

/// Aligns all quads to face the camera
fn billboard_system(
    cam_q: Query<&GlobalTransform, With<Camera>>,
    mut stars_q: Query<&mut Transform, With<Star>>,
) {
    let cam_rot = cam_q.single().compute_transform().rotation;
    for mut tf in stars_q.iter_mut() {
        tf.rotation = cam_rot;
    }
}
