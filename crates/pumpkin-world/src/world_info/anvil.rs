use std::{
    fs::File,
    io::{Cursor, Read},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use tracing::error;

use flate2::read::GzDecoder;
use num_traits::FromPrimitive;
use pumpkin_util::Difficulty;
use serde::{Deserialize, Serialize};

use crate::world_info::{
    MAXIMUM_SUPPORTED_LEVEL_VERSION, MAXIMUM_SUPPORTED_WORLD_DATA_VERSION,
    MINIMUM_SUPPORTED_LEVEL_VERSION, MINIMUM_SUPPORTED_WORLD_DATA_VERSION,
    data_files::{
        minecraft_data_dir, read_game_rules, read_scoreboard, read_wandering_trader, read_weather,
        read_world_clocks, read_world_gen_settings, write_custom_boss_events_stub,
        write_game_rules, write_scheduled_events_stub, write_scoreboard, write_wandering_trader,
        write_weather, write_world_clocks, write_world_gen_settings,
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

/// Reads the nested `spawn` compound (`LevelData.RespawnData.CODEC`, see
/// `net/minecraft/world/level/storage/PrimaryLevelData.java:126,139` and the record at
/// `net/minecraft/world/level/storage/LevelData.java:33-43`): a `GlobalPos` (`dimension`
/// string + `pos` 3-element int array, `net/minecraft/core/GlobalPos.java:13-15`) plus
/// float `yaw`/`pitch`.
///
/// Falls back to the legacy flat `SpawnX`/`SpawnY`/`SpawnZ`/`SpawnAngle`/`SpawnPitch` keys
/// that older Pumpkin builds wrote, so worlds saved by those builds still load correctly.
/// Split out of `read_world_info` to keep that function under the workspace line limit.
fn read_spawn(data: &pumpkin_nbt::compound::NbtCompound, level_data: &mut LevelData) {
    if let Some(spawn) = data.get_compound("spawn") {
        if let Some(pos) = spawn.get_int_array("pos")
            && let [x, y, z] = *pos
        {
            level_data.spawn_x = x;
            level_data.spawn_y = y;
            level_data.spawn_z = z;
        }
        if let Some(v) = spawn.get_float("yaw") {
            level_data.spawn_yaw = v;
        }
        if let Some(v) = spawn.get_float("pitch") {
            level_data.spawn_pitch = v;
        }
        return;
    }

    // Legacy Pumpkin schema (commit 76265bef), migrated on next save.
    if let Some(v) = data.get_int("SpawnX") {
        level_data.spawn_x = v;
    }
    if let Some(v) = data.get_int("SpawnY") {
        level_data.spawn_y = v;
    }
    if let Some(v) = data.get_int("SpawnZ") {
        level_data.spawn_z = v;
    }
    if let Some(v) = data.get_float("SpawnAngle") {
        level_data.spawn_yaw = v;
    }
    if let Some(v) = data.get_float("SpawnPitch") {
        level_data.spawn_pitch = v;
    }
}

/// Reads the nested `difficulty_settings` compound
/// (`LevelSettings.DifficultySettings.CODEC`, `net/minecraft/world/level/LevelSettings.java:58-67`):
/// `difficulty` as a lowercase string (`Difficulty.CODEC` is `StringRepresentable.fromEnum`,
/// `net/minecraft/world/Difficulty.java:12-18`), plus `hardcore` and `locked` booleans.
///
/// Falls back to the legacy flat root-level `Difficulty` byte / `DifficultyLocked` bool
/// that older Pumpkin builds wrote, so worlds saved by those builds still load correctly.
/// Pumpkin does not track hardcore mode in `LevelData`, so `hardcore` is read but discarded.
/// Split out of `read_world_info` to keep that function under the workspace line limit.
fn read_difficulty(data: &pumpkin_nbt::compound::NbtCompound, level_data: &mut LevelData) {
    if let Some(settings) = data.get_compound("difficulty_settings") {
        if let Some(v) = settings
            .get_string("difficulty")
            .and_then(|s| s.parse::<Difficulty>().ok())
        {
            level_data.difficulty = v;
        }
        if let Some(v) = settings.get_bool("locked") {
            level_data.difficulty_locked = v;
        }
        return;
    }

    // Legacy Pumpkin schema (commit 76265bef), migrated on next save.
    if let Some(v) = data.get_byte("Difficulty").and_then(Difficulty::from_i8) {
        level_data.difficulty = v;
    }
    if let Some(v) = data.get_bool("DifficultyLocked") {
        level_data.difficulty_locked = v;
    }
}

/// Reads the `DataPacks` sub-compound of level.dat's `Data` compound.
/// Split out of `read_world_info` to keep that function under the workspace line limit.
fn read_data_packs(data: &pumpkin_nbt::compound::NbtCompound, level_data: &mut LevelData) {
    let Some(packs) = data.get_compound("DataPacks") else {
        return;
    };
    let strings = |list: &[pumpkin_nbt::tag::NbtTag]| -> Vec<String> {
        list.iter()
            .filter_map(|t| match t {
                pumpkin_nbt::tag::NbtTag::String(s) => Some(s.to_string()),
                _ => None,
            })
            .collect()
    };
    if let Some(list) = packs.get_list("Disabled") {
        level_data.data_packs.disabled = strings(list);
    }
    if let Some(list) = packs.get_list("Enabled") {
        level_data.data_packs.enabled = strings(list);
    }
}

/// Builds the `DataPacks` sub-compound for level.dat.
/// Split out of `write_world_info` to keep that function under the workspace line limit.
fn write_data_packs(level_data: &LevelData) -> pumpkin_nbt::compound::NbtCompound {
    let as_tags = |v: &[String]| -> Vec<pumpkin_nbt::tag::NbtTag> {
        v.iter()
            .map(|s| pumpkin_nbt::tag::NbtTag::String(s.clone().into_boxed_str()))
            .collect()
    };
    let mut comp = pumpkin_nbt::compound::NbtCompound::new();
    comp.put_list("Disabled", as_tags(&level_data.data_packs.disabled));
    comp.put_list("Enabled", as_tags(&level_data.data_packs.enabled));
    comp
}

/// Builds the nested `spawn` compound. See [`read_spawn`] for the schema citation.
/// Pumpkin does not track a per-spawn dimension, so `dimension` is always written as the
/// overworld, matching the "minecraft:overworld" literal used elsewhere in this module.
fn write_spawn(level_data: &LevelData) -> pumpkin_nbt::compound::NbtCompound {
    let mut spawn = pumpkin_nbt::compound::NbtCompound::new();
    spawn.put_string("dimension", "minecraft:overworld".to_string());
    spawn.put(
        "pos",
        pumpkin_nbt::tag::NbtTag::IntArray(vec![
            level_data.spawn_x,
            level_data.spawn_y,
            level_data.spawn_z,
        ]),
    );
    spawn.put_float("yaw", level_data.spawn_yaw);
    spawn.put_float("pitch", level_data.spawn_pitch);
    spawn
}

/// Builds the nested `difficulty_settings` compound. See [`read_difficulty`] for the
/// schema citation. Pumpkin does not track hardcore mode in `LevelData`, so `hardcore` is
/// always written as `false`.
fn write_difficulty(level_data: &LevelData) -> pumpkin_nbt::compound::NbtCompound {
    let mut settings = pumpkin_nbt::compound::NbtCompound::new();
    settings.put_string("difficulty", level_data.difficulty.name().to_string());
    settings.put_bool("hardcore", false);
    settings.put_bool("locked", level_data.difficulty_locked);
    settings
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

        // Fields still stored directly in level.dat's "Data" compound (spawn point,
        // difficulty, allowCommands, level name, data packs). Everything else has moved to
        // the per-type save files under data/minecraft/*.dat, handled below. `WorldBorder`
        // and `map_id` are no longer read from level.dat at all -- vanilla keeps those in
        // separate SavedData files ("world_border" and "maps/last_id" respectively; see
        // net/minecraft/world/level/border/WorldBorder.java:24-26 and
        // net/minecraft/world/level/saveddata/maps/MapIndex.java:12-16) that Pumpkin does
        // not yet write, so those values stay at LevelData::default() until that lands.
        // Missing keys otherwise fall back to LevelData::default()'s values.
        {
            let mut cursor = Cursor::new(&buf);
            let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
                pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
            );
            if let Ok(nbt) = pumpkin_nbt::Nbt::read(&mut reader)
                && let Some(data) = nbt.root_tag.get_compound("Data")
            {
                if let Some(v) = data.get_bool("allowCommands") {
                    level_data.allow_commands = v;
                }
                if let Some(v) = data.get_string("LevelName") {
                    level_data.level_name = v.to_string();
                }
                read_spawn(data, &mut level_data);
                read_difficulty(data, &mut level_data);
                read_data_packs(data, &mut level_data);
            }
        }

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
            level_data.rain_time = weather.rain_time;
            level_data.raining = weather.raining;
            level_data.thundering = weather.thundering;
            level_data.thunder_time = weather.thunder_time;
        }

        // world_gen_settings.dat
        if minecraft_data_dir(level_folder)
            .join("world_gen_settings.dat")
            .exists()
            && let Some(wgs) = read_world_gen_settings(level_folder)
        {
            level_data.world_gen_settings = wgs;
        }

        // scoreboard.dat
        level_data.scoreboard_data = read_scoreboard(level_folder);

        // (wandering_trader.dat is not part of LevelData; stored separately when needed)

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
        data_comp.put_bool("allowCommands", level_data.allow_commands);
        data_comp.put_string("LevelName", level_data.level_name.clone());
        data_comp.put_compound("difficulty_settings", write_difficulty(&level_data));
        data_comp.put_compound("spawn", write_spawn(&level_data));
        // `WorldBorder` and `map_id` are deliberately not written: vanilla keeps them in
        // separate SavedData files, not level.dat (see the comment in read_world_info).

        data_comp.put_compound("DataPacks", write_data_packs(&level_data));

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
        let weather = crate::world_info::data_files::WeatherData {
            rain_time: info.rain_time,
            raining: info.raining,
            thundering: info.thundering,
            thunder_time: info.thunder_time,
            clear_weather_time: info.clear_weather_time,
            data_version,
        };
        if let Err(e) = write_weather(level_folder, &weather) {
            error!("Failed to write weather.dat: {e}");
        }

        // scoreboard.dat
        if let Err(e) = write_scoreboard(level_folder, &info.scoreboard_data) {
            error!("Failed to write scoreboard.dat: {e}");
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
    use crate::world_info::data_files::ScoreboardData;

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
            rain_time: 0,
            raining: false,
            thundering: false,
            thunder_time: 0,
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
            scoreboard_data: ScoreboardData::default(),
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

    #[test]
    fn preserves_world_clock_and_weather_data() {
        let temp_dir = TempDir::new().unwrap();
        let mut data = LevelData::default(Seed(42));
        data.day_time = 36_001;
        data.clear_weather_time = 1_200;
        data.rain_time = 9_001;
        data.raining = true;
        data.thundering = true;
        data.thunder_time = 4_501;

        AnvilLevelInfo
            .write_world_info(&data, temp_dir.path())
            .unwrap();

        let loaded = AnvilLevelInfo.read_world_info(temp_dir.path()).unwrap();
        assert_eq!(loaded.day_time, data.day_time);
        assert_eq!(loaded.clear_weather_time, data.clear_weather_time);
        assert_eq!(loaded.rain_time, data.rain_time);
        assert_eq!(loaded.raining, data.raining);
        assert_eq!(loaded.thundering, data.thundering);
        assert_eq!(loaded.thunder_time, data.thunder_time);
    }
}
