// The struct lives in a private global, then passed by value.
// expect: 4 5
struct S { v: vec3<f32>, a: f32, b: f32 }
@group(0) @binding(0) var<uniform> u: array<vec4<f32>, 4>;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;
var<private> p: S;
fn get_a(s: S) -> f32 { return s.a; }
fn get_b(s: S) -> f32 { return s.b; }
@compute @workgroup_size(1)
fn main() {
    p.v = u[0].xyz; p.a = u[0].w; p.b = u[1].x;
    output[0] = get_a(p); output[1] = get_b(p);
}
