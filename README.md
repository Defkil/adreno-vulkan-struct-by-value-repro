# Adreno Vulkan: wrong field reads from structs passed by value

A compute shader on **Adreno X1-85 / Windows Vulkan** reads a struct member
at byte offset **16 instead of 12** when a uniform or storage-buffer struct
is passed by value into a helper. The helper returns **5.0 instead of 4.0**;
a direct read of the same field in the same invocation returns 4.0.

A **56-line hand-written SPIR-V module** reproduces the failure without naga,
wgpu, or Bevy. All 13 committed SPIR-V modules pass the standalone validator.
The same WGSL passes on **DX12 on the affected Adreno**. On a second host,
an **AMD Radeon RX 7900 XTX passes all 100 comparison checks**. These
controls point to the Adreno Vulkan driver's shader compiler.

## Reproduce

Requires Rust (stable, 2024 edition) and a Vulkan driver. The default build
depends only on [`ash`](https://crates.io/crates/ash) and loads the
committed SPIR-V; no shader compiler is involved.

```text
git clone https://github.com/Defkil/adreno-vulkan-struct-by-value-repro.git
cd adreno-vulkan-struct-by-value-repro
cargo run --release --locked                 # six standard cases via raw Vulkan
cargo run --release --locked -- --spv shaders/by_value.spv
```

Exit code **1** means a mismatch; exit code **0** means every case matched.
The runner prints the selected GPU and driver. No window or game assets are
required. Optional paths:

```text
# WGSL through wgpu 30.0.1: Vulkan, and the DX12 control
cargo run --release --locked --features wgpu -- --wgpu shaders/by_value.wgsl
cargo run --release --locked --features wgpu -- --wgpu shaders/by_value.wgsl --dx12

# Any WGSL compiled by naga 30.0.1, run via raw Vulkan (used for experiments/)
cargo run --release --locked --features naga -- --wgsl experiments/e03_vec3_f32.wgsl
```

## Tested devices and results

Measurements dated **2026-10-14**. The Radeon comparison used unchanged source
at commit **c0e58657446649a622811d2a8b0d016f9507cc6b**.

| | Affected host | Comparison host |
| --- | --- | --- |
| Device | Surface Pro, 11th Edition / Snapdragon X Elite X1E80100 | Windows x64 desktop |
| GPU | Qualcomm Adreno X1-85 | AMD Radeon RX 7900 XTX |
| Windows display driver | 31.0.170.0 | 32.0.31041.1004 |
| Vulkan driver | Adreno compiler E031.52.01.00 | AMD 26.8.1 (LLPC) |
| Vulkan API | 1.4.295 | 1.4.349 |
| Windows | 11 Home 25H2, 26200.9457, ARM64 | 11 Pro 25H2, 26200.9550, x64 |
| Minimal by-value readback, Vulkan | **[5.0, 4.0] — incorrect** | [4.0, 4.0] — correct |
| WGSL through wgpu/DX12 | All 29 files pass | All 29 files pass |

The Radeon passed **all 13 committed SPIR-V modules** through raw Vulkan and
**all 29 WGSL files** (2 standard + 27 experiments) on each of three paths:
naga/raw Vulkan, wgpu/Vulkan, and wgpu/DX12. That is **100 GPU checks, no
mismatches**, including every case that failed on Adreno. Both minimal
by-value modules returned [4.0, 4.0].

- [Radeon validation report, exact environment, and commands](docs/validation/amd-radeon-2026-10-14.md)
- [All 100 Radeon readbacks](docs/validation/amd-radeon-2026-10-14.txt)

This confirms the standalone repro passes on that Radeon device/driver.
The original Bevy game was not tested on the comparison host, and other
GPU/driver combinations remain untested.

On **2026-10-19** the same source also ran on **Android with Qualcomm's
Vulkan driver on an Adreno 650** (Galaxy Tab S7, driver dated 10/07/21,
compiler EV031.32.02.10). All 71 checks (13 SPIR-V modules, 29 WGSL files
through naga and through wgpu) returned **exactly the X1-85 readbacks**,
failures included: the bug is not specific to the Windows driver or to the
X1-85.

- [Android Adreno 650 report, exact environment, and commands](docs/validation/android-adreno650-2026-10-19.md)
- [All 71 Android readbacks](docs/validation/android-adreno650-2026-10-19.txt)

## Expected and actual readback on Adreno

Every shader reads the same uniform buffer: 64 floats where float `i` holds
`i + 1` (so a value names the slot it was read from: byte offset =
`(value - 1) * 4`). The storage buffer is prefilled with `-1`.

| Shader (committed SPIR-V) | `out[0]` = helper result | `out[1]` = `S.a` read directly | Result |
| --- | --- | --- | --- |
| `shaders/by_value.spv` (WGSL via naga) | expected 4, **actual 5** | 4 | FAIL |
| `shaders/scalar.spv` (control) | 4 | 4 | pass |
| `shaders/handwritten_by_value.spv` (hand-written SPIR-V) | expected 4, **actual 5** | 4 | FAIL |
| `shaders/handwritten_scalar.spv` (control) | 4 | 4 | pass |
| `shaders/by_value_glsl.spv` (GLSL via glslang) | 4 | 4 | pass, see below |
| `shaders/scalar_glsl.spv` (GLSL control) | 4 | 4 | pass |
| `shaders/by_value.wgsl` through wgpu, **DX12** | 4 | 4 | pass |
| `shaders/by_value.wgsl` through wgpu, Vulkan | expected 4, **actual 5** | 4 | FAIL |

<details>
<summary>Full Adreno Vulkan output</summary>

Output of `cargo run --release` on Adreno X1-85, driver 31.0.170.0:

```text
Vulkan device : Qualcomm(R) Adreno(TM) X1-85 GPU (vendor 0x5143, device 0x36334330)
API version   : 1.4.295
driverVersion : 0x80376000 (raw)
driverName    : Qualcomm Technologies Inc. Adreno Vulkan Driver
driverInfo    : Driver Build: 0eb8636cd3, Id6ce193842, 1782814497
Date: 06/30/2026
Compiler Version: E031.52.01.00
Driver Branch: BVT_187
conformance   : 1.4.1.0

FAIL  vulkan(spirv) shaders\by_value.spv
      expected [4.0, 4.0]
      readback [5.0, 4.0]
PASS  vulkan(spirv) shaders\scalar.spv
      expected [4.0, 4.0]
      readback [4.0, 4.0]
FAIL  vulkan(spirv) shaders\handwritten_by_value.spv
      expected [4.0, 4.0]
      readback [5.0, 4.0]
PASS  vulkan(spirv) shaders\handwritten_scalar.spv
      expected [4.0, 4.0]
      readback [4.0, 4.0]
PASS  vulkan(spirv) shaders\by_value_glsl.spv
      expected [4.0, 4.0]
      readback [4.0, 4.0]
PASS  vulkan(spirv) shaders\scalar_glsl.spv
      expected [4.0, 4.0]
      readback [4.0, 4.0]

mismatching cases: ["by_value.spv", "handwritten_by_value.spv"]: miscompilation reproduced
```

</details>

## The minimal shader

[`shaders/by_value.wgsl`](shaders/by_value.wgsl):

```wgsl
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
    output[0] = get_a(u); // returns 5 (offset 16) on Adreno Vulkan, must be 4
    output[1] = u.a;      // returns 4
}
```

The control [`shaders/scalar.wgsl`](shaders/scalar.wgsl) differs only in
`fn get_a(a: f32) -> f32 { return a; }` called as `get_a(u.a)`.

<details>
<summary>SPIR-V analysis, failure conditions, and all 34 experiment results</summary>

The same as hand-written SPIR-V 1.0,
[`shaders/handwritten_by_value.spvasm`](shaders/handwritten_by_value.spvasm)
(no front end, no wrapper struct), essential part:

```text
OpMemberDecorate %S 0 Offset 0
OpMemberDecorate %S 1 Offset 12
OpDecorate %S Block
%S = OpTypeStruct %v3float %float
%u = OpVariable %ptr_u_S Uniform
%get_a = OpFunction %float None %fn_get      ; (S) -> float
%s = OpFunctionParameter %S
  %a = OpCompositeExtract %float %s 1        ; returns the float at offset 16
...
%v = OpLoad %S %u
%r = OpFunctionCall %float %get_a %v
```

### What triggers it

Found by the variants in [experiments/](experiments), all on Adreno X1-85
with driver 31.0.170.0, Vulkan. Every WGSL variant passes through DX12:

- An `OpLoad` of a whole explicitly laid-out struct from a `Uniform` or
  `StorageBuffer` variable whose SSA value reaches an `OpFunctionCall`
  argument (directly, through `OpCopyObject`, through a nested call, or
  passed into and returned back out of the helper), and
- the struct has a `vec3` followed by a member that starts inside the
  `vec3`'s 16-byte slot (12 bytes after the `vec3`).
- The member immediately after a `vec3` is then read 4 bytes too far (byte
  offset 16 instead of 12 for a `vec3` at offset 0, 32 instead of 28 in
  `e10`; later members shift as well, see `e02`). In `e12_two_vec3`
  (`v: vec3 @0, a: f32 @12, w: vec3 @16, b: f32 @28`), `a`, `w.x`, and `b`
  are read from byte offsets 16, 32, and 48 instead of 12, 16, and 28
  (`5, 9, 13` instead of `4, 5, 8`).
  The observed values match a layout where every `vec3` occupies a full
  16-byte slot—pushing `a` to byte 16, aligning `w: vec3` to byte 32, and
  pushing `b` to byte 48—i.e. as if the `Offset` decorations were replaced by
  a `vec4`-sized `vec3` layout for the struct value passed into the call
  (that is an interpretation of the readback, not knowledge of the driver
  internals). The wrong read can leave the struct: for the 16-byte struct
  above it reads byte 16.
- Not relevant: struct size (16, 32 and 96 bytes all fail), the 96-byte
  `mat4x4` tail, the element type of the `vec3` (`vec3<u32>` fails too),
  the function control (`None`, `Inline`, `DontInline` all fail), naga's
  block wrapper struct (the hand-written module without it fails).
- Not affected: the same load used in the same function (`OpLoad` +
  `OpCompositeExtract` without a call, `e06`/`h06`), a struct copied into a
  `Function` variable first and then either loaded and passed by value (`e09`)
  or passed as `ptr<function, S>` (`e08`), a struct in `Workgroup` (`e18`) or
  `Private` (`e19`) storage, a struct built with `OpCompositeConstruct` from
  member loads (`e16`/`h04`) or unpacked in the caller and re-packed into an
  undecorated parameter struct (`h05`), a struct whose post-`vec3` member is
  at offset 16 (`@align(16)`, `e21`), structs without `vec3` (`vec2`, `vec4`,
  scalars only), and a 0-argument helper that loads `u` and returns `S`
  without taking `S` as a parameter (tested ad hoc; no committed file).

GLSL: glslang (15.3.0 via shaderc) does not produce the failing pattern for
the equivalent GLSL ([`shaders/by_value_glsl.comp`](shaders/by_value_glsl.comp)).
Because GLSL parameters are mutable local variables in `Function` storage
class, glslang declares a second, undecorated struct type for the parameter,
copies the loaded block member by member into a `Function` variable and
passes a pointer (see `shaders/by_value_glsl.spvasm`). That matches the
"copied into a `Function` variable" cases (`e08`, `e09`, `h05`), which pass.
The hand-written SPIR-V shows the fault does not depend on naga.

### Variants tried

`Expected` is the correct readback; bold entries in `Vulkan readback` mark
mismatches. Run through naga + raw Vulkan (`--wgsl`) and wgpu Vulkan
(`--wgpu`): same results; wgpu DX12 (`--wgpu --dx12`): all pass.

| File | Variant | Expected | Vulkan readback |
| --- | --- | --- | --- |
| `e01_atmosphere_struct_return_field` | Bevy's 96-byte Atmosphere layout (vec3, f32, f32, mat4x4), helpers return inner/outer | 4 5 | **5 6** |
| `e02_vec3_f32_f32` | vec3, f32, f32 | 4 5 | **5 6** |
| `e03_vec3_f32` | vec3, f32 (16-byte struct) | 4 | **5** |
| `e04_scalars_only` | five f32, no vector | 4 5 | 4 5 |
| `e05_vec4_f32_f32` | vec4, f32, f32 | 5 6 | 5 6 |
| `e06_no_helper_whole_load` | `let s = u;` read members in `main`, no call | 4 5 | 4 5 |
| `e07_vec3_members` | helper returns the vec3 and the f32 | 1 2 3 4 | **1 2 3 5** |
| `e08_function_pointer` | copy to `var`, pass `ptr<function, S>` | 4 5 | 4 5 |
| `e09_var_copy_by_value` | copy to `var` in `main`, pass that by value | 4 5 | 4 5 |
| `e10_f32_vec3_f32` | f32 @0, vec3 @16, f32 @28 | 1 5 6 7 8 | **1 5 6 7 9** |
| `e11_vec2_f32_f32` | vec2, f32, f32 | 3 4 | 3 4 |
| `e12_two_vec3` | vec3 @0, f32 @12, vec3 @16, f32 @28 | 4 5 8 | **5 9 13** |
| `e13_scalar_control` | members passed as scalars | 4 5 | 4 5 |
| `e14_vec3_f32_only_helper_returns_a` | vec3, f32, f32; one helper call | 4 | **5** |
| `e15_nested_struct_member` | inner struct (vec3, f32) member of the uniform passed | 4 | **5** |
| `e16_constructed_struct` | struct constructed in the shader from uniform scalars | 4 5 | 4 5 |
| `e17_storage_buffer_struct` | struct in a storage buffer (written by the shader first) | 4 5 | **5 -1** |
| `e18_workgroup_struct` | struct in workgroup memory | 4 5 | 4 5 |
| `e19_private_struct` | struct in a private global | 4 5 | 4 5 |
| `e20_vec3u32_f32` | vec3&lt;u32&gt;, f32 | 4 | **5** |
| `e21_vec3_aligned_member` | vec3, `@align(16)` f32 (offset 16) | 5 | 5 |
| `e22_helper_uses_vec3_and_a` | helper uses the vec3 and the f32 | 4 | **5** |
| `e23_two_args` | helper `(s: S, x: f32)` like `clamp_to_surface` | 4 | **5** |
| `e24_nested_call` | struct passed through two helper levels | 4 | **5** |
| `e25_return_struct` | pass-through `fn(s: S) -> S`, caller reads `.a` | 4 | **5** |
| `e26_array_of_vec3_struct` | uniform `array<S, 2>`, elements passed | 4 8 | **5 9** |
| `e27_bevy_clamp_to_surface_shape` | the shape of Bevy's `clamp_to_surface` on the 96-byte layout | 5 | **6** |
| `spirv/h01_wrapped_load_call` | hand-written, naga's shape (Block wrapper) | 4 4 | **5 4** |
| `spirv/h02_block_struct_load_call` | hand-written, struct itself is the Block | 4 4 | **5 4** |
| `spirv/h03_copyobject_before_call` | `OpCopyObject` between load and call | 4 4 | **5 4** |
| `spirv/h04_construct_from_member_loads` | parameter is laid-out `S`; argument built by `OpCompositeConstruct` from member loads | 4 4 | 4 4 |
| `spirv/h05_separate_param_type` | `OpLoad S` unpacked in `main`, re-packed into undecorated `S2` parameter | 4 4 | 4 4 |
| `spirv/h06_load_extract_no_call` | load + extract in `main`, no call | 4 4 | 4 4 |
| `spirv/h07_scalar_param` | scalar parameter | 4 4 | 4 4 |
| e03 with helper `FunctionControl` `Inline` / `DontInline` (patched binary) | | 4 | **5** / **5** |

</details>

## Controls

- **Scalar argument vs struct by value**: the scalar-argument twin of every
  failing shader passes (`scalar.*`, `handwritten_scalar.*`, `e13`, `h07`).
- **Same-dispatch direct read**: each failing shader in `shaders/` and
  `experiments/spirv/` also reads `S.a` directly in `main` (`out[1]`) in the
  same compute invocation, returning `4.0` while `get_a(u)` in `out[0]`
  returns `5.0`.
- **Whole-struct `OpLoad` vs `OpFunctionCall` boundary**:
  - `OpLoad %S` from `Uniform` followed by `OpCompositeExtract` in `main`
    without a call passes (`e06`, `h06`).
  - Calling the exact same `%get_a(%S)` helper (taking the `Offset`-decorated
    `%S` by value) passes when the `%S` argument is loaded from a local
    `Function` variable (`e09`) or constructed with `OpCompositeConstruct`
    from member loads (`h04`, `e16`).
- **No shader front end**: a 56-line hand-written SPIR-V 1.0 module
  (`shaders/handwritten_by_value.spvasm`) without naga's block wrapper
  reproduces the exact same `5.0` vs `4.0` mismatch.
- **Vulkan vs DX12 on the same GPU**: every WGSL shader (including all 27
  variants in `experiments/`) passes through DX12 (`--wgpu --dx12`).
- **SPIR-V validity**: every committed SPIR-V module passes `spirv-val`
  (SPIRV-Tools v2026.4 and v2025.3) for Vulkan 1.0 and Vulkan 1.3.
- **Second host**: all 100 GPU checks pass on AMD Radeon RX 7900 XTX;
  see the comparison and readback links above.
- **Second Qualcomm driver**: Android's Adreno 650 Vulkan driver reproduces
  every X1-85 readback (71 checks); see the Android report linked above.

## Workarounds

In WGSL or SPIR-V targeting this driver:

1. **Pass individual scalar or vector fields** to helper functions instead of
   passing the uniform/storage struct by value (`shaders/scalar.wgsl`, `e13`,
   `h07`).
2. **Copy the struct into a function-local variable first** (`var s = u;
   get_a(s);` as in `e09`, or pass `&s` as `ptr<function, S>` as in `e08`) so
   the call argument is loaded from `Function` storage class rather than
   directly from `Uniform` or `StorageBuffer`.
3. **Avoid packing a member at byte offset 12 after a `vec3`** when you
   control the buffer layout (for example, annotate the following member with
   `@align(16)` as in `e21`, or pack `vec3<f32>` + `f32` into a `vec4<f32>`
   as in `e05`). Update the host-side buffer layout to match.

## Original impact

Found in Bevy 0.20's atmosphere shaders: the helper
`clamp_to_surface(atmosphere: Atmosphere, position)` read `outer_radius`
instead of `inner_radius`, clamping a ground + 2 m observer to 6,460,001 m
instead of 6,360,002 m. The upward distance became 0 and transmittance
lookup coordinates became NaN. Passing the needed radii as scalar arguments
works around the problem.

## Adreno environment

<details>
<summary>Full affected-device and toolchain details</summary>

Gathered on 2026-10-14 with `Get-CimInstance`, the Windows registry and the
adapter info printed by this program.

| | |
| --- | --- |
| Device | Microsoft Surface Pro, 11th Edition (SKU `Surface_Pro_11th_Edition_2076`), UEFI 175.222.235 |
| SoC | Snapdragon X Elite X1E80100 (`Snapdragon(R) X 12-core X1E80100 @ 3.40 GHz`) |
| GPU | Qualcomm(R) Adreno(TM) X1-85 GPU, Vulkan vendor `0x5143`, device `0x36334330` |
| Graphics driver | 31.0.170.0 (Windows driver date 2026-07-25) |
| Vulkan `driverName` | Qualcomm Technologies Inc. Adreno Vulkan Driver |
| Vulkan `driverInfo` | `Driver Build: 0eb8636cd3, Id6ce193842, 1782814497` / `Date: 06/30/2026` / `Compiler Version: E031.52.01.00` / `Driver Branch: BVT_187` |
| Vulkan `driverVersion` (raw) | `0x80376000` |
| Vulkan API version | 1.4.295, conformance version 1.4.1.0 |
| Vulkan loader | `vulkan-1.dll` 1.4.350.0 |
| OS | Windows 11 Home, version 25H2, build 26200.9457, ARM64 |
| Rust | rustc 1.98.1 (48a229cea 2026-09-01) and rustc 1.96.0 (ac68faa20 2026-05-25), host `aarch64-pc-windows-msvc` |
| Crates | ash 0.38.0+1.3.281; naga 30.0.1; rspirv 0.13.0+sdk-1.4.341.0; wgpu 30.0.1; pollster 0.4.0; tools: shaderc 0.10.1 (glslang 15.3.0), spirv-tools 0.13.3 (SPIRV-Tools v2025.3, used for `as`/`dis`/`validate`); standalone validation: `spirv-val` v2026.4 |

The original Bevy case also failed on the earlier driver 31.0.137.0 (Vulkan
`Driver Build: f6d77cea75, I4ecc8b8813`, 2025-11-20, compiler
`E031.50.31.01`); this reduced repro was only run on 31.0.170.0.

</details>

## Regenerating and validating shaders

<details>
<summary>Compiler tools, regeneration commands, and standalone SPIR-V validation</summary>

### Regenerating and validating the SPIR-V

The naga output (SPIR-V 1.0, naga 30.0.1 default writer options; the
disassembly is rspirv 0.13.0's):

```text
cargo run --release --features naga -- --emit shaders/by_value.wgsl   # writes .spv and .spvasm
cargo run --release --features naga -- --emit shaders/scalar.wgsl
```

GLSL compilation, hand-written assembly, disassembly, and in-tree validation
use the separate [`tools/shader-tools`](tools/shader-tools) crate (builds
glslang 15.3.0 through shaderc 0.10.1 and SPIRV-Tools v2025.3 through
spirv-tools 0.13.3 from source; needs CMake, Ninja, Python and a C++
compiler):

```text
cd tools/shader-tools
cargo run --release -- glsl ../../shaders/by_value_glsl.comp      # Vulkan 1.0, SPIR-V 1.0, -O0
cargo run --release -- glsl ../../shaders/scalar_glsl.comp
cargo run --release -- dis ../../shaders/by_value_glsl.spv
cargo run --release -- dis ../../shaders/scalar_glsl.spv
cargo run --release -- as ../../shaders/handwritten_by_value.spvasm
cargo run --release -- as ../../shaders/handwritten_scalar.spvasm
cargo run --release -- validate ../../shaders/by_value.spv ../../shaders/scalar.spv ../../shaders/handwritten_by_value.spv ../../shaders/handwritten_scalar.spv ../../shaders/by_value_glsl.spv ../../shaders/scalar_glsl.spv
```

All 13 committed modules (`shaders/*.spv` and `experiments/spirv/*.spv`)
were also validated on 2026-10-14 with the official standalone validator,
built from source: KhronosGroup/SPIRV-Tools tag `v2026.4` (commit `ef96ed7`)
with KhronosGroup/SPIRV-Headers at `4965431` (the revision pinned by that
tag's `DEPS`), CMake Release build. `spirv-val --version` prints
`SPIRV-Tools v2026.4 v2026.4-0-gef96ed7`.

```text
spirv-val --target-env vulkan1.0 <file.spv>
spirv-val --target-env vulkan1.3 <file.spv>
```

Every module passes for both target environments (exit code 0, no
diagnostics). The original Bevy module was checked earlier with SPIRV-Tools
2026.1 `spirv-val --target-env vulkan1.0`.

The regeneration is deterministic: re-emitting the WGSL, GLSL, or `.spvasm`
files produces the committed bytes.

</details>

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option. `experiments/e27` reproduces the
five-line shape of Bevy's `clamp_to_surface` (Bevy is MIT OR Apache-2.0).
