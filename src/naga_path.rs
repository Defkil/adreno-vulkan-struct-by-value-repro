//! WGSL -> SPIR-V with naga's default SPIR-V writer options (the committed
//! `.spv` files are produced by `emit`).
use rspirv::binary::Disassemble;
use std::path::Path;

pub fn compile(path: &Path) -> Vec<u32> {
    let source = std::fs::read_to_string(path).unwrap();
    let module = naga::front::wgsl::parse_str(&source)
        .unwrap_or_else(|e| panic!("{}", e.emit_to_string(&source)));
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .unwrap();
    naga::back::spv::write_vec(
        &module,
        &info,
        &naga::back::spv::Options {
            lang_version: (1, 0),
            ..Default::default()
        },
        Some(&naga::back::spv::PipelineOptions {
            shader_stage: naga::ShaderStage::Compute,
            entry_point: "main".into(),
        }),
    )
    .unwrap()
}

pub fn emit(path: &Path) {
    let words = compile(path);
    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    std::fs::write(path.with_extension("spv"), bytes).unwrap();
    std::fs::write(
        path.with_extension("spvasm"),
        rspirv::dr::load_words(&words).unwrap().disassemble() + "\n",
    )
    .unwrap();
    println!("wrote {} and .spvasm", path.with_extension("spv").display());
}
