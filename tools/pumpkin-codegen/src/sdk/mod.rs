pub mod block;
pub mod item;

use std::fs;
use std::path::Path;

pub const SDK_OUT_DIR: &str = "../../crates/pumpkin-plugin-api/src/generated";

pub fn main() {
    fs::create_dir_all(SDK_OUT_DIR).expect("Failed to create SDK generated directory");

    let targets: Vec<(fn() -> String, &str)> =
        vec![(item::build, "item.rs"), (block::build, "block.rs")];

    for (build_fn, file) in targets {
        println!("Generating SDK for {}", file);
        let code = build_fn();
        let path = Path::new(SDK_OUT_DIR).join(file);
        fs::write(&path, code).unwrap_or_else(|_| panic!("Failed to write {file}"));
    }
}
