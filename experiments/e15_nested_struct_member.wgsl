// Pass a nested struct member (inner: vec3+f32) of a bigger uniform by value.
// expect: 4
struct Inner { v: vec3<f32>, a: f32 }
struct S { inner: Inner, pad: vec4<f32> }
@group(0) @binding(0) var<uniform> u: S;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
fn get_a(s: Inner) -> f32 { return s.a; }
@compute @workgroup_size(1)
fn main() { output[0] = get_a(u.inner); }
