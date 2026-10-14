// FAILS on Adreno X1-85 / Vulkan: the struct is passed by value to a helper.
// Uniform floats hold 1, 2, 3, ... so s.a (byte offset 12) must read 4.
// output[0] = helper result, output[1] = the same member read directly.
// expect: 4 4
struct S {
    v: vec3<f32>, // offset 0, size 12
    a: f32,       // offset 12 (packed into the vec3's 16-byte slot)
}

@group(0) @binding(0) var<uniform> u: S;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;

fn get_a(s: S) -> f32 {
    return s.a;
}

@compute @workgroup_size(1)
fn main() {
    output[0] = get_a(u);
    output[1] = u.a;
}
