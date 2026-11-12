use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/minecraft-version.txt");
    println!("cargo:rerun-if-env-changed=PUMPKIN_MINECRAFT_SERVER_JAR");

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let assets = manifest_dir.join("../../assets");
    println!(
        "cargo:rerun-if-changed={}",
        assets.join("datapack/data").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        assets.join("conventional_tags").display()
    );

    pumpkin_data_prepare::prepare(&assets)
}
