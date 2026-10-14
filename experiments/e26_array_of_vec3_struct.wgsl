// Uniform array element (struct with vec3 + f32) passed by value.
// expect: 4 8
struct S { v: vec3<f32>, a: f32 }
@group(0) @binding(0) var<uniform> u: array<S, 2>;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
fn get_a(s: S) -> f32 { return s.a; }
@compute @workgroup_size(1)
fn main() { output[0] = get_a(u[0]); output[1] = get_a(u[1]); }
