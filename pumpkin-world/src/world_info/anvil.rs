use std::{
    fs::{self, File},
    io::{Cursor, Read},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::{error, warn};

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};

use crate::world_info::{
    MAXIMUM_SUPPORTED_LEVEL_VERSION, MAXIMUM_SUPPORTED_WORLD_DATA_VERSION,
    MINIMUM_SUPPORTED_LEVEL_VERSION, MINIMUM_SUPPORTED_WORLD_DATA_VERSION,
    data_files::{
        minecraft_data_dir, read_game_rules, read_wandering_trader, read_weather,
        read_world_clocks, read_world_gen_settings, write_custom_boss_events_stub,
        write_game_rules, write_scheduled_events_stub, write_wandering_trader, write_weather,
        write_world_clocks, write_world_gen_settings,
    },
};

use super::{LevelData, WorldInfoError, WorldInfoReader, WorldInfoWriter};

pub const LEVEL_DAT_FILE_NAME: &str = "level.dat";
pub const LEVEL_DAT_BACKUP_FILE_NAME: &str = "level.dat_old";

pub struct AnvilLevelInfo;

fn read_level_dat(path: &Path) -> Result<LevelData, WorldInfoError> {
    let world_info_file = File::open(path)?;
    let mut buf = Vec::new();
    GzDecoder::new(world_info_file).read_to_end(&mut buf)?;

    check_file_data_version(&buf)?;
    check_file_level_version(&buf)?;
    let info = pumpkin_nbt::from_bytes::<LevelDat>(Cursor::new(buf))
        .map_err(|e| WorldInfoError::DeserializationError(e.to_string()))?;

    Ok(info.data)
}

fn write_level_dat(level: &LevelDat, level_folder: &Path) -> Result<(), WorldInfoError> {
    let current_path = level_folder.join(LEVEL_DAT_FILE_NAME);
    let backup_path = level_folder.join(LEVEL_DAT_BACKUP_FILE_NAME);
    let temporary_path =
        level_folder.join(format!("{LEVEL_DAT_FILE_NAME}.{}.tmp", std::process::id()));

    let temporary_file = File::create(&temporary_path)?;
    let mut compression_writer = GzEncoder::new(temporary_file, Compression::best());
    pumpkin_nbt::to_bytes(level, &mut compression_writer)
        .map_err(|e| WorldInfoError::SerializationError(e.to_string()))?;
    let temporary_file = compression_writer.finish()?;
    temporary_file.sync_all()?;
    drop(temporary_file);

    if current_path.exists() {
        if backup_path.exists() {
            fs::remove_file(&backup_path)?;
        }
        fs::rename(&current_path, &backup_path)?;
    }

    if let Err(error) = fs::rename(&temporary_path, &current_path) {
        if !current_path.exists() && backup_path.exists() {
            let _ = fs::rename(&backup_path, &current_path);
        }
        return Err(error.into());
    }

    Ok(())
}

fn restore_level_dat_from_backup(
    current_path: &Path,
    backup_path: &Path,
) -> Result<(), WorldInfoError> {
    let corrupted_path = current_path.with_file_name("level.dat_corrupted");

    if current_path.exists() {
        if corrupted_path.exists() {
            fs::remove_file(&corrupted_path)?;
        }
        fs::rename(current_path, &corrupted_path)?;
    }

    if let Err(error) = fs::rename(backup_path, current_path) {
        if !current_path.exists() && corrupted_path.exists() {
            let _ = fs::rename(&corrupted_path, current_path);
        }
        return Err(error.into());
    }

    Ok(())
}

fn check_file_data_version(raw_nbt: &[u8]) -> Result<(), WorldInfoError> {
    // Define a struct that only has the data version. This is necessary because if a user tries to
    // load a world with different data, they will get a generic "Failed to deserialize level.dat error".
    // When only checking for the data version, we can determine if we can support the full
    // deserializiation before going through with it.
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct LevelData {
        data_version: i32,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct LevelDat {
        data: LevelData,
    }

    let info: LevelDat = pumpkin_nbt::from_bytes(Cursor::new(raw_nbt))
        .map_err(|e|{
            error!("The level.dat file does not have a data version! This means it is either corrupt or very old (read unsupported)");
            WorldInfoError::DeserializationError(e.to_string())})?;

    let data_version = info.data.data_version;

    if (MINIMUM_SUPPORTED_WORLD_DATA_VERSION..=MAXIMUM_SUPPORTED_WORLD_DATA_VERSION)
        .contains(&data_version)
    {
        Ok(())
    } else {
        Err(WorldInfoError::UnsupportedDataVersion(data_version))
    }
}

fn check_file_level_version(raw_nbt: &[u8]) -> Result<(), WorldInfoError> {
    #[derive(Deserialize)]
    struct LevelData {
        version: i32,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct LevelDat {
        data: LevelData,
    }

    let info: LevelDat = pumpkin_nbt::from_bytes(Cursor::new(raw_nbt))
        .map_err(|e|{
            error!("The level.dat file does not have a level version! This means it is either corrupt or very old (read unsupported)");
            WorldInfoError::DeserializationError(e.to_string())})?;

    let level_version = info.data.version;

    if (MINIMUM_SUPPORTED_LEVEL_VERSION..=MAXIMUM_SUPPORTED_LEVEL_VERSION).contains(&level_version)
    {
        Ok(())
    } else {
        Err(WorldInfoError::UnsupportedLevelVersion(level_version))
    }
}

impl WorldInfoReader for AnvilLevelInfo {
    fn read_world_info(&self, level_folder: &Path) -> Result<LevelData, WorldInfoError> {
        let path = level_folder.join(LEVEL_DAT_FILE_NAME);
        let backup_path = level_folder.join(LEVEL_DAT_BACKUP_FILE_NAME);

        let mut data = match read_level_dat(&path) {
            Ok(data) => data,
            Err(
                error @ (WorldInfoError::UnsupportedDataVersion(_)
                | WorldInfoError::UnsupportedLevelVersion(_)),
            ) => return Err(error),
            Err(primary_error) => match read_level_dat(&backup_path) {
                Ok(data) => {
                    restore_level_dat_from_backup(&path, &backup_path)?;
                    warn!(
                        error = %primary_error,
                        current = %path.display(),
                        backup = %backup_path.display(),
                        "Failed to read level.dat; using level.dat_old"
                    );
                    data
                }
                Err(_) => return Err(primary_error),
            },
        };

        // game_rules.dat – prefer the new file; fall back to level.dat values
        if minecraft_data_dir(level_folder)
            .join("game_rules.dat")
            .exists()
        {
            data.game_rules = read_game_rules(level_folder);
        }

        // world_gen_settings.dat
        if let Some(wgs) = read_world_gen_settings(level_folder) {
            data.world_gen_settings = wgs;
        }

        // world_clocks.dat – read the overworld day_time
        if minecraft_data_dir(level_folder)
            .join("world_clocks.dat")
            .exists()
        {
            let clocks = read_world_clocks(level_folder);
            if let Some(overworld) = clocks.clocks.get("minecraft:overworld") {
                data.day_time = overworld.total_ticks;
            }
        }

        // weather.dat
        if minecraft_data_dir(level_folder)
            .join("weather.dat")
            .exists()
        {
            let weather = read_weather(level_folder);
            data.clear_weather_time = weather.clear_weather_time;
        }

        // (wandering_trader.dat is not part of LevelData; stored separately when needed)

        Ok(data)
    }
}

impl WorldInfoWriter for AnvilLevelInfo {
    fn write_world_info(
        &self,
        info: &LevelData,
        level_folder: &Path,
    ) -> Result<(), WorldInfoError> {
        fs::create_dir_all(level_folder)?;

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let mut level_data = info.clone();
        level_data.last_played = since_the_epoch.as_millis() as i64;
        let level = LevelDat { data: level_data };

        write_level_dat(&level, level_folder)?;

        let data_version = info.data_version;

        // ── Write data/minecraft/*.dat files ─────────────────────────────────

        // game_rules.dat
        if let Err(e) = write_game_rules(level_folder, &info.game_rules, data_version) {
            error!("Failed to write game_rules.dat: {e}");
        }

        // world_gen_settings.dat
        if let Err(e) =
            write_world_gen_settings(level_folder, &info.world_gen_settings, data_version)
        {
            error!("Failed to write world_gen_settings.dat: {e}");
        }

        // world_clocks.dat – persist the overworld day_time; preserve other
        let mut clocks = read_world_clocks(level_folder);
        clocks.data_version = data_version;
        clocks
            .clocks
            .entry("minecraft:overworld".to_string())
            .and_modify(|c| c.total_ticks = info.day_time)
            .or_insert(crate::world_info::data_files::DimensionClock {
                total_ticks: info.day_time,
            });

        if let Err(e) = write_world_clocks(level_folder, &clocks) {
            error!("Failed to write world_clocks.dat: {e}");
        }

        // weather.dat
        let mut weather = read_weather(level_folder);
        weather.clear_weather_time = info.clear_weather_time;
        if let Err(e) = write_weather(level_folder, &weather, data_version) {
            error!("Failed to write weather.dat: {e}");
        }

        // wandering_trader.dat (stub / load-save)
        let wandering_trader = read_wandering_trader(level_folder);
        if let Err(e) = write_wandering_trader(level_folder, &wandering_trader, data_version) {
            error!("Failed to write wandering_trader.dat: {e}");
        }

        // custom_boss_events.dat
        if let Err(e) = write_custom_boss_events_stub(level_folder, data_version) {
            error!("Failed to write custom_boss_events.dat: {e}");
        }

        // scheduled_events.dat
        if let Err(e) = write_scheduled_events_stub(level_folder, data_version) {
            error!("Failed to write scheduled_events.dat: {e}");
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct LevelDat {
    // This tag contains all the level data.
    #[serde(rename = "Data")]
    pub data: LevelData,
}

#[cfg(test)]
mod test {

    use flate2::read::GzDecoder;
    use pumpkin_data::game_rules::GameRuleRegistry;
    use pumpkin_nbt::{deserializer::from_bytes, serializer::to_bytes};
    use pumpkin_util::{Difficulty, world_seed::Seed};
    use std::{
        fs,
        io::{Cursor, Read},
        sync::LazyLock,
    };
    use tempfile::TempDir;

    use crate::{
        global_path,
        world_info::{DataPacks, LevelData, WorldGenSettings, WorldInfoError, WorldVersion},
    };

    use super::{
        AnvilLevelInfo, LEVEL_DAT_BACKUP_FILE_NAME, LEVEL_DAT_FILE_NAME, LevelDat, WorldInfoReader,
        WorldInfoWriter,
    };

    #[test]
    fn preserve_level_dat_seed() {
        let seed = 1337;

        let data = LevelData::default(Seed(1337));

        let temp_dir = TempDir::new().unwrap();
        let world_path = temp_dir.path().join("world");

        AnvilLevelInfo.write_world_info(&data, &world_path).unwrap();

        assert!(world_path.join(LEVEL_DAT_FILE_NAME).is_file());

        let data = AnvilLevelInfo.read_world_info(&world_path).unwrap();

        assert_eq!(data.world_gen_settings.seed, seed);
    }

    #[test]
    fn restores_valid_backup_after_primary_level_dat_is_corrupted() {
        let temp_dir = TempDir::new().unwrap();
        let world_path = temp_dir.path().join("world");

        let mut backup_data = LevelData::default(Seed(1337));
        backup_data.level_name = "backup".to_string();
        let mut primary_data = LevelData::default(Seed(42));
        primary_data.level_name = "primary".to_string();
        AnvilLevelInfo
            .write_world_info(&backup_data, &world_path)
            .unwrap();
        AnvilLevelInfo
            .write_world_info(&primary_data, &world_path)
            .unwrap();

        fs::write(
            world_path.join(LEVEL_DAT_FILE_NAME),
            b"corrupted level data",
        )
        .unwrap();

        let data = AnvilLevelInfo.read_world_info(&world_path).unwrap();

        assert_eq!(data.level_name, "backup");
        assert!(world_path.join("level.dat_corrupted").is_file());
        assert!(world_path.join(LEVEL_DAT_FILE_NAME).is_file());
        assert!(!world_path.join(LEVEL_DAT_BACKUP_FILE_NAME).exists());
    }

    static LEVEL_DAT: LazyLock<LevelDat> = LazyLock::new(|| LevelDat {
        data: LevelData {
            allow_commands: true,
            border_center_x: 0.0,
            border_center_z: 0.0,
            border_damage_per_block: 0.2,
            border_size: 59_999_968.0,
            border_safe_zone: 5.0,
            border_size_lerp_target: 59_999_968.0,
            border_size_lerp_time: 0,
            border_warning_blocks: 5.0,
            border_warning_time: 15.0,
            clear_weather_time: 0,
            data_packs: DataPacks {
                disabled: vec![
                    "minecart_improvements".to_string(),
                    "redstone_experiments".to_string(),
                    "trade_rebalance".to_string(),
                ],
                enabled: vec!["vanilla".to_string()],
            },
            enabled_features: vec!["minecraft:vanilla".to_string()],
            data_version: 4189,
            day_time: 1727,
            difficulty: Difficulty::Normal,
            difficulty_locked: false,
            game_rules: GameRuleRegistry {
                block_explosion_drop_decay: true,
                command_block_output: true,
                drowning_damage: true,
                ender_pearls_vanish_on_death: true,
                fall_damage: true,
                fire_damage: true,
                forgive_dead_players: true,
                freeze_damage: true,
                global_sound_events: true,
                keep_inventory: false,
                lava_source_conversion: false,
                log_admin_commands: true,
                max_entity_cramming: 24,
                mob_explosion_drop_decay: true,
                mob_griefing: true,
                players_nether_portal_creative_delay: 0,
                players_nether_portal_default_delay: 80,
                players_sleeping_percentage: 100,
                projectiles_can_break_blocks: true,
                random_tick_speed: 3,
                reduced_debug_info: false,
                send_command_feedback: true,
                show_death_messages: true,
                spectators_generate_chunks: true,
                tnt_explosion_drop_decay: false,
                universal_anger: false,
                water_source_conversion: true,
                ..Default::default()
            },
            world_gen_settings: WorldGenSettings::new(Seed(1)),
            last_played: 1733847709327,
            level_name: "New World".to_string(),
            spawn_x: 160,
            spawn_y: 70,
            spawn_z: 160,
            spawn_yaw: 0.0,
            spawn_pitch: 0.0,
            level_version: 19133,
            world_version: WorldVersion {
                name: "1.21.4".to_string(),
                id: 4189,
                snapshot: false,
                series: "main".to_string(),
            },
            map_id: 0,
        },
    });

    #[test]
    fn deserialize_level_dat() {
        let raw_compressed_nbt = fs::read("assets/level_1_21_4.dat").unwrap();
        assert!(!raw_compressed_nbt.is_empty());

        let mut decoder = GzDecoder::new(&raw_compressed_nbt[..]);
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf).unwrap();
        let level_dat: LevelDat = from_bytes(Cursor::new(buf)).expect("Failed to decode from file");

        assert_eq!(level_dat, *LEVEL_DAT);
    }

    #[test]
    fn serialize_level_dat() {
        let mut serialized = Vec::new();
        to_bytes(&*LEVEL_DAT, &mut serialized).expect("Failed to encode to bytes");

        assert!(!serialized.is_empty());

        let level_dat_again: LevelDat =
            from_bytes(Cursor::new(serialized)).expect("Failed to decode from bytes");

        let mut expected = (*LEVEL_DAT).clone();
        expected.data.game_rules = GameRuleRegistry::default();
        expected.data.world_gen_settings = WorldGenSettings::default();
        expected.data.day_time = 0;
        expected.data.clear_weather_time = 0;

        assert_eq!(level_dat_again, expected);
    }

    #[test]
    fn round_trips_enabled_features() {
        let mut level_dat = (*LEVEL_DAT).clone();
        level_dat.data.enabled_features = vec![
            "minecraft:vanilla".to_string(),
            "minecraft:trade_rebalance".to_string(),
        ];

        let mut serialized = Vec::new();
        to_bytes(&level_dat, &mut serialized).expect("Failed to encode level.dat");

        let decoded: LevelDat =
            from_bytes(Cursor::new(serialized)).expect("Failed to decode level.dat");

        assert_eq!(
            decoded.data.enabled_features,
            level_dat.data.enabled_features
        );
    }

    #[test]
    fn failed_deserialize_old_level_dat() {
        let temp_dir = TempDir::new().unwrap();

        let test_dat = global_path!("../../assets/level_1_20.dat");
        fs::copy(
            test_dat,
            temp_dir.path().to_path_buf().join(LEVEL_DAT_FILE_NAME),
        )
        .unwrap();

        let result = AnvilLevelInfo.read_world_info(temp_dir.path());
        assert!(matches!(
            result,
            Err(WorldInfoError::UnsupportedDataVersion(_))
        ));
    }
}
