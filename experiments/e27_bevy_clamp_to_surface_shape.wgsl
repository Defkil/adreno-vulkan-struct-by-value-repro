// Shape of the original case: Bevy 0.20 atmosphere `clamp_to_surface`
// (Bevy is MIT OR Apache-2.0) on the real 96-byte Atmosphere layout.
// inner_radius reads 4, so min_radius = 5 and a zero position is pushed to
// radius 5. With the bug inner_radius reads the outer radius (5) -> 6.
// expect: 5
struct Atmosphere {
    ground_albedo: vec3<f32>,
    inner_radius: f32,
    outer_radius: f32,
    world_to_atmosphere: mat4x4<f32>,
}
@group(0) @binding(0) var<uniform> atmosphere: Atmosphere;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
const EPSILON: f32 = 1.0;
fn clamp_to_surface(atmosphere: Atmosphere, position: vec3<f32>) -> vec3<f32> {
    let min_radius = atmosphere.inner_radius + EPSILON;
    let r = length(position);
    if r < min_radius {
        let up = normalize(position);
        return up * min_radius;
    }
    return position;
}
@compute @workgroup_size(1)
fn main() {
    output[0] = length(clamp_to_surface(atmosphere, vec3(0.0, 1.0, 0.0)));
}
