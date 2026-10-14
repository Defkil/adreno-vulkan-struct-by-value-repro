// The real Bevy 0.20 Atmosphere layout (96 bytes); helper returns a field.
// expect: 4 5
struct Atmosphere {
    ground_albedo: vec3<f32>,
    inner_radius: f32,
    outer_radius: f32,
    world_to_atmosphere: mat4x4<f32>,
}
@group(0) @binding(0) var<uniform> u: Atmosphere;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
fn get_inner(a: Atmosphere) -> f32 { return a.inner_radius; }
fn get_outer(a: Atmosphere) -> f32 { return a.outer_radius; }
@compute @workgroup_size(1)
fn main() {
    output[0] = get_inner(u);
    output[1] = get_outer(u);
}
