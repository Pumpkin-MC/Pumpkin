pub mod bedrock_packet;
pub mod entity_type;
pub mod java_packet;
pub mod packet_mapping;
pub mod particle;
pub mod sound;
pub mod utils;

use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

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

    mapping = format_code(&mapping).unwrap_or(mapping);

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

/// Error returned when `rustfmt` is unavailable or fails to format code.
pub struct RustFmtError;

/// Formats a Rust source string by piping it through `rustfmt`.
///
/// # Arguments
/// - `unformatted_code` – Raw Rust source code to format.
///
/// # Returns
/// The formatted source string, or `Err(RustFmtError)` if `rustfmt` is not available
/// or formatting fails.
pub fn format_code(unformatted_code: &str) -> Result<String, RustFmtError> {
    let child_result = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();

    let Ok(mut child) = child_result else {
        return Err(RustFmtError);
    };

    // Write the code to rustfmt's stdin
    if let Some(mut stdin) = child.stdin.take()
        && stdin.write_all(unformatted_code.as_bytes()).is_err()
    {
        return Err(RustFmtError);
    }

    match child.wait_with_output() {
        Ok(output) if output.status.success() => {
            String::from_utf8(output.stdout).map_err(|_| RustFmtError)
        }
        _ => Err(RustFmtError),
    }
}
