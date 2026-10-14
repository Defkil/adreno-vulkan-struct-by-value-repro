// Helper returns the struct by value; caller reads the member.
// expect: 4
struct S { v: vec3<f32>, a: f32 }
@group(0) @binding(0) var<uniform> u: S;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
fn pass_through(s: S) -> S { return s; }
@compute @workgroup_size(1)
fn main() { output[0] = pass_through(u).a; }
