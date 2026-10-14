// Struct built in the shader from uniform scalars, then passed by value.
// expect: 4 5
struct S { v: vec3<f32>, a: f32, b: f32 }
struct U { x: vec4<f32>, y: vec4<f32> }
@group(0) @binding(0) var<uniform> u: U;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
fn get_a(s: S) -> f32 { return s.a; }
fn get_b(s: S) -> f32 { return s.b; }
@compute @workgroup_size(1)
fn main() { let s = S(u.x.xyz, u.x.w, u.y.x); output[0] = get_a(s); output[1] = get_b(s); }
