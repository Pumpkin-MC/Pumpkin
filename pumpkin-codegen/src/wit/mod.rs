pub mod bedrock_packet;
pub mod entity_type;
pub mod java_packet;
pub mod packet_mapping;
pub mod particle;
pub mod sound;
pub mod utils;

use std::fs;
use std::path::Path;

pub const WIT_OUT_DIR: &str = "../pumpkin-plugin-wit/v0.1";
pub const MAPPING_OUT_DIR: &str = "../pumpkin/src/plugin/loader/wasm/wasm_host/wit/v0_1";

pub fn main() {
    fs::create_dir_all(WIT_OUT_DIR).expect("Failed to create WIT output directory");

    type BuildFn = fn() -> String;
    let build_functions: Vec<(BuildFn, &str)> = vec![
        (particle::build, "particles.wit"),
        (sound::build, "sounds.wit"),
        (entity_type::build, "entity-types.wit"),
        (java_packet::build, "java-packets.wit"),
        (bedrock_packet::build, "bedrock-packets.wit"),
    ];

    for (build_fn, file) in build_functions {
        println!("Generating WIT for {}", file);
        let wit_code = build_fn();
        write_generated_wit(&wit_code, file);
    }

    println!("Generating Java and Bedrock packet mapping");
    let mut mapping = packet_mapping::build_java_mapping();
    mapping.push_str(&packet_mapping::build_bedrock_mapping());
    let mapping_path = Path::new(MAPPING_OUT_DIR).join("generated_packets.rs");
    fs::write(&mapping_path, mapping).expect("Failed to write packet mapping");
}

fn write_generated_wit(new_code: &str, out_file: &str) {
    let path = Path::new(WIT_OUT_DIR).join(out_file);

    if path.exists()
        && let Ok(existing_code) = fs::read_to_string(&path)
        && existing_code == new_code
    {
        return;
    }

    fs::write(&path, new_code)
        .unwrap_or_else(|_| panic!("Failed to write to file: {}", path.display()));
}
