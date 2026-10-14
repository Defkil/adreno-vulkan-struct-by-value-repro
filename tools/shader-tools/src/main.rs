//! `shader-tools validate <file.spv>...`  spirv-val --target-env vulkan1.0 equivalent
//! `shader-tools dis <file.spv>`           write <file>.spvasm (SPIRV-Tools disassembler)
//! `shader-tools as <file.spvasm>`      assemble SPIR-V text -> <file>.spv
//! `shader-tools glsl <file.comp>`         glslang (via shaderc) -> <file>.spv, Vulkan 1.0 / SPIR-V 1.0
use spirv_tools::{TargetEnv, assembler::Assembler, val::Validator};
use std::path::Path;

fn words(path: &Path) -> Vec<u32> {
    std::fs::read(path)
        .unwrap()
        .chunks_exact(4)
        .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
        .collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("validate") => {
            let validator = spirv_tools::val::create(Some(TargetEnv::Vulkan_1_0));
            let mut ok = true;
            for file in &args[1..] {
                match validator.validate(words(Path::new(file)), None) {
                    Ok(()) => println!("valid   {file}"),
                    Err(e) => {
                        ok = false;
                        println!("INVALID {file}: {e}")
                    }
                }
            }
            std::process::exit(if ok { 0 } else { 1 });
        }
        Some("dis") => {
            let path = Path::new(&args[1]);
            let assembler = spirv_tools::assembler::create(Some(TargetEnv::Vulkan_1_0));
            let text = assembler
                .disassemble(
                    words(path),
                    spirv_tools::assembler::DisassembleOptions {
                        use_friendly_names: true,
                        indent: true,
                        ..Default::default()
                    },
                )
                .unwrap()
                .unwrap();
            std::fs::write(path.with_extension("spvasm"), text).unwrap();
        }
        Some("as") => {
            // Assemble hand-written SPIR-V text (Vulkan 1.0 / SPIR-V 1.0).
            let path = Path::new(&args[1]);
            let text = std::fs::read_to_string(path).unwrap();
            let assembler = spirv_tools::assembler::create(Some(TargetEnv::Vulkan_1_0));
            let binary = assembler
                .assemble(&text, Default::default())
                .unwrap_or_else(|e| panic!("{e}"));
            let bytes: Vec<u8> = binary
                .as_words()
                .iter()
                .flat_map(|w| w.to_le_bytes())
                .collect();
            std::fs::write(path.with_extension("spv"), bytes).unwrap();
            println!("wrote {}", path.with_extension("spv").display());
        }
        Some("glsl") => {
            let path = Path::new(&args[1]);
            let source = std::fs::read_to_string(path).unwrap();
            let compiler = shaderc::Compiler::new().unwrap();
            let mut options = shaderc::CompileOptions::new().unwrap();
            options.set_target_env(
                shaderc::TargetEnv::Vulkan,
                shaderc::EnvVersion::Vulkan1_0 as u32,
            );
            options.set_target_spirv(shaderc::SpirvVersion::V1_0);
            options.set_optimization_level(shaderc::OptimizationLevel::Zero);
            let artifact = compiler
                .compile_into_spirv(
                    &source,
                    shaderc::ShaderKind::Compute,
                    &path.to_string_lossy(),
                    "main",
                    Some(&options),
                )
                .unwrap_or_else(|e| panic!("{e}"));
            std::fs::write(path.with_extension("spv"), artifact.as_binary_u8()).unwrap();
            println!("wrote {}", path.with_extension("spv").display());
        }
        _ => {
            eprintln!(
                "usage: shader-tools validate <spv>... | dis <spv> | as <spvasm> | glsl <comp>"
            );
            std::process::exit(2);
        }
    }
}
