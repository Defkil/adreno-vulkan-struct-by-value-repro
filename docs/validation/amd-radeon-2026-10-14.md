# Radeon comparison, 2026-10-14

The miscompilation was **not reproduced** on this desktop's AMD Radeon RX 7900
XTX. All 100 GPU checks matched their expected readbacks. This validates the
standalone shader reproduction on this device/driver; the original Bevy game
and other AMD drivers were not tested.

## Source and environment

- Public repository: [Defkil/adreno-vulkan-struct-by-value-repro](https://github.com/Defkil/adreno-vulkan-struct-by-value-repro).
- Exact source: `c0e58657446649a622811d2a8b0d016f9507cc6b`; downloaded as a
  GitHub source archive into temporary storage. No source or shader changes.
- GPU: AMD Radeon RX 7900 XTX, vendor `0x1002`, device `0x744c`.
- Vulkan: API `1.4.349`, driver `AMD proprietary driver`, driver info
  `26.8.1 (LLPC)`, raw version `0x0080018b`, conformance `1.4.3.3`.
- Windows display driver: `32.0.31041.1004`, dated 2026-08-17.
- OS: Windows 11 Pro 25H2, build `26200.9550`, x64.
- Rust: `rustc 1.99.0 (b940084d7 2026-09-28)`,
  `x86_64-pc-windows-msvc`; repository Cargo.lock used with `--locked`.
- Crates: ash `0.38.0+1.3.281`, naga `30.0.1`, wgpu `30.0.1`.

## Results

| Path | Cases | Passed | Failed |
|---|---:|---:|---:|
| Committed `shaders/*.spv`, raw Vulkan | 6 | 6 | 0 |
| Committed `experiments/spirv/*.spv`, raw Vulkan | 7 | 7 | 0 |
| WGSL compiled by naga, raw Vulkan | 29 | 29 | 0 |
| WGSL through wgpu, Vulkan | 29 | 29 | 0 |
| WGSL through wgpu, DX12 | 29 | 29 | 0 |
| Total | 100 | 100 | 0 |

The 29 WGSL inputs are the two standard shaders plus all 27 experiment files.
Each process exited 0. Both `by_value.spv` and `handwritten_by_value.spv`
returned `[4.0, 4.0]`, compared with the previously recorded Adreno result
`[5.0, 4.0]`. The two-vec3 experiment returned `[4.0, 5.0, 8.0]` on all three
WGSL paths, compared with `[5.0, 9.0, 13.0]` on Adreno Vulkan.

Full expected/actual readbacks and adapter output:
[amd-radeon-2026-10-14.txt](amd-radeon-2026-10-14.txt).

## Commands

```powershell
cargo run --release --locked
cargo build --release --locked --features naga,wgpu

# Run for every experiments/spirv/*.spv file:
.\target\release\repro.exe --spv <file.spv>

# Run each command for every shaders/*.wgsl and experiments/*.wgsl file:
.\target\release\repro.exe --wgsl <file.wgsl>
.\target\release\repro.exe --wgpu <file.wgsl>
.\target\release\repro.exe --wgpu <file.wgsl> --dx12
```

The default six-case command ran before the optional-feature build. The other
94 checks ran sequentially using that build.
