// The struct lives in a storage buffer (written first), then passed by value.
// expect: 4 5
struct S { v: vec3<f32>, a: f32, b: f32 }
struct B { out: array<f32, 4>, s: S }
@group(0) @binding(0) var<uniform> u: array<vec4<f32>, 4>;
@group(0) @binding(1) var<storage, read_write> buf: B;
fn get_a(s: S) -> f32 { return s.a; }
fn get_b(s: S) -> f32 { return s.b; }
@compute @workgroup_size(1)
fn main() {
    buf.s.v = u[0].xyz; buf.s.a = u[0].w; buf.s.b = u[1].x;
    storageBarrier();
    let a = get_a(buf.s); let b = get_b(buf.s);
    buf.out[0] = a; buf.out[1] = b;
}
