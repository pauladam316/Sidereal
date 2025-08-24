use crate::events::PlanetariumEvent;
use crate::starfield::StarfieldRoot;
use bevy::prelude::*;
use bevy::render::camera::Projection;

#[derive(Component)]
pub struct TargetLabel; // marker for the text child

pub struct TargetPlugin;
#[derive(Component)]
pub enum Marker {
    TrackingTargetMarker,
    MountTargetMarker,
}
impl Plugin for TargetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, place_target_on_right_click);
        app.add_systems(PostUpdate, rescale_targets_system);
        app.add_event::<PlanetariumEvent>();
        app.add_systems(Update, handle_set_mount_position_events);
        app.add_systems(PostUpdate, orient_targets_to_camera);
    }
}

pub fn place_target_on_right_click(
    mut commands: Commands,
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
    root_entity_q: Query<Entity, With<StarfieldRoot>>,
    root_tf_q: Query<&GlobalTransform, With<StarfieldRoot>>,
    mut q: Query<(&Marker, &mut Transform)>,
) {
    if !buttons.just_pressed(MouseButton::Right) {
        return;
    }

    let Ok((camera, cam_glob)) = camera_q.single() else {
        return;
    };
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(ray) = camera.viewport_to_world(cam_glob, cursor_pos) else {
        return;
    };

    let distance = 100.0;
    let pos_world = ray.origin + ray.direction.normalize() * distance;

    let Ok(root_gtf) = root_tf_q.single() else {
        return;
    };
    let world_to_root = root_gtf.affine().inverse(); // Affine3A
    let pos_local = world_to_root.transform_point3(pos_world);

    // rotation already computed relative to root – keep it
    let root_rot = root_gtf.compute_transform().rotation;
    let local_rot = root_rot.inverse() * cam_glob.compute_transform().rotation;

    if let Some((_, mut tf)) = q
        .iter_mut()
        .find(|(m, _)| matches!(m, Marker::TrackingTargetMarker))
    {
        tf.translation = pos_local;
        tf.rotation = local_rot;
    } else {
        let _ = spawn_tracking_target(
            &mut commands,
            &mut meshes,
            &mut mats,
            &assets,
            &root_entity_q,
            pos_local,
            local_rot,
        );
    }
}

/// Face all Marker targets toward the camera, with up aligned to the camera's up (no roll).
pub fn orient_targets_to_camera(
    camera_q: Query<&GlobalTransform, With<Camera>>,
    root_q: Query<&GlobalTransform, With<StarfieldRoot>>,
    mut targets_q: Query<(&mut Transform, &GlobalTransform), With<Marker>>,
) {
    let Ok(cam_gtf) = camera_q.single() else {
        return;
    };
    let cam_pos = cam_gtf.translation();
    let cam_rot = cam_gtf.compute_transform().rotation;
    let cam_up = (cam_rot * Vec3::Y).normalize(); // camera's up in world

    // Parent (StarfieldRoot) rotation, to convert world rotation -> child's local rotation
    let Ok(root_gtf) = root_q.single() else {
        return;
    };
    let root_rot_world = root_gtf.compute_transform().rotation;

    for (mut local_tf, gtf) in &mut targets_q {
        let target_world_pos = gtf.translation();

        let to_cam = cam_pos - target_world_pos;
        if to_cam.length_squared() < 1e-12 {
            continue; // camera is exactly on top; undefined facing
        }

        // Desired world basis:
        //  - forward (Z) points *toward* the camera, so the quad faces the camera
        //  - up matches the camera's up (no roll)
        let forward = to_cam.normalize();

        // If camera up is nearly parallel to forward, pick an alternate up to avoid degeneracy
        let mut up_ref = cam_up;
        if up_ref.dot(forward).abs() > 0.999 {
            // choose a vector guaranteed not parallel to forward
            up_ref = Vec3::X;
            if up_ref.dot(forward).abs() > 0.999 {
                up_ref = Vec3::Z;
            }
        }

        // Right-handed orthonormal basis
        let right = up_ref.cross(forward).normalize();
        let up = forward.cross(right).normalize();

        // World rotation whose columns are (right, up, forward)
        let world_rot = Quat::from_mat3(&Mat3::from_cols(right, up, forward));

        // Convert to local (child) rotation under StarfieldRoot
        let local_rot = root_rot_world.inverse() * world_rot;

        local_tf.rotation = local_rot;
    }
}

fn rescale_targets_system(
    windows: Query<&Window>,
    camera_q: Query<(&GlobalTransform, &Projection), With<Camera>>,
    mut target_q: Query<&mut Transform, With<Marker>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((cam_glob, projection)) = camera_q.single() else {
        return;
    };

    for mut transform in &mut target_q {
        let distance = (cam_glob.translation() - transform.translation).length();

        let desired_pixels: f32 = 60.0;
        let quad_world_height: f32 = 1.0;

        let scale = if let Projection::Perspective(p) = projection {
            let fov_y = p.fov;
            let pixels_per_world_at_d = window.height() / (2.0 * distance * (fov_y * 0.5).tan());
            desired_pixels / (pixels_per_world_at_d * quad_world_height)
        } else {
            1.0
        };

        transform.scale = Vec3::splat(scale);
    }
}

pub fn spawn_target_with_marker(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
    assets: &AssetServer,
    root_entity_q: &Query<Entity, With<StarfieldRoot>>,
    position: Vec3,
    rotation: Quat,
    scale: f32,
    marker: Marker,
) -> Option<Entity> {
    let quad = meshes.add(Mesh::from(Rectangle::new(1.0, 1.0)));
    let tex = match marker {
        Marker::MountTargetMarker => assets.load("target_inner.png"),
        Marker::TrackingTargetMarker => assets.load("target_outer.png"),
    };
    let color = match marker {
        Marker::TrackingTargetMarker => Color::LinearRgba(LinearRgba {
            red: 0.918,
            green: 0.878,
            blue: 0.349,
            alpha: 1.0,
        }),
        Marker::MountTargetMarker => Color::LinearRgba(LinearRgba {
            red: 0.475,
            green: 0.941,
            blue: 0.475,
            alpha: 1.0,
        }),
    };
    let mat = mats.add(StandardMaterial {
        base_color_texture: Some(tex),
        base_color: color,
        emissive: color.into(),
        alpha_mode: AlphaMode::Add,
        unlit: true,
        ..default()
    });

    let root = root_entity_q.single().ok()?;

    let child = commands
        .spawn((
            Mesh3d(quad),
            MeshMaterial3d(mat),
            Transform {
                translation: position,
                rotation,
                scale: Vec3::splat(scale),
                ..Default::default()
            },
            Visibility::default(),
            marker, // enum component
        ))
        .id();

    commands.entity(root).add_child(child);
    Some(child)
}
pub fn spawn_mount_target(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
    assets: &AssetServer,
    root_entity_q: &Query<Entity, With<StarfieldRoot>>,
    position: Vec3,
    rotation: Quat,
) -> Option<Entity> {
    spawn_target_with_marker(
        commands,
        meshes,
        mats,
        assets,
        root_entity_q,
        position,
        rotation,
        1.5,
        Marker::MountTargetMarker,
    )
}

pub fn spawn_tracking_target(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
    assets: &AssetServer,
    root_entity_q: &Query<Entity, With<StarfieldRoot>>,
    position: Vec3,
    rotation: Quat,
) -> Option<Entity> {
    spawn_target_with_marker(
        commands,
        meshes,
        mats,
        assets,
        root_entity_q,
        position,
        rotation,
        1.5,
        Marker::TrackingTargetMarker,
    )
}

pub fn handle_set_mount_position_events(
    mut commands: Commands,
    mut ev: EventReader<PlanetariumEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
    root_entity_q: Query<Entity, With<StarfieldRoot>>,
    root_tf_q: Query<&GlobalTransform, With<StarfieldRoot>>,
    mut q: Query<(&Marker, &mut Transform)>,
    camera_q: Query<&GlobalTransform, With<Camera>>,
) {
    #[inline]
    fn radec_dir_from_hours(ra_hours: f32, dec_deg: f32) -> Vec3 {
        let ra = (ra_hours * 15.0).to_radians();
        let dec = dec_deg.to_radians();
        Vec3::new(dec.cos() * ra.cos(), dec.sin(), dec.cos() * ra.sin())
    }

    // 1) Read only the last SetMountPosition of this frame
    let mut last: Option<(f32, f32)> = None;
    for evt in ev.read() {
        if let PlanetariumEvent::SetMountPosition { ra_hours, dec_deg } = *evt {
            last = Some((ra_hours, dec_deg));
        }
    }
    let Some((ra_hours, dec_deg)) = last else {
        return;
    };

    // 2) Compute local position/rotation under the rotating StarfieldRoot
    let Ok(root_gtf) = root_tf_q.single() else {
        return;
    };
    let world_to_root = root_gtf.affine().inverse();
    let root_rot = root_gtf.compute_transform().rotation;

    let cam_rot = camera_q
        .single()
        .ok()
        .map(|g| g.compute_transform().rotation);

    let distance = 100.0;
    let dir_world = radec_dir_from_hours(ra_hours, dec_deg).normalize();
    let pos_world = dir_world * distance;
    let pos_local = world_to_root.transform_point3(pos_world);
    let rot_local = cam_rot.map_or(Quat::IDENTITY, |c| root_rot.inverse() * c);

    // 3) Move existing mount target if present; otherwise spawn exactly one
    if let Some((_, mut tf)) = q
        .iter_mut()
        .find(|(m, _)| matches!(*m, &Marker::MountTargetMarker))
    // <-- fix
    {
        tf.translation = pos_local;
        tf.rotation = rot_local;
    } else {
        let _ = spawn_mount_target(
            &mut commands,
            &mut meshes,
            &mut mats,
            &assets,
            &root_entity_q,
            pos_local,
            rot_local,
        );
    }
}
