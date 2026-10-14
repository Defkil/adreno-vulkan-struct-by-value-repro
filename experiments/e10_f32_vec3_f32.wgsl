// f32, vec3 (offset 16), f32 (offset 28).
// expect: 1 5 6 7 8
struct S { a: f32, v: vec3<f32>, b: f32 }
@group(0) @binding(0) var<uniform> u: S;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
fn get_a(s: S) -> f32 { return s.a; }
fn get_v(s: S) -> vec3<f32> { return s.v; }
fn get_b(s: S) -> f32 { return s.b; }
@compute @workgroup_size(1)
fn main() { output[0] = get_a(u); let v = get_v(u); output[1] = v.x; output[2] = v.y; output[3] = v.z; output[4] = get_b(u); }
