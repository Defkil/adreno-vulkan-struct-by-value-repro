// Helper returns vec3 components and the following scalar.
// expect: 1 2 3 4
struct S { v: vec3<f32>, a: f32 }
@group(0) @binding(0) var<uniform> u: S;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
fn get_v(s: S) -> vec3<f32> { return s.v; }
fn get_a(s: S) -> f32 { return s.a; }
@compute @workgroup_size(1)
fn main() { let v = get_v(u); output[0] = v.x; output[1] = v.y; output[2] = v.z; output[3] = get_a(u); }
