pub mod block;
mod helpers;
pub mod item;

use std::fs;
use std::path::Path;

const SDK_OUT_DIRS: [&str; 1] = ["../../crates/pumpkin-plugin-api/src/generated"];

pub fn main() {
    fs::write(
        "../../crates/pumpkin-plugin-helpers/src/generated.rs",
        helpers::build(),
    )
    .expect("Failed to write shared helper data");
    for directory in SDK_OUT_DIRS {
        fs::create_dir_all(directory).expect("Failed to create SDK generated directory");
    }

    let targets: Vec<(fn() -> String, &str)> =
        vec![(item::build, "item.rs"), (block::build, "block.rs")];

    for (build_fn, file) in targets {
        println!("Generating SDK for {}", file);
        let code = build_fn();
        for directory in SDK_OUT_DIRS {
            let path = Path::new(directory).join(file);
            fs::write(&path, &code).unwrap_or_else(|_| panic!("Failed to write {file}"));
        }
    }
}
