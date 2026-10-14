// Struct passed through two helper levels.
// expect: 4
struct S { v: vec3<f32>, a: f32 }
@group(0) @binding(0) var<uniform> u: S;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
fn inner(s: S) -> f32 { return s.a; }
fn outer(s: S) -> f32 { return inner(s); }
@compute @workgroup_size(1)
fn main() { output[0] = outer(u); }
