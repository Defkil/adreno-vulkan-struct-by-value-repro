// Control: the same struct, but the helper takes the field as a scalar.
// expect: 4 5
struct S { v: vec3<f32>, a: f32, b: f32 }
@group(0) @binding(0) var<uniform> u: S;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
fn id(x: f32) -> f32 { return x; }
@compute @workgroup_size(1)
fn main() { output[0] = id(u.a); output[1] = id(u.b); }
