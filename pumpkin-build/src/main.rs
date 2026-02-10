use std::{env, fmt, fs, path::PathBuf};

fn is_cargo_run() -> bool {
    // These should only be set during compile-time
    env::var("CARGO").is_ok() && env::var("CARGO_MANIFEST_DIR").is_ok()
}

fn main() -> Result<(), BootstrapError> {
    if is_cargo_run() {
        eprintln!("Running with cargo. Bootstrapping is useless.")
    }

    let self_path = env::current_exe().map_err(BootstrapError::CurrentExe)?;

    // pumpkin-build crate dir
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // real pumpkin binary
    let real_bin = manifest_dir
        .join("..")
        .join("target")
        .join("release")
        .join("pumpkin");

    if !real_bin.exists() {
        return Err(BootstrapError::RealBinaryNotFound(real_bin));
    }

    let tmp = self_path.with_extension("tmp");

    fs::copy(&real_bin, &tmp).map_err(|e| BootstrapError::CopyFailed {
        from: real_bin.clone(),
        to: tmp.clone(),
        err: e,
    })?;

    fs::rename(&tmp, &self_path).map_err(|e| BootstrapError::ReplaceFailed {
        tmp: tmp.clone(),
        dest: self_path.clone(),
        err: e,
    })?;

    println!("`pumpkin` bootstrap complete");
    println!("Real binary installed at:");
    println!("- {:?}", self_path);
    if !is_cargo_run() {
        println!("You can now run `pumpkin` normally.");
    }
    Ok(())
}

#[derive(Debug)]
pub enum BootstrapError {
    CurrentExe(std::io::Error),
    RealBinaryNotFound(PathBuf),
    CopyFailed {
        from: PathBuf,
        to: PathBuf,
        err: std::io::Error,
    },
    ReplaceFailed {
        tmp: PathBuf,
        dest: PathBuf,
        err: std::io::Error,
    },
}

impl std::error::Error for BootstrapError {}

impl fmt::Display for BootstrapError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "bootstrap failed: ")?;
        match self {
            BootstrapError::CurrentExe(e) => {
                write!(f, "cannot get current executable path: {e}")
            }
            BootstrapError::RealBinaryNotFound(p) => {
                write!(f, "real pumpkin binary not found at {}", p.display())
            }
            BootstrapError::CopyFailed { from, to, err } => {
                write!(
                    f,
                    "cannot copy real binary from {} to {}: {err}",
                    from.display(),
                    to.display()
                )
            }
            BootstrapError::ReplaceFailed { tmp, dest, err } => {
                write!(
                    f,
                    "cannot replace {} with {}: {err}",
                    dest.display(),
                    tmp.display()
                )
            }
        }
    }
}
