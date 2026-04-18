use std::io::{Cursor, Read};

use async_trait::async_trait;
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::Deserialize;
use tokio::fs;

use crate::error::StorageError;
use crate::level_info::{
    LevelData, LevelInfoStorage, MAXIMUM_SUPPORTED_LEVEL_VERSION,
    MAXIMUM_SUPPORTED_WORLD_DATA_VERSION, MINIMUM_SUPPORTED_LEVEL_VERSION,
    MINIMUM_SUPPORTED_WORLD_DATA_VERSION,
};
use crate::vanilla::VanillaStorage;

pub const LEVEL_DAT_FILE_NAME: &str = "level.dat";
pub const LEVEL_DAT_BACKUP_FILE_NAME: &str = "level.dat_old";

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct LevelDat {
    #[serde(rename = "Data")]
    data: LevelData,
}

fn check_data_version(raw_nbt: &[u8]) -> Result<(), StorageError> {
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct DataVersionOnly {
        data_version: i32,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Wrapper {
        data: DataVersionOnly,
    }

    let info: Wrapper = pumpkin_nbt::from_bytes(Cursor::new(raw_nbt))
        .map_err(|e| StorageError::Deserialize(e.to_string()))?;
    let v = info.data.data_version;
    if (MINIMUM_SUPPORTED_WORLD_DATA_VERSION..=MAXIMUM_SUPPORTED_WORLD_DATA_VERSION).contains(&v) {
        Ok(())
    } else {
        Err(StorageError::UnsupportedVersion(format!(
            "world data version {v} out of supported range"
        )))
    }
}

fn check_level_version(raw_nbt: &[u8]) -> Result<(), StorageError> {
    #[derive(Deserialize)]
    struct LevelVersionOnly {
        version: i32,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Wrapper {
        data: LevelVersionOnly,
    }

    let info: Wrapper = pumpkin_nbt::from_bytes(Cursor::new(raw_nbt))
        .map_err(|e| StorageError::Deserialize(e.to_string()))?;
    let v = info.data.version;
    if (MINIMUM_SUPPORTED_LEVEL_VERSION..=MAXIMUM_SUPPORTED_LEVEL_VERSION).contains(&v) {
        Ok(())
    } else {
        Err(StorageError::UnsupportedVersion(format!(
            "level version {v} out of supported range"
        )))
    }
}

#[async_trait]
impl LevelInfoStorage for VanillaStorage {
    async fn load(&self) -> Result<LevelData, StorageError> {
        let path = self.base_dir().join(LEVEL_DAT_FILE_NAME);
        let compressed = fs::read(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                StorageError::NotFound {
                    message: format!("level.dat not found at {}", path.display()),
                }
            } else {
                StorageError::io_at(&path, e)
            }
        })?;

        let mut buf = Vec::new();
        GzDecoder::new(Cursor::new(compressed))
            .read_to_end(&mut buf)
            .map_err(StorageError::io)?;

        check_data_version(&buf)?;
        check_level_version(&buf)?;

        let dat: LevelDat = pumpkin_nbt::from_bytes(Cursor::new(buf))
            .map_err(|e| StorageError::Deserialize(e.to_string()))?;

        let backup = self.base_dir().join(LEVEL_DAT_BACKUP_FILE_NAME);
        if let Err(e) = fs::copy(&path, &backup).await {
            // Backup is best-effort; surface as a hard error only if the source
            // disappeared between read and copy (unlikely).
            if e.kind() != std::io::ErrorKind::NotFound {
                return Err(StorageError::io_at(&backup, e));
            }
        }

        Ok(dat.data)
    }

    async fn save(&self, data: &LevelData) -> Result<(), StorageError> {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        let mut stamped = data.clone();
        stamped.last_played = now_ms;
        let dat = LevelDat { data: stamped };

        let mut compressed = Vec::new();
        {
            let mut encoder = GzEncoder::new(&mut compressed, Compression::best());
            pumpkin_nbt::to_bytes(&dat, &mut encoder)
                .map_err(|e| StorageError::Serialize(e.to_string()))?;
            encoder.finish().map_err(StorageError::io)?;
        }

        let path = self.base_dir().join(LEVEL_DAT_FILE_NAME);
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::io_at(parent, e))?;
        }
        fs::write(&path, &compressed)
            .await
            .map_err(|e| StorageError::io_at(&path, e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};
    use std::sync::LazyLock;

    use flate2::read::GzDecoder;
    use pumpkin_data::game_rules::GameRuleRegistry;
    use pumpkin_nbt::{deserializer::from_bytes, serializer::to_bytes};
    use pumpkin_util::{Difficulty, world_seed::Seed};
    use temp_dir::TempDir;

    use crate::error::StorageError;
    use crate::level_info::{
        DataPacks, LevelData, LevelInfoStorage, WorldGenSettings, WorldVersion,
    };
    use crate::vanilla::VanillaStorage;

    use super::{LEVEL_DAT_FILE_NAME, LevelDat};

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
        },
    });

    #[test]
    fn deserialize_level_dat() {
        let raw_compressed_nbt = std::fs::read("assets/level_1_21_4.dat").unwrap();
        assert!(!raw_compressed_nbt.is_empty());

        let mut decoder = GzDecoder::new(&raw_compressed_nbt[..]);
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf).unwrap();
        let level_dat: LevelDat =
            from_bytes(Cursor::new(buf)).expect("Failed to decode from file");

        assert_eq!(level_dat, *LEVEL_DAT);
    }

    #[test]
    fn serialize_level_dat() {
        let mut serialized = Vec::new();
        to_bytes(&*LEVEL_DAT, &mut serialized).expect("Failed to encode to bytes");

        assert!(!serialized.is_empty());

        let round_tripped: LevelDat =
            from_bytes(Cursor::new(serialized)).expect("Failed to decode from bytes");

        assert_eq!(round_tripped, *LEVEL_DAT);
    }

    #[tokio::test]
    async fn failed_deserialize_old_level_dat() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::copy(
            "assets/level_1_20.dat",
            temp_dir.path().join(LEVEL_DAT_FILE_NAME),
        )
        .unwrap();

        let store = VanillaStorage::new(temp_dir.path());
        let err = store.load().await.unwrap_err();
        assert!(
            matches!(err, StorageError::UnsupportedVersion(_)),
            "unexpected error: {err}"
        );
    }
}
