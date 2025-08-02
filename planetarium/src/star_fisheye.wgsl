// star_fisheye.wgsl
struct VertexInput {
    @location(0) position: vec3<f32>,
    @builtin(instance_index) instance: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> view_proj: mat4x4<f32>;

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Transform world-space position to view-space
    let view_pos = (view_proj * vec4<f32>(input.position, 1.0)).xyz;

    // Normalize view vector
    let dir = normalize(view_pos);

    // Fisheye projection: Azimuthal Equidistant
    let r = acos(dir.z) / 3.1415926; // radial distance (0 to 1)
    let theta = atan2(dir.y, dir.x);

    let projected = vec2<f32>(cos(theta), sin(theta)) * r;

    // Convert to NDC (clip space)
    output.clip_position = vec4<f32>(projected, 0.0, 1.0);
    output.world_position = input.position;

    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0); // white star
}
