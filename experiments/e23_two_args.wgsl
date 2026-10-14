// Helper takes the struct and a scalar (like clamp_to_surface(atmosphere, position)).
// expect: 4
struct S { v: vec3<f32>, a: f32 }
@group(0) @binding(0) var<uniform> u: S;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
fn f(s: S, x: f32) -> f32 { return s.a + x; }
@compute @workgroup_size(1)
fn main() { output[0] = f(u, 0.0); }
