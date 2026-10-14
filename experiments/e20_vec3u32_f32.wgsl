// vec3<u32> + f32: the trigger is not float-specific.
// expect: 4
struct S { v: vec3<u32>, a: f32 }
@group(0) @binding(0) var<uniform> u: S;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
fn get_a(s: S) -> f32 { return s.a; }
@compute @workgroup_size(1)
fn main() { output[0] = get_a(u); }
