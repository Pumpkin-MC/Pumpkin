use std::{
    fs::OpenOptions,
    io::Read,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::{Deserialize, Serialize};

use crate::world_info::{
    MAXIMUM_SUPPORTED_WORLD_DATA_VERSION, MINIMUM_SUPPORTED_WORLD_DATA_VERSION,
};

use super::{LevelData, WorldInfoError, WorldInfoReader, WorldInfoWriter};

pub const LEVEL_DAT_FILE_NAME: &str = "level.dat";
pub const LEVEL_DAT_BACKUP_FILE_NAME: &str = "level.dat_old";

pub struct AnvilLevelInfo;

fn check_file_data_version(raw_nbt: &[u8]) -> Result<(), WorldInfoError> {
    // Define a struct that only has the data version. This is necessary because if a user tries to
    // load a world with different data, they will get a generic "Failed to deserialize level.dat error".
    // When only checking for the data version, we can determine if we can support the full
    // deserializiation before going through with it.
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct LevelData {
        #[allow(dead_code)]
        data_version: i32,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct LevelDat {
        #[allow(dead_code)]
        data: LevelData,
    }

    let info: LevelDat = pumpkin_nbt::from_bytes(raw_nbt)
        .map_err(|e|{
            log::error!("The world.dat file does not have a data version! This means it is either corrupt or very old (read unsupported)");
            WorldInfoError::DeserializationError(e.to_string())})?;

    let data_version = info.data.data_version;

    if !(MINIMUM_SUPPORTED_WORLD_DATA_VERSION..=MAXIMUM_SUPPORTED_WORLD_DATA_VERSION)
        .contains(&data_version)
    {
        Err(WorldInfoError::UnsupportedVersion(data_version))
    } else {
        Ok(())
    }
}

impl WorldInfoReader for AnvilLevelInfo {
    fn read_world_info(&self, level_folder: &Path) -> Result<LevelData, WorldInfoError> {
        let path = level_folder.join(LEVEL_DAT_FILE_NAME);

        let world_info_file = OpenOptions::new().read(true).open(path)?;
        let mut compression_reader = GzDecoder::new(world_info_file);
        let mut buf = Vec::new();
        let _ = compression_reader.read_to_end(&mut buf)?;

        check_file_data_version(&buf)?;
        let info = pumpkin_nbt::from_bytes::<LevelDat>(&buf[..])
            .map_err(|e| WorldInfoError::DeserializationError(e.to_string()))?;

        // TODO: check version

        Ok(info.data)
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
        let level = LevelDat {
            data: level_data.clone(),
        };

        // open file
        let path = level_folder.join(LEVEL_DAT_FILE_NAME);
        let world_info_file = OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(path)?;

        // write compressed data into file
        let compression_writer = GzEncoder::new(world_info_file, Compression::best());
        // TODO: Proper error handling
        pumpkin_nbt::to_bytes(&level, compression_writer)
            .expect("Failed to write level.dat to disk");
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct LevelDat {
    // This tag contains all the level data.
    #[serde(rename = "Data")]
    pub data: LevelData,
}

#[cfg(test)]
mod test {

    use std::{fs, sync::LazyLock};

    use flate2::read::GzDecoder;
    use pumpkin_data::game_rules::GameRuleRegistry;
    use pumpkin_nbt::{deserializer::from_bytes, serializer::to_bytes};
    use pumpkin_util::Difficulty;
    use temp_dir::TempDir;

    use crate::{
        global_path,
        world_info::{DataPacks, LevelData, WorldGenSettings, WorldInfoError, WorldVersion},
    };

    use super::{AnvilLevelInfo, LEVEL_DAT_FILE_NAME, LevelDat, WorldInfoReader, WorldInfoWriter};

    #[test]
    fn test_preserve_level_dat_seed() {
        let seed = 1337;

        let mut data = LevelData::default();
        data.world_gen_settings.seed = seed;

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
                announce_advancements: true,
                block_explosion_drop_decay: true,
                command_block_output: true,
                command_modification_block_limit: 32768,
                disable_elytra_movement_check: false,
                disable_player_movement_check: false,
                disable_raids: false,
                do_daylight_cycle: true,
                do_entity_drops: true,
                do_fire_tick: true,
                do_immediate_respawn: false,
                do_insomnia: true,
                do_limited_crafting: false,
                do_mob_loot: true,
                do_mob_spawning: true,
                do_patrol_spawning: true,
                do_tile_drops: true,
                do_trader_spawning: true,
                do_vines_spread: true,
                do_warden_spawning: true,
                do_weather_cycle: true,
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
                max_command_chain_length: 65536,
                max_command_fork_count: 65536,
                max_entity_cramming: 24,
                mob_explosion_drop_decay: true,
                mob_griefing: true,
                natural_regeneration: true,
                players_nether_portal_creative_delay: 0,
                players_nether_portal_default_delay: 80,
                players_sleeping_percentage: 100,
                projectiles_can_break_blocks: true,
                random_tick_speed: 3,
                reduced_debug_info: false,
                send_command_feedback: true,
                show_death_messages: true,
                snow_accumulation_height: 1,
                spawn_chunk_radius: 2,
                spawn_radius: 10,
                spectators_generate_chunks: true,
                tnt_explosion_drop_decay: false,
                universal_anger: false,
                water_source_conversion: true,
                ..Default::default()
            },
            world_gen_settings: WorldGenSettings { seed: 1 },
            last_played: 1733847709327,
            level_name: "New World".to_string(),
            spawn_x: 160,
            spawn_y: 70,
            spawn_z: 160,
            spawn_angle: 0.0,
            nbt_version: 19133,
            version: WorldVersion {
                name: "1.21.4".to_string(),
                id: 4189,
                snapshot: false,
                series: "main".to_string(),
            },
        },
    });

    #[test]
    fn test_deserialize_level_dat() {
        let raw_compressed_nbt = include_bytes!("../../assets/level_1_21_4.dat");
        assert!(!raw_compressed_nbt.is_empty());

        let decoder = GzDecoder::new(&raw_compressed_nbt[..]);
        let level_dat: LevelDat = from_bytes(decoder).expect("Failed to decode from file");

        assert_eq!(level_dat, *LEVEL_DAT);
    }

    #[test]
    fn test_serialize_level_dat() {
        let mut serialized = Vec::new();
        to_bytes(&*LEVEL_DAT, &mut serialized).expect("Failed to encode to bytes");

        assert!(!serialized.is_empty());

        let level_dat_again: LevelDat =
            from_bytes(&serialized[..]).expect("Failed to decode from bytes");

        assert_eq!(level_dat_again, *LEVEL_DAT);
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
        match result {
            Ok(_) => panic!("This should fail!"),
            Err(WorldInfoError::UnsupportedVersion(_)) => {}
            Err(_) => panic!("Wrong error!"),
        }
    }
}
