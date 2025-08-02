use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};

#[derive(Component)]
pub struct RotatingCamera {
    pub yaw: f32,
    pub pitch: f32,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup_camera)
            .add_system(camera_rotation_system)
            .add_system(camera_zoom_system)
            .add_system(update_camera_transform_system);
    }
}

/// Setup the camera with default position and rotation tracking
pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::default(),
            ..default()
        },
        RotatingCamera { yaw: 0.0, pitch: 0.0 },
    ));
}

/// Mouse input to rotate the camera
pub fn camera_rotation_system(
    mut motion_evr: EventReader<MouseMotion>,
    mouse_button: Res<Input<MouseButton>>,
    mut query: Query<&mut RotatingCamera>,
) {
    if mouse_button.pressed(MouseButton::Left) {
        let mut camera = query.single_mut();
        for ev in motion_evr.iter() {
            camera.yaw += ev.delta.x * 0.003;
            camera.pitch = (camera.pitch + ev.delta.y * 0.003).clamp(-1.54, 1.54);
        }
    }
}

/// Scroll to zoom the camera by modifying FOV
pub fn camera_zoom_system(
    mut scroll_evr: EventReader<MouseWheel>,
    mut query: Query<&mut Projection, With<Camera>>,
) {
    for mut projection in query.iter_mut() {
        if let Projection::Perspective(ref mut perspective) = *projection {
            for ev in scroll_evr.iter() {
                perspective.fov = (perspective.fov - ev.y * 0.05)
                    .clamp(0.1, std::f32::consts::PI - 0.01);
            }
        }
    }
}

/// Apply yaw/pitch rotation to camera transform
pub fn update_camera_transform_system(
    mut query: Query<(&RotatingCamera, &mut Transform)>
) {
    let (camera, mut transform) = query.single_mut();
    transform.translation = Vec3::ZERO;

    let yaw = Quat::from_rotation_y(camera.yaw);
    let pitch = Quat::from_rotation_x(camera.pitch);
    transform.rotation = yaw * pitch;
}
