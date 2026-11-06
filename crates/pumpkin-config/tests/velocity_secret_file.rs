#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use pumpkin_config::{LoadConfiguration, PumpkinConfig};
use std::fs;

#[test]
fn startup_resolves_the_file_without_writing_its_contents_back() {
    for proxy_enabled in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let secret_path = directory.path().join("forwarding.secret");
        fs::write(&secret_path, "fixture-secret-not-for-config\n").unwrap();
        let config_path = directory.path().join("pumpkin.toml");
        let file_value = toml::Value::String(secret_path.to_str().unwrap().to_owned());
        fs::write(
            &config_path,
            format!(
                "[networking.proxy]\nenabled = {proxy_enabled}\n\
                 [networking.proxy.velocity]\nenabled = true\nsecret_file = {file_value}\n"
            ),
        )
        .unwrap();
        let config = PumpkinConfig::load(directory.path());
        // A later login must use the startup snapshot, without another file read.
        fs::remove_file(&secret_path).unwrap();
        assert_eq!(
            config
                .advanced
                .networking
                .proxy
                .velocity
                .forwarding_secret()
                .unwrap(),
            "fixture-secret-not-for-config"
        );
        let written = fs::read_to_string(config_path).unwrap();
        assert!(!written.contains("fixture-secret-not-for-config"));
        let written: toml::Value = toml::from_str(&written).unwrap();
        assert_eq!(
            written["networking"]["proxy"]["velocity"]["secret_file"],
            file_value
        );
        assert_eq!(
            written["networking"]["proxy"]["velocity"]["secret"].as_str(),
            Some("")
        );
    }
}

#[test]
fn missing_file_fails_startup_only_when_velocity_is_enabled() {
    for enabled in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("missing.secret");
        let file_value = toml::Value::String(path.to_str().unwrap().to_owned());
        fs::write(
            directory.path().join("pumpkin.toml"),
            format!(
                "[networking.proxy.velocity]\nenabled = {enabled}\nsecret_file = {file_value}\n"
            ),
        )
        .unwrap();
        let result = std::panic::catch_unwind(|| PumpkinConfig::load(directory.path()));
        assert_eq!(result.is_err(), enabled);
    }
}
