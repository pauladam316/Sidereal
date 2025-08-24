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
        app.add_systems(Update, rescale_targets_system);
        app.add_event::<PlanetariumEvent>();
        app.add_systems(Update, handle_set_mount_position_events);
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
    let pos = ray.origin + ray.direction.normalize() * distance;

    let Ok(root_tf) = root_tf_q.single() else {
        return;
    };
    let root_rot = root_tf.compute_transform().rotation;
    let local_rot = root_rot.inverse() * cam_glob.compute_transform().rotation;

    if let Some((_, mut tf)) = q
        .iter_mut()
        .find(|(m, _)| matches!(m, Marker::TrackingTargetMarker))
    {
        tf.translation = pos;
        tf.rotation = local_rot;
    } else {
        let _ = spawn_tracking_target(
            &mut commands,
            &mut meshes,
            &mut mats,
            &assets,
            &root_entity_q,
            pos,
            local_rot,
        );
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

        let desired_pixels: f32 = 80.0;
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
    let color = Color::WHITE;
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

    let Ok(root_tf) = root_tf_q.single() else {
        return;
    };
    let root_rot = root_tf.compute_transform().rotation;
    let cam_rot = camera_q
        .single()
        .ok()
        .map(|g| g.compute_transform().rotation);
    let distance: f32 = 100.0;

    for evt in ev.read() {
        if let PlanetariumEvent::SetMountPosition { ra_hours, dec_deg } = *evt {
            let pos = radec_dir_from_hours(ra_hours, dec_deg).normalize() * distance;
            let rotation = cam_rot.map_or(Quat::IDENTITY, |c| root_rot.inverse() * c);

            // Look ONLY for the mount target
            if let Some((_, mut tf)) = q
                .iter_mut()
                .find(|(m, _)| matches!(m, Marker::MountTargetMarker))
            {
                tf.translation = pos;
                tf.rotation = rotation;
            } else {
                // Spawn exactly one mount target if missing
                let _ = spawn_mount_target(
                    &mut commands,
                    &mut meshes,
                    &mut mats,
                    &assets,
                    &root_entity_q,
                    pos,
                    rotation,
                );
            }
        }
    }
}
