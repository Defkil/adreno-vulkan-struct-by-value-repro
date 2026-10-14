// five f32 (no vector); helper returns members 3 and 4.
// expect: 4 5
struct S { x: f32, y: f32, z: f32, a: f32, b: f32 }
@group(0) @binding(0) var<uniform> u: S;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
fn get_a(s: S) -> f32 { return s.a; }
fn get_b(s: S) -> f32 { return s.b; }
@compute @workgroup_size(1)
fn main() { output[0] = get_a(u); output[1] = get_b(u); }
