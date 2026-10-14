// vec3 followed by a member that does not share its 16-byte slot
// (explicit @align(16) puts a at offset 16).
// expect: 5
struct S { v: vec3<f32>, @align(16) a: f32 }
@group(0) @binding(0) var<uniform> u: S;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
fn get_a(s: S) -> f32 { return s.a; }
@compute @workgroup_size(1)
fn main() { output[0] = get_a(u); }
