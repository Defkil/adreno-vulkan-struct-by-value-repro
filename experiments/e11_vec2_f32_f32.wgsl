// vec2 + f32 + f32 (no vec3).
// expect: 3 4
struct S { v: vec2<f32>, a: f32, b: f32 }
@group(0) @binding(0) var<uniform> u: S;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
fn get_a(s: S) -> f32 { return s.a; }
fn get_b(s: S) -> f32 { return s.b; }
@compute @workgroup_size(1)
fn main() { output[0] = get_a(u); output[1] = get_b(u); }
