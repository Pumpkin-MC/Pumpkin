use std::fs;

use pumpkin_config::{LoadConfiguration, PumpkinConfig};
use pumpkin_util::GameMode;

#[test]
fn missing_values_are_written_without_resetting_explicit_settings()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join(format!("pumpkin-config-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&dir)?;
    let path = dir.join("pumpkin.toml");
    fs::write(
        &path,
        "default_gamemode = \"Creative\"\nforce_gamemode = true\naccepts-transfers = false\n[telemetry]\nenabled = false\n",
    )?;

    let config = PumpkinConfig::load(&dir);
    let saved = fs::read_to_string(&path)?;
    fs::remove_dir_all(&dir)?;

    assert_eq!(config.basic.default_gamemode, GameMode::Creative);
    assert!(config.basic.force_gamemode);
    assert!(!config.basic.accepts_transfers);
    assert!(!config.telemetry.enabled);
    let reloaded: PumpkinConfig = toml::from_str(&saved)?;
    assert_eq!(reloaded.basic.default_gamemode, GameMode::Creative);
    assert!(reloaded.basic.force_gamemode);
    assert!(!reloaded.basic.accepts_transfers);
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
