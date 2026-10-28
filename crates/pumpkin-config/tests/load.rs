use std::{fs, process::Command};

use pumpkin_config::{LoadConfiguration, PumpkinConfig};
use pumpkin_util::GameMode;

#[test]
fn configuration_load_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
    const CONFIG_DIR: &str = "PUMPKIN_CONFIG_TEST_DIR";
    const EXPECTED_MODE: &str = "PUMPKIN_CONFIG_TEST_EXPECTED_MODE";
    if let Some(dir) = std::env::var_os(CONFIG_DIR) {
        let config = PumpkinConfig::load(std::path::Path::new(&dir));
        assert_eq!(
            format!("{:?}", config.basic.default_gamemode),
            std::env::var(EXPECTED_MODE)?
        );
        assert!(config.basic.force_gamemode);
        assert_eq!(config.basic.default_level_name, "kids");
        assert_eq!(config.basic.tps, 12.5);
        return Ok(());
    }

    // Each child loads the real configuration without a tracing subscriber.
    for (source, diagnostic, expected_mode, field) in [
        (
            "default_gamemode = Creative\n",
            "default_gamemode = Creative",
            "Survival",
            "default_gamemode",
        ),
        (
            "default_gamemode =\n",
            "default_gamemode =",
            "Survival",
            "default_gamemode",
        ),
        (
            "default_gamemode = \"Invalid\"\n",
            "default_gamemode = \"Invalid\"",
            "Survival",
            "default_gamemode",
        ),
        (
            "default_gamemode = \"Creative\"\n[networking.java]\nmax_players = \"six\"\n",
            "max_players = \"six\"",
            "Creative",
            "networking.java.max_players",
        ),
        (
            "default_gamemode = \"Creative\"\n[telemetry]\nenabled = \"yes\"\n",
            "enabled = \"yes\"",
            "Creative",
            "telemetry.enabled",
        ),
        (
            "default_gamemode = \"Creative\"\n[logging]\nfile = latest.log\n",
            "file = latest.log",
            "Creative",
            "logging.file",
        ),
        (
            "default_gamemode = \"Creative\"\n[world.chunk]\ntype = \"anvil\"\n[world.chunk.compression]\nlevel = \"six\"\n",
            "level = \"six\"",
            "Creative",
            "world.chunk.compression.level",
        ),
    ] {
        let source =
            format!("force_gamemode = true\ndefault_level_name = \"kids\"\ntps = 12.5\n{source}");
        let dir = std::env::temp_dir().join(format!("pumpkin-config-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&dir)?;
        let path = dir.join("pumpkin.toml");
        fs::write(&path, &source)?;
        let output = Command::new(std::env::current_exe()?)
            .args(["--exact", "configuration_load_diagnostics", "--nocapture"])
            .env(CONFIG_DIR, &dir)
            .env(EXPECTED_MODE, expected_mode)
            .output()?;
        let unchanged = fs::read_to_string(&path)?;
        fs::remove_dir_all(&dir)?;

        let stderr = String::from_utf8(output.stderr)?;
        let stdout = String::from_utf8(output.stdout)?;
        assert!(output.status.success(), "{stdout}\n{stderr}");
        assert!(stderr.contains(&path.display().to_string()), "{stderr}");
        assert!(stderr.contains(diagnostic), "{stderr}");
        assert!(
            stderr.contains(&format!("Invalid setting `{field}`")),
            "{stderr}"
        );
        assert!(stderr.contains("line "), "{stderr}");
        assert!(stderr.contains("column "), "{stderr}");
        assert!(
            stderr.contains("Valid settings have been retained."),
            "{stderr}"
        );
        assert!(
            stderr.contains("Falling back to default configuration settings for invalid fields."),
            "{stderr}"
        );
        assert!(!stdout.contains("values were missing"), "{stdout}");
        assert_eq!(unchanged, source);
    }
    Ok(())
}

#[test]
fn missing_values_are_written_without_resetting_explicit_settings()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join(format!("pumpkin-config-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&dir)?;
    let path = dir.join("pumpkin.toml");
    fs::write(
        &path,
        "default_gamemode = \"Creative\"\nforce_gamemode = true\n[telemetry]\nenabled = false\n",
    )?;

    let config = PumpkinConfig::load(&dir);
    let saved = fs::read_to_string(&path)?;
    fs::remove_dir_all(&dir)?;

    assert_eq!(config.basic.default_gamemode, GameMode::Creative);
    assert!(config.basic.force_gamemode);
    assert!(!config.telemetry.enabled);
    let reloaded: PumpkinConfig = toml::from_str(&saved)?;
    assert_eq!(reloaded.basic.default_gamemode, GameMode::Creative);
    assert!(reloaded.basic.force_gamemode);
    assert!(!reloaded.telemetry.enabled);
    assert!(saved.contains("default_level_name"));
    Ok(())
}

#[test]
fn invalid_values_do_not_reset_other_fields_in_the_same_table()
-> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
default_gamemode = "Creative"
force_gamemode = true
default_difficulty = "Invalid"
difficulty = peaceful

[networking.java]
max_players = "six"
motd = "Family server"
view_distance = 8

[networking.java.compression]
level = "invalid"
threshold = 512

[logging]
file = latest.log
level = "debug"

[world.chunk]
type = "anvil"

[world.chunk.compression]
algorithm = "ZLib"
level = "six"

[telemetry]
enabled = "yes"
interval_secs = 600
"#;
    let dir = std::env::temp_dir().join(format!("pumpkin-config-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&dir)?;
    let path = dir.join("pumpkin.toml");
    fs::write(&path, source)?;
    let config = PumpkinConfig::load(&dir);
    let unchanged = fs::read_to_string(&path)?;
    fs::remove_dir_all(&dir)?;

    assert_eq!(config.basic.default_gamemode, GameMode::Creative);
    assert!(config.basic.force_gamemode);
    assert_eq!(
        config.basic.default_difficulty,
        pumpkin_util::Difficulty::Normal
    );
    assert_eq!(config.advanced.networking.java.motd, "Family server");
    assert_eq!(config.advanced.networking.java.view_distance.get(), 8);
    assert_eq!(config.advanced.networking.java.max_players, 1000);
    assert_eq!(config.advanced.networking.java.compression.info.level, 4);
    assert_eq!(
        config.advanced.networking.java.compression.info.threshold,
        512
    );
    assert_eq!(config.advanced.logging.level, "debug");
    let pumpkin_config::chunk::ChunkConfig::Anvil(anvil) = config.advanced.world.chunk else {
        return Err("the valid Anvil storage format should be retained".into());
    };
    assert_eq!(anvil.compression.level, 6);
    assert!(matches!(
        anvil.compression.algorithm,
        pumpkin_config::chunk::Compression::ZLib
    ));
    assert!(config.telemetry.enabled);
    assert_eq!(config.telemetry.interval_secs, 600);
    assert_eq!(unchanged, source);
    Ok(())
}
