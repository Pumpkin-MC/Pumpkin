use std::{
    fs::File,
    io::{Cursor, Read},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::error;

use flate2::read::GzDecoder;
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

fn check_file_data_version(raw_nbt: &[u8]) -> Result<(), WorldInfoError> {
    let mut cursor = Cursor::new(raw_nbt);
    let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
        pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
    );
    let nbt = pumpkin_nbt::Nbt::read(&mut reader)
        .map_err(|e| WorldInfoError::DeserializationError(e.to_string()))?;
    let data_version = nbt
        .get_compound("Data")
        .and_then(|c| c.get_int("DataVersion"));

    let Some(data_version) = data_version else {
        error!(
            "The level.dat file does not have a data version! This means it is either corrupt or very old (read unsupported)"
        );
        return Err(WorldInfoError::DeserializationError(
            "Missing DataVersion".into(),
        ));
    };

    if (MINIMUM_SUPPORTED_WORLD_DATA_VERSION..=MAXIMUM_SUPPORTED_WORLD_DATA_VERSION)
        .contains(&data_version)
    {
        Ok(())
    } else {
        Err(WorldInfoError::UnsupportedDataVersion(data_version))
    }
}

fn check_file_level_version(raw_nbt: &[u8]) -> Result<(), WorldInfoError> {
    let mut cursor = Cursor::new(raw_nbt);
    let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
        pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
    );
    let nbt = pumpkin_nbt::Nbt::read(&mut reader)
        .map_err(|e| WorldInfoError::DeserializationError(e.to_string()))?;
    let level_version = nbt.get_compound("Data").and_then(|c| c.get_int("version"));

    let Some(level_version) = level_version else {
        error!(
            "The level.dat file does not have a level version! This means it is either corrupt or very old (read unsupported)"
        );
        return Err(WorldInfoError::DeserializationError(
            "Missing version".into(),
        ));
    };

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

        let world_info_file = File::open(path)?;
        let mut buf = Vec::new();
        GzDecoder::new(world_info_file).read_to_end(&mut buf)?;

        check_file_data_version(&buf)?;
        check_file_level_version(&buf)?;

        // For now, construct a default LevelData or parse manually
        let mut level_data = LevelData::default(pumpkin_util::world_seed::Seed(0));

        // game_rules.dat – prefer the new file; fall back to level.dat values
        if minecraft_data_dir(level_folder)
            .join("game_rules.dat")
            .exists()
        {
            level_data.game_rules = read_game_rules(level_folder);
        }

        if minecraft_data_dir(level_folder)
            .join("world_clocks.dat")
            .exists()
        {
            let clocks = read_world_clocks(level_folder);
            if let Some(overworld) = clocks.clocks.get("minecraft:overworld") {
                level_data.day_time = overworld.total_ticks;
            }
        }

        // weather.dat
        if minecraft_data_dir(level_folder)
            .join("weather.dat")
            .exists()
        {
            let weather = read_weather(level_folder);
            level_data.clear_weather_time = weather.clear_weather_time;
        }

        // world_gen_settings.dat
        if minecraft_data_dir(level_folder)
            .join("world_gen_settings.dat")
            .exists()
            && let Some(wgs) = read_world_gen_settings(level_folder)
        {
            level_data.world_gen_settings = wgs;
        }

        Ok(level_data)
    }
}

impl WorldInfoWriter for AnvilLevelInfo {
    fn write_world_info(
        &self,
        info: &LevelData,
        level_folder: &Path,
    ) -> Result<(), WorldInfoError> {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let mut level_data = info.clone();
        level_data.last_played = since_the_epoch.as_millis() as i64;

        // ── Write level.dat ───────────────────────────────────────────────────
        let path = level_folder.join(LEVEL_DAT_FILE_NAME);
        let world_info_file = File::create(path)?;

        let mut data_comp = pumpkin_nbt::compound::NbtCompound::new();
        data_comp.put_int("DataVersion", level_data.data_version);
        data_comp.put_int("version", MAXIMUM_SUPPORTED_LEVEL_VERSION);
        data_comp.put_long("LastPlayed", level_data.last_played);

        let mut root = pumpkin_nbt::compound::NbtCompound::new();
        root.put_compound("Data", data_comp);

        pumpkin_nbt::nbt_compress::write_gzip_compound_tag(root, world_info_file)
            .map_err(|e| WorldInfoError::SerializationError(e.to_string()))?;

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
        weather.data_version = data_version;
        if let Err(e) = write_weather(level_folder, &weather) {
            error!("Failed to write weather.dat: {e}");
        }

        // wandering_trader.dat (stub / load-save)
        let mut wandering_trader = read_wandering_trader(level_folder);
        wandering_trader.data_version = data_version;
        if let Err(e) = write_wandering_trader(level_folder, &wandering_trader) {
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

    use pumpkin_data::game_rules::GameRuleRegistry;
    use pumpkin_util::{Difficulty, world_seed::Seed};
    use std::sync::LazyLock;
    use tempfile::TempDir;

    use crate::world_info::{DataPacks, LevelData, WorldGenSettings, WorldVersion};

    use super::{AnvilLevelInfo, LevelDat, WorldInfoReader, WorldInfoWriter};

    #[test]
    fn preserve_level_dat_seed() {
        let seed = 1337;

        let data = LevelData::default(Seed(1337));

        let temp_dir = TempDir::new().unwrap();

        AnvilLevelInfo
            .write_world_info(&data, temp_dir.path())
            .unwrap();

        let data = AnvilLevelInfo.read_world_info(temp_dir.path()).unwrap();

        assert_eq!(data.world_gen_settings.seed, seed);
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

    // #[test]
    // fn deserialize_level_dat() {
    //     let raw_compressed_nbt = fs::read("assets/level_1_21_4.dat").unwrap();
    //     assert!(!raw_compressed_nbt.is_empty());

    //     let mut decoder = GzDecoder::new(&raw_compressed_nbt[..]);
    //     let mut buf = Vec::new();
    //     decoder.read_to_end(&mut buf).unwrap();
    //     let level_dat: LevelDat = from_bytes(Cursor::new(buf)).expect("Failed to decode from file");

    //     assert_eq!(level_dat, *LEVEL_DAT);
    // }

    #[test]
    fn serialize_level_dat() {
        let mut data_comp = pumpkin_nbt::compound::NbtCompound::new();
        data_comp.put_int("DataVersion", LEVEL_DAT.data.data_version);
        data_comp.put_long("LastPlayed", LEVEL_DAT.data.last_played);

        let mut root = pumpkin_nbt::compound::NbtCompound::new();
        root.put_compound("Data", data_comp);

        let bytes = pumpkin_nbt::Nbt::from(root).write();
        assert!(!bytes.is_empty());
    }
}
