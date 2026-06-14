use std::process::Command;

fn main() {
    // Sync submodules
    Command::new("git")
        .args(["submodule", "update", "--init"])
        .current_dir("..")
        .output()
        .expect("Failed to update submodules");

    println!("cargo::rerun-if-changed=../.git/HEAD");
    println!("cargo::rerun-if-changed=../.git/refs/heads/");
}
