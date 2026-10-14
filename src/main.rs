//! Minimal reproduction: a uniform struct passed by value into a helper
//! function yields wrong field values on Adreno X1-85 / Vulkan.
//!
//! Every shader reads the same uniform buffer: 64 floats where float `i`
//! holds `i + 1`, so a value read back names the 4-byte slot it came from
//! (byte offset = (value - 1) * 4). The storage buffer is prefilled with -1.
//! Each shader states its expected readback in a `// expect:` line.
use std::path::{Path, PathBuf};

#[cfg(feature = "naga")]
mod naga_path;
mod vulkan;
#[cfg(feature = "wgpu")]
mod wgpu_path;

pub const OUTPUT_LEN: usize = 16;

/// Shaders run by default, from `shaders/`, as committed SPIR-V.
const DEFAULT_CASES: &[&str] = &[
    "by_value.spv",             // WGSL via naga          -> FAILS
    "scalar.spv",               // WGSL via naga, control -> passes
    "handwritten_by_value.spv", // hand-written SPIR-V    -> FAILS
    "handwritten_scalar.spv",   // hand-written, control  -> passes
    "by_value_glsl.spv",        // GLSL via glslang       -> passes (glslang copies member-wise)
    "scalar_glsl.spv",          // GLSL via glslang       -> passes
];

pub fn uniform_bytes() -> Vec<u8> {
    (0..64)
        .flat_map(|i| (i as f32 + 1.0).to_le_bytes())
        .collect()
}

/// Reads the `expect:` line of the shader source (or of the `.wgsl` / `.comp`
/// source with the same stem when given a `.spv`).
fn expected(path: &Path) -> Vec<f32> {
    let candidates = [
        path.to_path_buf(),
        path.with_extension("wgsl"),
        path.with_extension("comp"),
        path.with_extension("spvasm"),
    ];
    for candidate in candidates {
        let Ok(text) = std::fs::read_to_string(&candidate) else {
            continue;
        };
        if let Some(line) = text.lines().find(|l| l.contains("expect:")) {
            return line
                .split("expect:")
                .nth(1)
                .unwrap()
                .split_whitespace()
                .map(|v| v.parse().unwrap())
                .collect();
        }
    }
    panic!("no `expect:` line for {}", path.display());
}

fn read_spirv(path: &Path) -> Vec<u32> {
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    assert_eq!(bytes.len() % 4, 0);
    bytes
        .chunks_exact(4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .collect()
}

/// Prints the readback and returns whether it matches the expectation.
fn report(label: &str, values: &[f32], expect: &[f32]) -> bool {
    let ok = expect
        .iter()
        .zip(values)
        .all(|(e, v)| (e - v).abs() <= 1e-3);
    println!(
        "{:<5} {label}\n      expected {expect:?}\n      readback {:?}",
        if ok { "PASS" } else { "FAIL" },
        &values[..expect.len()]
    );
    ok
}

fn print_adapter(a: &vulkan::Adapter) {
    println!(
        "Vulkan device : {} (vendor 0x{:04x}, device 0x{:08x})",
        a.device_name, a.vendor_id, a.device_id
    );
    println!("API version   : {}", a.api_version);
    println!("driverVersion : 0x{:08x} (raw)", a.driver_version_raw);
    println!("driverName    : {}", a.driver_name);
    println!("driverInfo    : {}", a.driver_info);
    println!("conformance   : {}", a.conformance);
    println!();
}

fn run_spv(path: &Path, show_adapter: bool) -> bool {
    let (adapter, values) = vulkan::run(&read_spirv(path), &uniform_bytes(), OUTPUT_LEN);
    if show_adapter {
        print_adapter(&adapter);
    }
    report(
        &format!("vulkan(spirv) {}", path.display()),
        &values,
        &expected(path),
    )
}

fn usage() -> ! {
    eprintln!(
        "usage:
  repro                       run the committed SPIR-V cases in shaders/ via raw Vulkan
  repro --spv <file.spv>      run one SPIR-V module via raw Vulkan
  repro --wgsl <file.wgsl>    (feature naga) compile with naga, run via raw Vulkan
  repro --emit <file.wgsl>    (feature naga) write <file>.spv and <file>.spvasm
  repro --wgpu <file.wgsl> [--dx12]
                              (feature wgpu) run the WGSL through wgpu (Vulkan or DX12)"
    );
    std::process::exit(2)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let shaders = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders");
    let ok = match args.first().map(String::as_str) {
        None => {
            let mut all = true;
            let mut failed = Vec::new();
            for (i, name) in DEFAULT_CASES.iter().enumerate() {
                if !run_spv(&shaders.join(name), i == 0) {
                    all = false;
                    failed.push(*name);
                }
            }
            println!();
            if all {
                println!("all cases match: miscompilation NOT reproduced on this device/driver");
            } else {
                println!("mismatching cases: {failed:?}: miscompilation reproduced");
            }
            all
        }
        Some("--spv") => run_spv(Path::new(args.get(1).unwrap_or_else(|| usage())), true),
        #[cfg(feature = "naga")]
        Some("--wgsl") => {
            let path = Path::new(args.get(1).unwrap_or_else(|| usage()));
            let words = naga_path::compile(path);
            let (adapter, values) = vulkan::run(&words, &uniform_bytes(), OUTPUT_LEN);
            if args.iter().any(|a| a == "--adapter") {
                print_adapter(&adapter);
            }
            report(
                &format!("vulkan(naga) {}", path.display()),
                &values,
                &expected(path),
            )
        }
        #[cfg(feature = "naga")]
        Some("--emit") => {
            naga_path::emit(Path::new(args.get(1).unwrap_or_else(|| usage())));
            true
        }
        #[cfg(feature = "wgpu")]
        Some("--wgpu") => {
            let path = Path::new(args.get(1).unwrap_or_else(|| usage()));
            let dx12 = args.iter().any(|a| a == "--dx12");
            let values = pollster::block_on(wgpu_path::run(path, dx12));
            let backend = if dx12 { "dx12" } else { "vulkan" };
            report(
                &format!("wgpu({backend}) {}", path.display()),
                &values,
                &expected(path),
            )
        }
        _ => usage(),
    };
    std::process::exit(if ok { 0 } else { 1 });
}
