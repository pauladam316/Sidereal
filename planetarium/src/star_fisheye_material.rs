use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "e6e155a4-8896-4ddf-8a4d-4047cdbfbe9c"]
pub struct FisheyeStarMaterial {}

impl Material for FisheyeStarMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/star_fisheye.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/star_fisheye.wgsl".into()
    }
}
