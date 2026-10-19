# Android Adreno 650 validation, 2026-10-19

The miscompilation **reproduces** with Qualcomm's Android Vulkan driver on an
Adreno 650 (Samsung Galaxy Tab S7 LTE, Snapdragon 865+). All 71 GPU checks
returned exactly the readbacks recorded for the Adreno X1-85 on Windows:
every case that fails there fails here with the same wrong values, and every
case that passes there passes here.

## Source and environment

- Repository source: `e83ce687aa113010cb201db9b95cf7882452a8b5`, unchanged.
  No source or shader changes.
- Device: Samsung SM-T875 (Galaxy Tab S7 LTE), board `kona`
  (Snapdragon 865/865+).
- GPU: Adreno (TM) 650, Vulkan vendor `0x5143`, device `0x06050002`.
- Vulkan: API `1.1.128`, driver `Qualcomm Technologies Inc. Adreno Vulkan
  Driver`, raw version `0x801f6000`, conformance `1.2.0.1`.
- Driver info: `Driver Build: 193b2ee, I593c16c433, 1633593732` /
  `Date: 10/07/21` / `Compiler Version: EV031.32.02.10` /
  `Driver Branch: Test-lib-SR05432264`.
- OS: Android 13, build `TP1A.220624.014.T875XXS8DXH1`, security patch
  2024-08-01, kernel `4.19.113-27114284`.
- Build: cross-compiled on Linux x86_64 for `aarch64-linux-android`
  (API level 30) with rustc `1.99.1 (58cf8f9c3 2026-10-13)` and Android NDK
  r27d; repository Cargo.lock used with `--locked`.
- Crates: ash `0.38.0+1.3.281`, naga `30.0.1`, wgpu `30.0.1`.

For comparison, the Windows X1-85 driver reports `Compiler Version:
E031.52.01.00` dated 06/30/2026. The two drivers are about five years apart and
belong to different Adreno architecture families (6xx vs X1).

## Results

| Path | Cases | Passed | Failed |
|---|---:|---:|---:|
| Committed `shaders/*.spv`, raw Vulkan | 6 | 4 | 2 |
| Committed `experiments/spirv/*.spv`, raw Vulkan | 7 | 4 | 3 |
| WGSL compiled by naga, raw Vulkan | 29 | 12 | 17 |
| WGSL through wgpu, Vulkan | 29 | 12 | 17 |
| Total | 71 | 32 | 39 |

A script compared each of the 71 readbacks with the Adreno X1-85 readbacks in
the README (the variants table and the six standard cases). There were no
differences. Examples:

- `by_value.spv` and `handwritten_by_value.spv`: `[5.0, 4.0]`, expected
  `[4.0, 4.0]`.
- `e12_two_vec3`: `[5.0, 9.0, 13.0]`, expected `[4.0, 5.0, 8.0]`.
- Controls pass: `scalar*`, GLSL (`by_value_glsl`, `scalar_glsl`), `e06`,
  `e08`, `e09`, `e16`, `e18`, `e19`, `e21`, `h04` to `h07`.

Full adapter output and readbacks:
[android-adreno650-2026-10-19.txt](android-adreno650-2026-10-19.txt).

## Not covered

- DX12 does not exist on Android, so there is no same-GPU second API.
- The patched `Inline`/`DontInline` binaries of `e03` are not committed and
  were not run.
- Linux with Mesa's Turnip driver was not tested. WSL2 was not used because its
  Vulkan driver (Mesa dzn) runs on top of D3D12 rather than Qualcomm's Vulkan
  compiler.
- The original Bevy game was not run on this device.

## Commands

```bash
# Build host (Linux x86_64)
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=<ndk>/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android30-clang
cargo build --release --locked --features naga,wgpu --target aarch64-linux-android

# Copy the binary to /data/local/tmp/repro/, every shaders/* and
# experiments/spirv/* file to /data/local/tmp/repro/spv/, and every
# shaders/*.wgsl and experiments/*.wgsl file to /data/local/tmp/repro/wgsl/:
adb push target/aarch64-linux-android/release/repro /data/local/tmp/repro/
adb shell 'cd /data/local/tmp/repro && ./repro --spv spv/<file.spv>'
adb shell 'cd /data/local/tmp/repro && ./repro --wgsl wgsl/<file.wgsl>'
adb shell 'cd /data/local/tmp/repro && ./repro --wgpu wgsl/<file.wgsl>'
```

The default six-case mode (no arguments) cannot run on the device: it finds
`shaders/` through the build-time `CARGO_MANIFEST_DIR`. Each module therefore
ran by path. The shader sources sit next to the modules because the runner
reads the `expect:` line from them. Every process exited 0 on a pass and 1 on
a mismatch. No process panicked.
