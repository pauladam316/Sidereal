use crate::starfield::StarfieldRoot;
use bevy::prelude::*;
use bevy::render::camera::Projection;

/// Marker on each spawned target
#[derive(Component)]
pub struct TargetMarker;

#[derive(Component)]
pub struct TargetLabel; // marker for the text child

pub struct TargetPlugin;

impl Plugin for TargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, place_target_on_right_click);
        app.add_systems(Update, rescale_targets_system);
    }
}

fn spawn_new_target(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
    root_entity_q: Query<Entity, With<StarfieldRoot>>,
    position: Vec3,
    rotation: Quat,
) {
    let quad_handle = meshes.add(Mesh::from(Rectangle::new(1.0, 1.0)));
    let tex = assets.load("target.png");
    let color = Color::WHITE;
    let scale = 1.5;
    let mat = mats.add(StandardMaterial {
        base_color_texture: Some(tex.clone()),
        base_color: color,
        emissive: color.into(),
        alpha_mode: AlphaMode::Add,
        unlit: true,
        // cull_mode: None, // optionally see both sides
        ..default()
    });

    if let Ok(root) = root_entity_q.single() {
        commands.entity(root).with_children(|parent| {
            parent.spawn((
                Mesh3d(quad_handle.clone()),
                MeshMaterial3d(mat.clone()),
                Transform {
                    translation: position,
                    rotation,
                    scale: Vec3::splat(scale),
                    ..Default::default()
                },
                Visibility::default(),
                TargetMarker,
            ));
        });
    }
}

pub fn place_target_on_right_click(
    commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    meshes: ResMut<Assets<Mesh>>,
    mats: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
    root_entity_q: Query<Entity, With<StarfieldRoot>>,
    root_tf_q: Query<&GlobalTransform, With<StarfieldRoot>>,
    mut target_q: Query<&mut Transform, With<TargetMarker>>,
) {
    // only on right‐click
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }

    let (camera, cam_glob) = if let Ok(pair) = camera_q.single() {
        pair
    } else {
        return;
    };
    let window = windows.single().unwrap();
    let cursor_pos = if let Some(pos) = window.cursor_position() {
        pos
    } else {
        return;
    };
    let ray = if let Ok(r) = camera.viewport_to_world(cam_glob, cursor_pos) {
        r
    } else {
        return;
    };

    let distance = 100.0;
    let origin = ray.origin;
    let dir = ray.direction.normalize();
    let pos = origin + dir * distance;

    let cam_rot = cam_glob.compute_transform().rotation;
    let root_tf = if let Ok(tf) = root_tf_q.single() {
        tf
    } else {
        return;
    };
    let root_rot = root_tf.compute_transform().rotation;
    let local_rot = root_rot.inverse() * cam_rot;

    if !target_q.is_empty() {
        for mut transform in &mut target_q {
            transform.translation = pos;
            transform.rotation = local_rot;
        }
        return;
    } else {
        spawn_new_target(
            commands,
            meshes,
            mats,
            assets,
            root_entity_q,
            pos,
            local_rot,
        );
    }
}

fn rescale_targets_system(
    windows: Query<&Window>,
    camera_q: Query<(&GlobalTransform, &Projection), With<Camera>>,
    mut target_q: Query<&mut Transform, With<TargetMarker>>,
) {
    let window = windows.single().unwrap();
    let (cam_glob, projection) = camera_q.single().unwrap();

    for mut transform in &mut target_q {
        let distance = (cam_glob.translation() - transform.translation).length();

        let desired_pixels: f32 = 40.0; // desired on-screen height
        let quad_world_height = 1.0; // size of the quad mesh in world units
        let scale = if let Projection::Perspective(p) = projection {
            let fov_y = p.fov;
            let pixels_per_world_at_d = window.height() / (2.0 * distance * (fov_y * 0.5).tan());
            desired_pixels / (pixels_per_world_at_d * quad_world_height)
        } else {
            // default scale if it's not perspective
            1.0
        };

        transform.scale = Vec3::splat(scale);
    }
}
