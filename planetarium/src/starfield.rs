// src/starfield.rs

use bevy::{ecs::query::QuerySingleError, prelude::*, render::mesh::shape::Quad};
use chrono::{DateTime, Utc};
use rand::Rng;
use std::{f64::consts::PI, path::PathBuf, time::Instant};

use crate::star_catalog::parse_catalog;

/// Marker on the root entity
#[derive(Component)]
pub struct StarfieldRoot;

/// Per‐star data so we can recompute positions on location change
#[derive(Component)]
pub struct StarData {
    pub ra: f64,
    pub dec: f64,
    pub magnitude: f32,
}

/// Events you can send at ANY TIME to jump the sky:
pub struct SetLocationEvent {
    /// degrees north positive
    pub lat_deg: f64,
    /// degrees east positive
    pub lon_deg: f64,
}
pub struct SetTimeEvent {
    pub time: DateTime<Utc>,
}

#[derive(Resource)]
pub struct StarfieldState {
    /// When we first spawned (the RA/Dec→horizon positions were for this UTC)
    pub spawn_utc: DateTime<Utc>,

    /// Our last “time override” instant, for smooth rotation updates
    pub base_instant: Instant,
    /// How far (radians) we’ve already rotated at base_instant
    pub base_angle: f32,

    /// Observer latitude & longitude (degrees)
    pub lat_deg: f64,
    pub lon_deg: f64,

    /// Rotation axis in local horizon coords (unit Vec3)
    pub axis: Vec3,
    /// Sidereal rate: 2π radians per 86 164.0905 s
    pub rate: f32,
}

impl Default for StarfieldState {
    fn default() -> Self {
        // will be overwritten in spawn_starfield()
        StarfieldState {
            spawn_utc: Utc::now(),
            base_instant: Instant::now(),
            base_angle: 0.0,
            lat_deg: 0.0,
            lon_deg: 0.0,
            axis: Vec3::Y,
            rate: (2.0 * PI as f32) / 86_164.0905_f32,
        }
    }
}

pub struct StarfieldPlugin;
impl Plugin for StarfieldPlugin {
    fn build(&self, app: &mut App) {
        app
            // register our events
            .add_event::<SetLocationEvent>()
            .add_event::<SetTimeEvent>()
            // initial spawn
            .add_startup_system(spawn_starfield)
            // handle any runtime jumps
            .add_system(handle_set_location_events)
            .add_system(handle_set_time_events)
            // per-frame updates
            .add_system(rotate_starfield_system.before(billboard_system))
            .add_system(starfield_follow_camera.before(rotate_starfield_system))
            .add_system(billboard_system);
    }
}

/// Compute the Julian Date (JD) from a UTC time.
fn julian_date(time: DateTime<Utc>) -> f64 {
    let unix = time.timestamp() as f64;
    let sub = time.timestamp_subsec_nanos() as f64 * 1e-9;
    2440587.5 + (unix + sub) / 86400.0
}

/// RA/Dec → local‐horizon unit vector (X=east, Y=up, Z=north)
fn star_direction(time: DateTime<Utc>, lat: f64, lon: f64, ra: f64, dec: f64) -> Vec3 {
    let jd = julian_date(time);
    let t = (jd - 2451545.0) / 36525.0;
    let gmst = (280.46061837 + 360.98564736629 * (jd - 2451545.0) + 0.000387933 * t * t
        - t * t * t / 38710000.0)
        .rem_euclid(360.0)
        .to_radians();
    let lst = (gmst + lon).rem_euclid(2.0 * PI);
    let ha = (lst - ra).rem_euclid(2.0 * PI);

    let east = dec.cos() * ha.sin();
    let north = dec.cos() * ha.cos() * lat.sin() - dec.sin() * lat.cos();
    let up = dec.cos() * ha.cos() * lat.cos() + dec.sin() * lat.sin();

    // Bevy: X=east, Y=up, Z=–north  (so “forward” is towards the sky)
    Vec3::new(east as f32, up as f32, -north as f32).normalize()
}

/// Map magnitude → quad scale
pub fn magnitude_to_scale(mag: f32) -> f32 {
    const MIN_MAG: f32 = -4.0;
    const MAX_MAG: f32 = 10.0;
    const OUT_MIN: f32 = 30.0;
    const OUT_MAX: f32 = 10_000.0;

    let m = mag.clamp(MIN_MAG, MAX_MAG);
    let t = (MAX_MAG - m) / (MAX_MAG - MIN_MAG);
    OUT_MIN * (OUT_MAX / OUT_MIN).powf(t)
}

/// Where your executable’s `assets/BSC5` folder lives
fn asset_base() -> PathBuf {
    let exe = std::env::current_exe().expect("no exe path");
    exe.parent().unwrap().to_path_buf()
}

/// Spawn root + all stars at their **spawn** positions
fn spawn_starfield(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    // load catalog
    let path = asset_base().join("assets").join("BSC5");
    let (_hdr, stars) = parse_catalog(path).unwrap();

    // observer defaults (NYC)
    let now = Utc::now();
    let lat = 40.7128_f64;
    let lon = -74.0060_f64;

    // build our StarfieldState
    let axis = {
        let lr = lat.to_radians();
        Vec3::new(0.0, lr.sin() as f32, lr.cos() as f32)
    };
    let rate = (2.0 * PI as f32) / 86_164.0905_f32;

    commands.insert_resource(StarfieldState {
        spawn_utc: now,
        base_instant: Instant::now(),
        base_angle: 0.0,
        lat_deg: lat,
        lon_deg: lon,
        axis,
        rate,
    });

    // prepare quad & texture
    let quad = meshes.add(Mesh::from(Quad::new(Vec2::splat(1.0))));
    let texture = assets.load("star.png");

    // spawn a single root
    let root = commands
        .spawn((SpatialBundle::default(), StarfieldRoot))
        .id();

    // now spawn each star as its child
    let mut rng = rand::thread_rng();
    for star in stars {
        let dir = star_direction(now, lat.to_radians(), lon.to_radians(), star.ra, star.dec);
        let pos = dir * 100_000.0;
        let scale = magnitude_to_scale(star.magnitudes[0]);
        let t: f32 = rng.gen();
        let mix = Vec3::new(1.0, 0.8, 0.6).lerp(Vec3::new(0.6, 0.8, 1.0), t);
        let color = Color::rgb_linear(mix.x, mix.y, mix.z) * 100.0;

        let mat = mats.add(StandardMaterial {
            base_color_texture: Some(texture.clone()),
            base_color: color,
            emissive: color,
            alpha_mode: AlphaMode::Add,
            unlit: true,
            ..default()
        });

        commands.entity(root).with_children(|p| {
            p.spawn((
                PbrBundle {
                    mesh: quad.clone(),
                    material: mat,
                    transform: Transform {
                        translation: pos,
                        scale: Vec3::splat(scale),
                        ..Default::default()
                    },
                    ..default()
                },
                StarData {
                    ra: star.ra,
                    dec: star.dec,
                    magnitude: star.magnitudes[0],
                },
            ));
        });
    }
}

/// When you send a SetLocationEvent, recompute `axis` **and** every star’s base position
fn handle_set_location_events(
    mut ev: EventReader<SetLocationEvent>,
    mut state: ResMut<StarfieldState>,
    mut q: Query<(&StarData, &mut Transform), Without<Camera>>,
) {
    for SetLocationEvent { lat_deg, lon_deg } in ev.iter() {
        // update state
        state.lat_deg = *lat_deg;
        state.lon_deg = *lon_deg;
        let lr = lat_deg.to_radians();
        state.axis = Vec3::new(0.0, lr.sin() as f32, lr.cos() as f32);

        // recompute every star’s initial translation
        for (data, mut tf) in q.iter_mut() {
            let dir = star_direction(
                state.spawn_utc,
                state.lat_deg.to_radians(),
                state.lon_deg.to_radians(),
                data.ra,
                data.dec,
            );
            tf.translation = dir * 100_000.0;
        }
    }
}

/// When you send a SetTimeEvent, jump the rotation to that UTC
fn handle_set_time_events(mut ev: EventReader<SetTimeEvent>, mut state: ResMut<StarfieldState>) {
    for SetTimeEvent { time } in ev.iter() {
        // how many seconds since spawn?
        let delta_s = (time
            .signed_duration_since(state.spawn_utc)
            .num_milliseconds() as f32)
            * 1e-3;
        // set base_angle so that angle = rate * delta_s
        state.base_angle = state.rate * delta_s;
        state.base_instant = Instant::now();
    }
}

/// Each frame: rotate the root by (base_angle + rate * elapsed_since_base)
pub fn rotate_starfield_system(
    time: Res<Time>,
    state: Res<StarfieldState>,
    mut q: Query<&mut Transform, With<StarfieldRoot>>,
) {
    let elapsed = state.base_instant.elapsed().as_secs_f32();
    let angle = state.base_angle + state.rate * elapsed;
    let mut tf = q.single_mut();
    tf.rotation = Quat::from_axis_angle(state.axis, -angle);
}

/// Keep the root positioned at the camera
fn starfield_follow_camera(
    cam_q: Query<&GlobalTransform, With<Camera>>,
    mut star_q: Query<&mut Transform, With<StarfieldRoot>>,
) {
    // Try to grab exactly one camera; if it’s not there yet, just return.
    let cam_tf = match cam_q.get_single() {
        Ok(tf) => tf,
        Err(_) => return, // no camera spawned yet
    };

    // Now propagate its position to all StarfieldRoot entities:
    for mut star_tf in star_q.iter_mut() {
        star_tf.translation = cam_tf.translation();
    }
}
/// Billboarding: make every star quad face the camera
fn billboard_system(
    cam_q: Query<&GlobalTransform, With<Camera>>,
    mut stars: Query<&mut Transform, With<StarData>>,
) {
    let cam_tf = match cam_q.get_single() {
        Ok(tf) => tf,
        Err(_) => return, // no camera spawned yet
    };

    let cam_rot = cam_tf.compute_transform().rotation;
    for mut tf in stars.iter_mut() {
        tf.rotation = cam_rot;
    }
}
