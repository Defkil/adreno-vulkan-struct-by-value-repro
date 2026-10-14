// No helper: load the whole struct into a let in main, then read members.
// expect: 4 5
struct S { v: vec3<f32>, a: f32, b: f32 }
@group(0) @binding(0) var<uniform> u: S;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
@compute @workgroup_size(1)
fn main() { let s = u; output[0] = s.a; output[1] = s.b; }
