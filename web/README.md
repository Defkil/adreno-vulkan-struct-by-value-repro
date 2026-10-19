# WebGPU reach (browser)

The same miscompilation is reachable from a web page through WebGPU on Adreno,
because a browser's WGSL compiler can emit the trigger pattern. These two pages
reproduce and bound it without any native build.

## Pages

- [`index.html`](index.html) — runs the minimal `by_value` shader and its scalar
  control, each from a `var<uniform>` and a `var<storage>` buffer, and prints the
  readback.
- [`oob.html`](oob.html) — binds the struct to **exactly its 16 bytes** with a
  sentinel value just past the binding, to check whether the faulty read (byte
  offset 16 instead of 12) crosses the binding boundary.

## Run

Serve this directory and open a page in a WebGPU browser:

```text
python -m http.server 8765
# desktop: http://localhost:8765/
# Android over USB: adb reverse tcp:8765 tcp:8765, then open http://localhost:8765/
```

## Results on Adreno 650 (Samsung Galaxy Tab S7, Chrome 155, 2026-10-19)

| Page / case | Expected | Readback | |
| --- | --- | --- | --- |
| `index.html` uniform `by_value` | `[4, 4]` | `[4, 4]` | pass |
| `index.html` uniform scalar (control) | `[4, 4]` | `[4, 4]` | pass |
| `index.html` storage `by_value` | `[4, 4]` | `[5, 4]` | **FAIL** |
| `index.html` storage scalar (control) | `[4, 4]` | `[4, 4]` | pass |
| `oob.html` storage `by_value`, 16-byte binding | `[4, 4]` | `[0, 4]` (helper clamped) | contained |

Two observations:

- **Uniform is spared, storage is not.** Chrome's Tint decomposes a uniform
  buffer into `array<vec4<u32>>` and rebuilds the struct member by member, so no
  whole-struct `OpLoad` reaches the call. Storage buffers keep the whole-struct
  load, so the driver bug fires on current Chrome. This is incidental to Tint's
  uniform handling, not an Adreno-specific fix.
- **Not an out-of-binding read (contained by robustness).** When the struct is
  bound to only its 16 bytes, the faulty read at byte offset 16 lands outside the
  binding and is clamped to `0` by WebGPU's robustness; the sentinel past the
  binding is never returned. So this is a correctness bug reachable from the web,
  not a memory-disclosure bug.

Both results were deterministic across repeated runs. The storage `by_value`
failure appears the same on current Brave, Edge and Opera (all Chromium); an
older Brave (1.71.123, 2024) failed on `var<uniform>` too, before Tint
decomposed uniform buffers.

## Firefox

Firefox does not reach Qualcomm's Vulkan compiler in a default build: release
WebGPU is Windows (DX12) and Apple-silicon macOS (Metal) only. Even with WebGPU
force-enabled on Android (`dom.webgpu.enabled`, `gfx.webgpu.ignore-blocklist`),
Firefox Nightly 160 returned a `null` adapter on the Adreno 650: wgpu's
`STRICT_WEBGPU_COMPLIANCE` filter (which Firefox sets) hides the adapter because
it is missing the `SURFACE_VIEW_FORMATS` downlevel capability. Chrome's Dawn does
not require that, so only Chromium-based browsers reach the bug here.
