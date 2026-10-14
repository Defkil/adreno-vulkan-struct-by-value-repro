// Helper uses the vec3 and the scalar together (like clamp_to_surface).
// expect: 4
struct S { v: vec3<f32>, a: f32 }
@group(0) @binding(0) var<uniform> u: S;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
fn f(s: S) -> f32 { return s.a + 0.0 * s.v.x; }
@compute @workgroup_size(1)
fn main() { output[0] = f(u); }
