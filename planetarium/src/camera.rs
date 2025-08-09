use bevy::input::mouse::{AccumulatedMouseMotion, MouseWheel};
use bevy::prelude::*;

#[derive(Component)]
pub struct RotatingCamera {
    pub yaw: f32,
    pub pitch: f32,
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, (camera_rotation_system, camera_zoom_system));
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            fov: 60.0_f32.to_radians(),
            ..default()
        }),
        Transform::IDENTITY, // at origin
        RotatingCamera {
            yaw: 0.0,
            pitch: 0.0,
        },
    ));
}

// Rotate camera in place (origin) on drag
pub fn camera_rotation_system(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut q: Query<(&mut RotatingCamera, &mut Transform), With<Camera3d>>,
) {
    if !mouse_buttons.pressed(MouseButton::Left) {
        return;
    }

    let (mut rc, mut t) = q.single_mut().unwrap();
    let d = accumulated_mouse_motion.delta;
    if d == Vec2::ZERO {
        return;
    }

    // FPS-style: move mouse right → look right; move up → look up
    rc.yaw += d.x * 0.003;
    rc.pitch += d.y * 0.003;
    rc.pitch = rc.pitch.clamp(-1.54, 1.54);

    // Apply rotation (yaw around Y, then pitch around X)
    t.rotation = Quat::from_euler(EulerRot::YXZ, rc.yaw, rc.pitch, 0.0);
}

// Mouse wheel → FOV zoom (perspective only)
pub fn camera_zoom_system(
    mut wheel: EventReader<MouseWheel>,
    mut proj: Query<&mut Projection, With<Camera3d>>,
) {
    let mut projection = proj.single_mut().unwrap();
    if let Projection::Perspective(ref mut p) = *projection {
        for ev in wheel.read() {
            p.fov = (p.fov - ev.y * 0.05).clamp(0.1, std::f32::consts::PI - 0.01);
        }
    }
}
