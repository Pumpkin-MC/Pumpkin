use std::process::Command;

fn main() {
    // Get short hash (7 chars) for display
    let short_output = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output();

    let git_hash_short = match short_output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    };

    // Get full hash for hover text
    let full_output = Command::new("git").args(["rev-parse", "HEAD"]).output();

    let git_hash_full = match full_output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    };

    println!("cargo::rerun-if-changed=../.git/HEAD");
    println!("cargo::rerun-if-changed=../.git/refs/heads/");
    println!("cargo::rustc-env=GIT_HASH={git_hash_short}");
    println!("cargo::rustc-env=GIT_HASH_FULL={git_hash_full}");
}
