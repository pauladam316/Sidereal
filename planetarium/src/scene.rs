use bevy::asset::RenderAssetUsages;
use bevy::{
    mesh::{Mesh, PrimitiveTopology},
    prelude::*,
};
use meshtext::{MeshGenerator, MeshText, TextSection};

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ground)
            .add_systems(Update, billboard_labels);
    }
}

#[derive(Component)]
struct GroundLabel;

fn spawn_ground(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 1) Ground plane
    let plane = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(100_000.0)));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.4, 0.1),
        perceptual_roughness: 1.0,
        unlit: true,
        ..default()
    });

    commands.spawn((
        Mesh3d(plane),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, -5.0, 0.0),
        Visibility::default(),
    ));

    // 2) Directional light
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 100.0, 0.0))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
        Visibility::default(),
    ));

    // 3) Prepare the meshtext generator
    let font_data = include_bytes!("../assets/SwanseaBoldItalic-p3Dv.ttf");
    let mut generator = MeshGenerator::new(font_data);

    // scale for text size
    let text_scale = 25.0_f32;
    let transform_array = Mat4::from_scale(Vec3::splat(text_scale)).to_cols_array();

    // 4) Cardinal markers: (label, position)
    let height = 10.0; // slightly above the plane
    let dist = 2000.0; // radius
    let markers = [
        ("N", Vec3::new(0.0, height, dist)),
        ("S", Vec3::new(0.0, height, -dist)),
        ("E", Vec3::new(-dist, height, 0.0)),
        ("W", Vec3::new(dist, height, 0.0)),
    ];

    for (label, pos) in markers.iter() {
        // generate a MeshText for this single character
        let text_mesh: MeshText = generator
            .generate_section(
                &label.to_string(),
                /* centered */ true,
                Some(&transform_array),
            )
            .unwrap();

        // extract vertex positions & UVs
        let vertices = text_mesh.vertices;
        let positions: Vec<[f32; 3]> = vertices
            .chunks(3)
            .map(|chunk| [chunk[0], chunk[1], chunk[2]])
            .collect();
        let uvs = vec![[0.0, 0.0]; positions.len()];

        // build a Bevy mesh
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::RENDER_WORLD, // or MAIN_WORLD | RENDER_WORLD if you also need CPU-side access
        );

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        // mesh.insert_indices(Indices::U32(indices));
        mesh.compute_flat_normals();

        // add it to assets and spawn
        let mesh_handle = meshes.add(mesh);
        let mat_handle = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        });

        commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(mat_handle),
            Transform::from_translation(*pos),
            Visibility::default(),
            GroundLabel,
        ));
    }
}

/// Rotate each GroundLabel around Y so its local +Z axis points at the camera.
fn billboard_labels(
    // only query Transforms that do *not* have GroundLabel
    cam_q: Query<&Transform, (With<Camera3d>, Without<GroundLabel>)>,
    mut q: Query<&mut Transform, With<GroundLabel>>,
) {
    let cam_tf = match cam_q.single() {
        Ok(tf) => tf,
        Err(_) => return,
    };
    let cam_pos = cam_tf.translation;

    for mut tf in q.iter_mut() {
        // direction from label → camera, flatten to XZ plane
        let mut dir = cam_pos - tf.translation;
        dir.y = 0.0;
        if dir.length_squared() < 1e-6 {
            continue;
        }
        let dir = dir.normalize();

        // find yaw so that Vec3::Z rotated by yaw → dir
        // i.e. sin(yaw)=dir.x, cos(yaw)=dir.z
        let yaw = dir.x.atan2(dir.z);

        tf.rotation = Quat::from_rotation_y(yaw);
    }
}
