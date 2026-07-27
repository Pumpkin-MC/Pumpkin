use std::process::Command;

/// Runs a command and returns its trimmed stdout, or `None` when the command is
/// missing or exits with a failure status.
fn capture(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

/// Same as [`capture`], but falls back to `"unknown"` so that a build without
/// git (release tarball, vendored source, Docker layer without `.git`) still
/// compiles instead of failing.
fn capture_or_unknown(program: &str, args: &[&str]) -> String {
    capture(program, args).unwrap_or_else(|| "unknown".to_string())
}

fn main() {
    // Short hash (7 chars) for display, full hash for hover text.
    let git_hash_short = capture_or_unknown("git", &["rev-parse", "--short=7", "HEAD"]);
    let git_hash_full = capture_or_unknown("git", &["rev-parse", "HEAD"]);

    // Branch the binary was built from. Detached HEAD reports "HEAD", which we
    // normalize to "detached" so the value reads sensibly in `/about`.
    let git_branch = match capture("git", &["rev-parse", "--abbrev-ref", "HEAD"]) {
        Some(branch) if branch == "HEAD" => "detached".to_string(),
        Some(branch) => branch,
        None => "unknown".to_string(),
    };

    // Whether the working tree had uncommitted changes at build time. A build
    // without git reports "unknown" rather than pretending to be clean.
    let git_dirty = match capture("git", &["status", "--porcelain", "--untracked-files=no"]) {
        Some(changes) => {
            if changes.is_empty() {
                "clean"
            } else {
                "dirty"
            }
        }
        // `git status` prints nothing when the tree is clean, so `capture`
        // returns `None` both for "clean" and for "git unavailable". Only treat
        // it as clean when we know we are inside a work tree.
        None => match capture("git", &["rev-parse", "--is-inside-work-tree"]) {
            Some(inside) if inside == "true" => "clean",
            _ => "unknown",
        },
    }
    .to_string();

    // Compiler that produced this binary, e.g. "rustc 1.90.0 (...)".
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let rustc_version = capture_or_unknown(&rustc, &["--version"]);

    // Target triple is provided by Cargo; no need to shell out for it.
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

    println!("cargo::rerun-if-changed=../.git/HEAD");
    println!("cargo::rerun-if-changed=../.git/refs/heads/");
    println!("cargo::rustc-env=GIT_HASH={git_hash_short}");
    println!("cargo::rustc-env=GIT_HASH_FULL={git_hash_full}");
    println!("cargo::rustc-env=GIT_BRANCH={git_branch}");
    println!("cargo::rustc-env=GIT_DIRTY={git_dirty}");
    println!("cargo::rustc-env=BUILD_TARGET={target}");
    println!("cargo::rustc-env=BUILD_RUSTC_VERSION={rustc_version}");
}
