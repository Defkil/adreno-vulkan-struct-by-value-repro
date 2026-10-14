// Control, PASSES: identical except that the helper takes the member as a scalar.
// expect: 4 4
struct S {
    v: vec3<f32>, // offset 0, size 12
    a: f32,       // offset 12
}

@group(0) @binding(0) var<uniform> u: S;
@group(0) @binding(1) var<storage, read_write> output: array<f32, 16>;

fn get_a(a: f32) -> f32 {
    return a;
}

@compute @workgroup_size(1)
fn main() {
    output[0] = get_a(u.a);
    output[1] = u.a;
}
