use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

#[derive(Component)]
pub struct RotatingCamera {
    pub yaw: f32,
    pub pitch: f32,
}

/// Stores the world-space direction that was under the cursor when the drag began.
#[derive(Component, Default)]
pub struct PanAnchor(pub Option<Vec3>);

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
        PanAnchor::default(), // <-- add this
    ));
}
fn signed_angle_around_axis(u: Vec3, v: Vec3, axis: Vec3) -> f32 {
    let ua = u - axis * u.dot(axis);
    let va = v - axis * v.dot(axis);
    let ua_n = ua.try_normalize().unwrap_or(Vec3::ZERO);
    let va_n = va.try_normalize().unwrap_or(Vec3::ZERO);
    if ua_n == Vec3::ZERO || va_n == Vec3::ZERO {
        return 0.0;
    }
    let sin = axis.dot(ua_n.cross(va_n));
    let cos = ua_n.dot(va_n);
    sin.atan2(cos)
}
// Wrap angle to [-PI, PI]
fn wrap_pi(a: f32) -> f32 {
    let mut x = (a + std::f32::consts::PI) % (2.0 * std::f32::consts::PI);
    if x < 0.0 {
        x += 2.0 * std::f32::consts::PI;
    }
    x - std::f32::consts::PI
}

pub fn camera_rotation_system(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<
        (
            &mut RotatingCamera,
            &mut Transform,
            &Projection,
            &Camera,
            &GlobalTransform,
            &mut PanAnchor,
        ),
        With<Camera3d>,
    >,
) {
    let (mut rc, mut t, projection, camera, gtf, mut anchor) = if let Ok(v) = q.single_mut() {
        v
    } else {
        return;
    };
    let window = if let Ok(w) = windows.single() {
        w
    } else {
        return;
    };

    // Capture world direction under cursor on press (exact)
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(p) = window.cursor_position() {
            if let Ok(ray) = camera.viewport_to_world(gtf, p) {
                anchor.0 = Some(ray.direction.normalize());
            }
        }
    }
    // Stop anchoring on release
    if mouse_buttons.just_released(MouseButton::Left) {
        anchor.0 = None;
        return;
    }
    if !mouse_buttons.pressed(MouseButton::Left) {
        return;
    }
    let target = if let Some(d) = anchor.0 {
        d.normalize()
    } else {
        return;
    };

    // Perspective only
    let (vfov, aspect) = match &*projection {
        Projection::Perspective(p) => (p.fov, p.aspect_ratio),
        _ => return,
    };

    // Cursor -> camera-local ray for THIS pixel (independent of current rotation)
    let cursor = window
        .cursor_position()
        .unwrap_or(Vec2::new(window.width() * 0.5, window.height() * 0.5));
    let x_ndc = (cursor.x / window.width()) * 2.0 - 1.0;
    let y_ndc = 1.0 - (cursor.y / window.height()) * 2.0; // +Y up
    let tan_v = (vfov * 0.5).tan();
    let tan_h = tan_v * aspect;
    let dir_cam = Vec3::new(x_ndc * tan_h, y_ndc * tan_v, -1.0).normalize();

    // Current world ray for that pixel using current yaw/pitch
    let rot_current = Quat::from_euler(EulerRot::YXZ, rc.yaw, rc.pitch, 0.0);
    let w_cur = (rot_current * dir_cam).normalize();

    // If already aligned, do nothing (prevents any drift when holding still)
    if w_cur.dot(target) > 0.999_999 {
        return;
    }

    // --- 1) Yaw delta (around +Y): match azimuths in world XZ plane ---
    let az_cur = w_cur.x.atan2(-w_cur.z);
    let az_tgt = target.x.atan2(-target.z);
    let yaw_delta = -1.0 * wrap_pi(az_tgt - az_cur);
    rc.yaw = wrap_pi(rc.yaw + yaw_delta);

    // --- 2) Pitch delta (around camera RIGHT after yaw) ---
    let rot_after_yaw = Quat::from_euler(EulerRot::YXZ, rc.yaw, rc.pitch, 0.0);
    let right = (rot_after_yaw * Vec3::X).normalize();
    let w_after_yaw = (rot_after_yaw * dir_cam).normalize();

    // Signed angle from current ray to target around the right axis
    let u = w_after_yaw - right * w_after_yaw.dot(right);
    let v = target - right * target.dot(right);
    let u_n = u.try_normalize().unwrap_or(Vec3::ZERO);
    let v_n = v.try_normalize().unwrap_or(Vec3::ZERO);
    if u_n != Vec3::ZERO && v_n != Vec3::ZERO {
        let sin = right.dot(u_n.cross(v_n));
        let cos = u_n.dot(v_n);
        let pitch_delta = sin.atan2(cos);
        rc.pitch = (rc.pitch + pitch_delta).clamp(-1.54, 1.54);
    }

    // Apply (no roll)
    t.rotation = Quat::from_euler(EulerRot::YXZ, rc.yaw, rc.pitch, 0.0);
}
pub fn camera_zoom_system(
    mut wheel: EventReader<MouseWheel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<
        (
            &mut Projection,
            &mut RotatingCamera,
            &mut Transform,
            &Camera,
            &GlobalTransform,
        ),
        With<Camera3d>,
    >,
) {
    let (mut projection, mut rc, mut t, camera, gtf) = if let Ok(v) = q.single_mut() {
        v
    } else {
        return;
    };
    let window = if let Ok(w) = windows.single() {
        w
    } else {
        return;
    };

    // Only handle perspective cameras.
    let Projection::Perspective(ref mut persp) = *projection else {
        return;
    };

    for ev in wheel.read() {
        // Cursor pos (use center if not available)
        let cursor = window
            .cursor_position()
            .unwrap_or(Vec2::new(window.width() * 0.5, window.height() * 0.5));

        // --- 1) Old world ray through the cursor (pre-zoom) -> target_dir ---
        let target_dir = match camera.viewport_to_world(gtf, cursor) {
            Ok(ray) => ray.direction.normalize(),
            Err(_) => {
                // Rare fallback: manual using current FOV
                let w = window.width().max(1.0);
                let h = window.height().max(1.0);
                let x_ndc = (cursor.x / w) * 2.0 - 1.0;
                let y_ndc = 1.0 - (cursor.y / h) * 2.0;
                let tan_v = (persp.fov * 0.5).tan();
                let dir_cam =
                    Vec3::new(x_ndc * tan_v * persp.aspect_ratio, y_ndc * tan_v, -1.0).normalize();
                (t.rotation * dir_cam).normalize()
            }
        };

        // --- 2) Change FOV (zoom) ---
        let new_fov = (persp.fov - ev.y * 0.05).clamp(0.1, std::f32::consts::PI - 0.01);
        persp.fov = new_fov;

        // We want the point under the cursor to stay fixed on screen BOTH ways (in/out),
        // so we always solve the rotation correction.

        // --- 3) Build the *camera-local* cursor ray for the NEW FOV ---
        let w = window.width().max(1.0);
        let h = window.height().max(1.0);
        let x_ndc = (cursor.x / w) * 2.0 - 1.0;
        let y_ndc = 1.0 - (cursor.y / h) * 2.0;
        let tan_v_new = (new_fov * 0.5).tan();
        let tan_h_new = tan_v_new * persp.aspect_ratio;
        let dir_cam_new = Vec3::new(x_ndc * tan_h_new, y_ndc * tan_v_new, -1.0).normalize();

        // Current world ray for that new-FOV cursor direction
        let rot = Quat::from_euler(EulerRot::YXZ, rc.yaw, rc.pitch, 0.0);
        let w_cur = (rot * dir_cam_new).normalize();

        // --- 5) Yaw correction: match azimuth (XZ-plane) of w_cur -> target_dir ---
        // Signed angle in XZ-plane around +Y
        let f = Vec2::new(w_cur.x, w_cur.z);
        let d = Vec2::new(target_dir.x, target_dir.z);
        let f_n = f.normalize_or_zero();
        let d_n = d.normalize_or_zero();

        let yaw_delta = if f_n == Vec2::ZERO || d_n == Vec2::ZERO {
            0.0
        } else {
            let cross_y = f_n.x * d_n.y - f_n.y * d_n.x;
            let dot = f_n.dot(d_n);
            cross_y.atan2(dot)
        };
        rc.yaw -= yaw_delta;

        // --- 6) Pitch correction: rotate around camera RIGHT to match elevation ---
        let rot_after_yaw = Quat::from_euler(EulerRot::YXZ, rc.yaw, rc.pitch, 0.0);
        let right = (rot_after_yaw * Vec3::X).normalize();
        let w_after_yaw = (rot_after_yaw * dir_cam_new).normalize();

        let pitch_delta = signed_angle_around_axis(w_after_yaw, target_dir, right);
        rc.pitch = (rc.pitch + pitch_delta).clamp(-1.54, 1.54);

        // --- 7) Apply final rotation (no roll) ---
        t.rotation = Quat::from_euler(EulerRot::YXZ, rc.yaw, rc.pitch, 0.0);
    }
}
