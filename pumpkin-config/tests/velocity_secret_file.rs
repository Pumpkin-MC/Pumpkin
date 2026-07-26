//! End-to-end coverage for reading the Velocity forwarding secret from a file.
//!
//! These drive the real [`PumpkinConfig::load`] path rather than a hand-built
//! struct, because the interesting behaviour only appears there: `load` writes
//! the config back to disk whenever it fills in missing defaults, which is
//! exactly when a resolved secret could leak into `pumpkin.toml`.

use std::fs;

use pumpkin_config::{LoadConfiguration, PumpkinConfig};
use tempfile::TempDir;

const SECRET: &str = "s3cret-that-must-not-be-written-back";

/// Writes a secret file and a deliberately incomplete `pumpkin.toml` that points
/// at it, then loads the config from that directory.
///
/// The config is incomplete on purpose: every field left out is filled in from
/// the defaults, which is what makes `load` rewrite the file.
fn load_with_secret_file(proxy_enabled: bool) -> (TempDir, PumpkinConfig) {
    let dir = TempDir::new().expect("failed to create temp dir");

    let secret_path = dir.path().join("forwarding.secret");
    fs::write(&secret_path, format!("{SECRET}\n")).expect("failed to write secret file");

    // An absolute path keeps the test independent of the working directory the
    // test harness happens to run in.
    let config = format!(
        "[networking.proxy]\n\
         enabled = {proxy_enabled}\n\
         \n\
         [networking.proxy.velocity]\n\
         enabled = true\n\
         secret = \"@{}\"\n",
        secret_path.display().to_string().replace('\\', "\\\\")
    );
    fs::write(dir.path().join("pumpkin.toml"), config).expect("failed to write config");

    let loaded = PumpkinConfig::load(dir.path());
    (dir, loaded)
}

#[test]
fn secret_is_read_from_the_referenced_file() {
    let (_dir, config) = load_with_secret_file(true);

    assert_eq!(
        config.advanced.networking.proxy.velocity.secret(),
        SECRET,
        "the secret should have been read from the referenced file"
    );
}

#[test]
fn resolved_secret_is_not_written_back_to_the_config() {
    let (dir, config) = load_with_secret_file(true);

    // Force resolution, mirroring what a real server does while handling a login.
    assert_eq!(config.advanced.networking.proxy.velocity.secret(), SECRET);

    let written =
        fs::read_to_string(dir.path().join("pumpkin.toml")).expect("failed to read back config");

    assert!(
        !written.contains(SECRET),
        "the resolved secret leaked into pumpkin.toml:\n{written}"
    );
    assert!(
        written.contains("forwarding.secret"),
        "the `@file` reference was not preserved in pumpkin.toml:\n{written}"
    );
}

/// A `@file` reference must keep working when the proxy master switch is off, so
/// that flipping it back on does not surprise the operator with a broken secret.
#[test]
fn reference_still_resolves_with_the_proxy_master_switch_off() {
    let (_dir, config) = load_with_secret_file(false);

    assert_eq!(config.advanced.networking.proxy.velocity.secret(), SECRET);
}

/// TOML basic strings treat a backslash as an escape, so Windows users are told
/// to write absolute paths as literal (single-quoted) strings. This checks that
/// the documented form actually round-trips through the loader.
#[test]
fn literal_string_paths_resolve() {
    let dir = TempDir::new().expect("failed to create temp dir");

    let secret_path = dir.path().join("forwarding.secret");
    fs::write(&secret_path, SECRET).expect("failed to write secret file");

    let config = format!(
        "[networking.proxy]\n\
         enabled = true\n\
         \n\
         [networking.proxy.velocity]\n\
         enabled = true\n\
         secret = '@{}'\n",
        secret_path.display()
    );
    fs::write(dir.path().join("pumpkin.toml"), config).expect("failed to write config");

    let loaded = PumpkinConfig::load(dir.path());

    assert_eq!(loaded.advanced.networking.proxy.velocity.secret(), SECRET);
}
