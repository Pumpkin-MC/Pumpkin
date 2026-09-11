use std::{
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
};

use pumpkin_data::game_rules::{GameRule, GameRuleRegistry, GameRuleValue};
use pumpkin_nbt::{compound::NbtCompound, nbt_compress::read_gzip_compound_tag, tag::NbtTag};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::world_info::{WorldGenSettings, WorldInfoError};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct DataFileRoot<T> {
    #[serde(rename = "data")]
    pub data: T,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct WeatherData {
    #[serde(rename = "rain_time", default)]
    pub rain_time: i32,
    #[serde(rename = "raining", default)]
    pub raining: bool,
    #[serde(rename = "thundering", default)]
    pub thundering: bool,
    #[serde(rename = "thunder_time", default)]
    pub thunder_time: i32,
    #[serde(rename = "clear_weather_time", default)]
    pub clear_weather_time: i32,
    #[serde(rename = "DataVersion", default)]
    pub data_version: i32,
}

impl Default for WeatherData {
    fn default() -> Self {
        Self {
            rain_time: 0,
            raining: false,
            thundering: false,
            thunder_time: 0,
            clear_weather_time: -1,
            data_version: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct WorldGenSettingsData {
    #[serde(flatten)]
    pub settings: WorldGenSettings,
    #[serde(rename = "DataVersion", default)]
    pub data_version: i32,
    #[serde(rename = "bonus_chest", default)]
    pub bonus_chest: bool,
    #[serde(rename = "generate_structures", default = "default_true")]
    pub generate_structures: bool,
}

const fn default_true() -> bool {
    true
}

impl WorldGenSettingsData {
    #[must_use]
    pub const fn new(settings: WorldGenSettings, data_version: i32) -> Self {
        Self {
            settings,
            data_version,
            bonus_chest: false,
            generate_structures: true,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct DimensionClock {
    pub total_ticks: i64,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct WorldClocksData {
    pub clocks: std::collections::HashMap<String, DimensionClock>,
    pub data_version: i32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct WanderingTraderData {
    #[serde(rename = "spawn_delay", default = "default_wandering_trader_delay")]
    pub spawn_delay: i32,
    #[serde(rename = "spawn_chance", default = "default_wandering_trader_chance")]
    pub spawn_chance: i32,
    #[serde(rename = "DataVersion", default)]
    pub data_version: i32,
}

const fn default_wandering_trader_delay() -> i32 {
    24_000
}
const fn default_wandering_trader_chance() -> i32 {
    25
}

impl Default for WanderingTraderData {
    fn default() -> Self {
        Self {
            spawn_delay: default_wandering_trader_delay(),
            spawn_chance: default_wandering_trader_chance(),
            data_version: 0,
        }
    }
}

#[must_use]
pub fn minecraft_data_dir(level_folder: &Path) -> PathBuf {
    level_folder.join("data").join("minecraft")
}

/// Ensures the `<world>/data/minecraft/` directory exists.
pub fn ensure_minecraft_data_dir(level_folder: &Path) -> Result<PathBuf, WorldInfoError> {
    let dir = minecraft_data_dir(level_folder);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn read_weather(level_folder: &Path) -> WeatherData {
    let path = minecraft_data_dir(level_folder).join("weather.dat");
    if !path.exists() {
        return WeatherData::default();
    }
    match File::open(&path) {
        Ok(f) => match read_gzip_compound_tag(f) {
            Ok(compound) => {
                let data_compound = compound.get_compound("data");
                let c = data_compound.as_ref().map_or(&compound, |v| v);
                WeatherData {
                    clear_weather_time: c.get_int("clear_weather_time").unwrap_or(0),
                    rain_time: c.get_int("rain_time").unwrap_or(0),
                    thunder_time: c.get_int("thunder_time").unwrap_or(0),
                    raining: c.get_bool("raining").unwrap_or(false),
                    thundering: c.get_bool("thundering").unwrap_or(false),
                    data_version: c.get_int("DataVersion").unwrap_or(0),
                }
            }
            Err(e) => {
                warn!("Failed to deserialize weather.dat, using defaults: {e}");
                WeatherData::default()
            }
        },
        Err(e) => {
            warn!("Failed to open weather.dat, using defaults: {e}");
            WeatherData::default()
        }
    }
}

pub fn write_weather(level_folder: &Path, data: &WeatherData) -> Result<(), WorldInfoError> {
    let dir = ensure_minecraft_data_dir(level_folder)?;
    let path = dir.join("weather.dat");
    let file = File::create(&path)?;
    let mut data_comp = NbtCompound::new();
    data_comp.put_int("clear_weather_time", data.clear_weather_time);
    data_comp.put_int("rain_time", data.rain_time);
    data_comp.put_int("thunder_time", data.thunder_time);
    data_comp.put_bool("raining", data.raining);
    data_comp.put_bool("thundering", data.thundering);
    let mut root = NbtCompound::new();
    root.put_int("DataVersion", data.data_version);
    root.put_compound("data", data_comp);
    pumpkin_nbt::nbt_compress::write_gzip_compound_tag(root, BufWriter::new(file))
        .map_err(|e| WorldInfoError::SerializationError(e.to_string()))
}

#[must_use]
pub fn json_to_nbt_tag(val: &serde_json::Value) -> NbtTag {
    match val {
        serde_json::Value::Null => NbtTag::End,
        serde_json::Value::Bool(b) => NbtTag::Byte(i8::from(*b)),
        serde_json::Value::Number(n) => n.as_i64().map_or_else(
            || n.as_f64().map_or(NbtTag::End, NbtTag::Double),
            |i| i32::try_from(i).map_or(NbtTag::Long(i), NbtTag::Int),
        ),
        serde_json::Value::String(s) => NbtTag::String(s.clone().into()),
        serde_json::Value::Array(arr) => NbtTag::List(arr.iter().map(json_to_nbt_tag).collect()),
        serde_json::Value::Object(map) => {
            let mut compound = NbtCompound::new();
            for (k, v) in map {
                compound.put(k, json_to_nbt_tag(v));
            }
            NbtTag::Compound(compound)
        }
    }
}

#[must_use]
pub fn nbt_tag_to_json(tag: &NbtTag) -> serde_json::Value {
    match tag {
        NbtTag::Byte(b) => serde_json::Value::Number((*b).into()),
        NbtTag::Short(s) => serde_json::Value::Number((*s).into()),
        NbtTag::Int(i) => serde_json::Value::Number((*i).into()),
        NbtTag::Long(l) => serde_json::Value::Number((*l).into()),
        NbtTag::Float(f) => serde_json::Number::from_f64(*f as f64)
            .map_or(serde_json::Value::Null, serde_json::Value::Number),
        NbtTag::Double(d) => serde_json::Number::from_f64(*d)
            .map_or(serde_json::Value::Null, serde_json::Value::Number),
        NbtTag::String(s) => serde_json::Value::String(s.to_string()),
        NbtTag::List(list) => serde_json::Value::Array(list.iter().map(nbt_tag_to_json).collect()),
        NbtTag::Compound(comp) => {
            let mut map = serde_json::Map::new();
            for (k, v) in &comp.child_tags {
                map.insert(k.to_string(), nbt_tag_to_json(v));
            }
            serde_json::Value::Object(map)
        }
        NbtTag::ByteArray(arr) => serde_json::Value::Array(
            arr.iter()
                .map(|b| serde_json::Value::Number((*b).into()))
                .collect(),
        ),
        NbtTag::IntArray(arr) => serde_json::Value::Array(
            arr.iter()
                .map(|i| serde_json::Value::Number((*i).into()))
                .collect(),
        ),
        NbtTag::LongArray(arr) => serde_json::Value::Array(
            arr.iter()
                .map(|l| serde_json::Value::Number((*l).into()))
                .collect(),
        ),
        NbtTag::End => serde_json::Value::Null,
    }
}

pub fn read_world_gen_settings(level_folder: &Path) -> Option<WorldGenSettings> {
    let path = minecraft_data_dir(level_folder).join("world_gen_settings.dat");
    if !path.exists() {
        return None;
    }
    match File::open(&path) {
        Ok(f) => match read_gzip_compound_tag(f) {
            Ok(compound) => {
                let data_compound = compound.get_compound("data");
                let c = data_compound.as_ref().map_or(&compound, |v| v);
                let seed = c.get_long("seed");
                if seed.is_none() {
                    warn!("world_gen_settings.dat has no seed");
                }
                let seed = seed?;
                let mut dimensions = std::collections::HashMap::new();
                if let Some(dims_comp) = c.get_compound("dimensions") {
                    for (dim_name, dim_tag) in &dims_comp.child_tags {
                        if let NbtTag::Compound(dim_c) = dim_tag {
                            let dim_type = dim_c.get_string("type").unwrap_or(dim_name).to_string();
                            if let Some(gen_c) = dim_c.get_compound("generator") {
                                let generator_type = gen_c
                                    .get_string("type")
                                    .unwrap_or("minecraft:noise")
                                    .to_string();
                                let settings = gen_c
                                    .get_string("settings")
                                    .map(|s| {
                                        crate::world_info::GeneratorSettings::Reference(
                                            s.to_string(),
                                        )
                                    })
                                    .or_else(|| {
                                        gen_c.get_compound("settings").map(|settings_c| {
                                            let json_val = nbt_tag_to_json(&NbtTag::Compound(
                                                settings_c.clone(),
                                            ));
                                            crate::world_info::GeneratorSettings::Compound(json_val)
                                        })
                                    });
                                let biome_source = gen_c.get_compound("biome_source").map(|bs_c| {
                                    let biome_type = bs_c
                                        .get_string("type")
                                        .unwrap_or("minecraft:multi_noise")
                                        .to_string();
                                    match bs_c.get_string("preset") {
                                        Some(preset) => {
                                            crate::world_info::BiomeSource::WithPreset {
                                                preset: preset.to_string(),
                                                biome_type,
                                            }
                                        }
                                        None => {
                                            crate::world_info::BiomeSource::Simple { biome_type }
                                        }
                                    }
                                });
                                dimensions.insert(
                                    dim_name.to_string(),
                                    crate::world_info::Dimension {
                                        generator: crate::world_info::Generator {
                                            settings,
                                            biome_source,
                                            generator_type,
                                        },
                                        dimension_type: dim_type,
                                    },
                                );
                            }
                        }
                    }
                }
                Some(WorldGenSettings { seed, dimensions })
            }
            Err(e) => {
                warn!("Failed to deserialize world_gen_settings.dat: {e}");
                None
            }
        },
        Err(e) => {
            warn!("Failed to open world_gen_settings.dat: {e}");
            None
        }
    }
}

pub fn write_world_gen_settings(
    level_folder: &Path,
    settings: &WorldGenSettings,
    data_version: i32,
) -> Result<(), WorldInfoError> {
    let dir = ensure_minecraft_data_dir(level_folder)?;
    let path = dir.join("world_gen_settings.dat");
    let file = File::create(&path)?;
    let mut inner = NbtCompound::new();
    inner.put_int("DataVersion", data_version);
    inner.put_long("seed", settings.seed);
    inner.put_bool("generate_structures", true);
    inner.put_bool("bonus_chest", false);

    let mut dims_comp = NbtCompound::new();
    for (dim_name, dim) in &settings.dimensions {
        let mut dim_comp = NbtCompound::new();
        dim_comp.put_string("type", dim.dimension_type.clone());

        let mut gen_comp = NbtCompound::new();
        gen_comp.put_string("type", dim.generator.generator_type.clone());
        if let Some(s) = &dim.generator.settings {
            match s {
                crate::world_info::GeneratorSettings::Reference(r) => {
                    gen_comp.put_string("settings", r.clone());
                }
                crate::world_info::GeneratorSettings::Compound(json_val) => {
                    gen_comp.put("settings", json_to_nbt_tag(json_val));
                }
            }
        }
        if let Some(bs) = &dim.generator.biome_source {
            let mut bs_comp = NbtCompound::new();
            match bs {
                crate::world_info::BiomeSource::WithPreset { preset, biome_type } => {
                    bs_comp.put_string("preset", preset.clone());
                    bs_comp.put_string("type", biome_type.clone());
                }
                crate::world_info::BiomeSource::Simple { biome_type } => {
                    bs_comp.put_string("type", biome_type.clone());
                }
            }
            gen_comp.put_compound("biome_source", bs_comp);
        }
        dim_comp.put_compound("generator", gen_comp);
        dims_comp.put_compound(dim_name, dim_comp);
    }
    inner.put_compound("dimensions", dims_comp);

    let mut root = NbtCompound::new();
    root.put_compound("data", inner);
    pumpkin_nbt::nbt_compress::write_gzip_compound_tag(root, BufWriter::new(file))
        .map_err(|e| WorldInfoError::SerializationError(e.to_string()))
}

#[must_use]
pub fn game_rules_to_nbt(rules: &GameRuleRegistry, data_version: i32) -> NbtCompound {
    let mut inner = NbtCompound::new();
    for rule in GameRule::all() {
        let key = format!("minecraft:{rule}");
        match rules.get(rule) {
            GameRuleValue::Bool(b) => inner.put(&key, NbtTag::Byte(i8::from(*b))),
            GameRuleValue::Int(i) => inner.put(&key, NbtTag::Int(*i as i32)),
        }
    }
    inner.put_int("DataVersion", data_version);

    let mut root = NbtCompound::new();
    root.put_compound("data", inner);
    root
}

pub fn game_rules_from_nbt(root: &NbtCompound) -> GameRuleRegistry {
    let mut registry = GameRuleRegistry::default();

    let Some(inner) = root.get_compound("data") else {
        warn!("game_rules.dat missing 'data' compound, using defaults");
        return registry;
    };

    for rule in GameRule::all() {
        let key = format!("minecraft:{rule}");
        match registry.get_mut(rule) {
            GameRuleValue::Bool(b) => {
                if let Some(v) = inner.get_byte(&key) {
                    *b = v != 0;
                }
            }
            GameRuleValue::Int(i) => {
                if let Some(v) = inner.get_int(&key) {
                    *i = i64::from(v);
                }
            }
        }
    }

    registry
}

pub fn read_game_rules(level_folder: &Path) -> GameRuleRegistry {
    let path = minecraft_data_dir(level_folder).join("game_rules.dat");
    if !path.exists() {
        return GameRuleRegistry::default();
    }

    match File::open(&path) {
        Ok(f) => match read_gzip_compound_tag(f) {
            Ok(compound) => game_rules_from_nbt(&compound),
            Err(e) => {
                warn!("Failed to parse game_rules.dat: {e}");
                GameRuleRegistry::default()
            }
        },
        Err(e) => {
            warn!("Failed to open game_rules.dat: {e}");
            GameRuleRegistry::default()
        }
    }
}

pub fn write_game_rules(
    level_folder: &Path,
    rules: &GameRuleRegistry,
    data_version: i32,
) -> Result<(), WorldInfoError> {
    let dir = ensure_minecraft_data_dir(level_folder)?;
    let path = dir.join("game_rules.dat");

    let compound = game_rules_to_nbt(rules, data_version);
    let file = File::create(&path)?;

    pumpkin_nbt::nbt_compress::write_gzip_compound_tag(compound, file)
        .map_err(|e| WorldInfoError::SerializationError(e.to_string()))
}

pub fn read_world_clocks(level_folder: &Path) -> WorldClocksData {
    let path = minecraft_data_dir(level_folder).join("world_clocks.dat");
    if !path.exists() {
        return WorldClocksData::default();
    }

    match File::open(&path) {
        Ok(f) => match read_gzip_compound_tag(f) {
            Ok(compound) => world_clocks_from_nbt(&compound),
            Err(e) => {
                warn!("Failed to parse world_clocks.dat: {e}");
                WorldClocksData::default()
            }
        },
        Err(e) => {
            warn!("Failed to open world_clocks.dat: {e}");
            WorldClocksData::default()
        }
    }
}

fn world_clocks_from_nbt(root: &NbtCompound) -> WorldClocksData {
    let mut result = WorldClocksData::default();

    let Some(inner) = root.get_compound("data") else {
        return result;
    };

    result.data_version = inner.get_int("DataVersion").unwrap_or(0);

    for (key, tag) in &inner.child_tags {
        if key.as_ref() == "DataVersion" {
            continue;
        }
        if let NbtTag::Compound(dim_compound) = tag {
            let total_ticks = dim_compound.get_long("total_ticks").unwrap_or(0);
            result
                .clocks
                .insert(key.to_string(), DimensionClock { total_ticks });
        }
    }

    result
}

pub fn write_world_clocks(
    level_folder: &Path,
    clocks: &WorldClocksData,
) -> Result<(), WorldInfoError> {
    let dir = ensure_minecraft_data_dir(level_folder)?;
    let path = dir.join("world_clocks.dat");

    let mut inner = NbtCompound::new();
    for (dim_name, clock) in &clocks.clocks {
        let mut dim_compound = NbtCompound::new();
        dim_compound.put_long("total_ticks", clock.total_ticks);
        inner.put_compound(dim_name, dim_compound);
    }
    inner.put_int("DataVersion", clocks.data_version);

    let mut root = NbtCompound::new();
    root.put_compound("data", inner);

    let file = File::create(&path)?;

    pumpkin_nbt::nbt_compress::write_gzip_compound_tag(root, file)
        .map_err(|e| WorldInfoError::SerializationError(e.to_string()))
}

pub fn read_wandering_trader(level_folder: &Path) -> WanderingTraderData {
    let path = minecraft_data_dir(level_folder).join("wandering_trader.dat");
    if !path.exists() {
        return WanderingTraderData::default();
    }
    match File::open(&path) {
        Ok(f) => match read_gzip_compound_tag(f) {
            Ok(compound) => {
                let data_compound = compound.get_compound("data");
                let c = data_compound.as_ref().map_or(&compound, |v| v);
                let data_version = compound
                    .get_int("DataVersion")
                    .or_else(|| c.get_int("DataVersion"))
                    .unwrap_or(0);
                WanderingTraderData {
                    spawn_delay: c
                        .get_int("spawn_delay")
                        .or_else(|| c.get_int("WanderingTraderSpawnDelay"))
                        .unwrap_or(24_000),
                    spawn_chance: c
                        .get_int("spawn_chance")
                        .or_else(|| c.get_int("WanderingTraderSpawnChance"))
                        .unwrap_or(25),
                    data_version,
                }
            }
            Err(e) => {
                warn!("Failed to deserialize wandering_trader.dat, using defaults: {e}");
                WanderingTraderData::default()
            }
        },
        Err(e) => {
            warn!("Failed to open wandering_trader.dat, using defaults: {e}");
            WanderingTraderData::default()
        }
    }
}

pub fn write_wandering_trader(
    level_folder: &Path,
    data: &WanderingTraderData,
) -> Result<(), WorldInfoError> {
    let dir = ensure_minecraft_data_dir(level_folder)?;
    let path = dir.join("wandering_trader.dat");
    let file = File::create(&path)?;
    let mut data_comp = NbtCompound::new();
    data_comp.put_int("spawn_delay", data.spawn_delay);
    data_comp.put_int("spawn_chance", data.spawn_chance);
    let mut root = NbtCompound::new();
    root.put_int("DataVersion", data.data_version);
    root.put_compound("data", data_comp);
    pumpkin_nbt::nbt_compress::write_gzip_compound_tag(root, BufWriter::new(file))
        .map_err(|e| WorldInfoError::SerializationError(e.to_string()))
}

pub fn write_custom_boss_events_stub(
    level_folder: &Path,
    data_version: i32,
) -> Result<(), WorldInfoError> {
    let dir = ensure_minecraft_data_dir(level_folder)?;
    let path = dir.join("custom_boss_events.dat");
    if path.exists() {
        return Ok(());
    }

    let mut root = NbtCompound::new();
    root.put_int("DataVersion", data_version);
    root.put_compound("data", NbtCompound::new());

    let file = File::create(&path)?;
    pumpkin_nbt::nbt_compress::write_gzip_compound_tag(root, file)
        .map_err(|e| WorldInfoError::SerializationError(e.to_string()))
}

pub fn write_scheduled_events_stub(
    level_folder: &Path,
    data_version: i32,
) -> Result<(), WorldInfoError> {
    let dir = ensure_minecraft_data_dir(level_folder)?;
    let path = dir.join("scheduled_events.dat");
    if path.exists() {
        return Ok(());
    }

    let mut inner = NbtCompound::new();
    inner.put("events", NbtTag::List(vec![]));
    let mut root = NbtCompound::new();
    root.put_int("DataVersion", data_version);
    root.put_compound("data", inner);

    let file = File::create(&path)?;
    pumpkin_nbt::nbt_compress::write_gzip_compound_tag(root, file)
        .map_err(|e| WorldInfoError::SerializationError(e.to_string()))
}

pub fn write_random_sequences_stub(
    level_folder: &Path,
    data_version: i32,
) -> Result<(), WorldInfoError> {
    let dir = ensure_minecraft_data_dir(level_folder)?;
    let path = dir.join("random_sequences.dat");
    if path.exists() {
        return Ok(());
    }

    let mut inner = NbtCompound::new();
    inner.put_int("salt", 0);
    inner.put_compound("sequences", NbtCompound::new());
    let mut root = NbtCompound::new();
    root.put_int("DataVersion", data_version);
    root.put_compound("data", inner);

    let file = File::create(&path)?;
    pumpkin_nbt::nbt_compress::write_gzip_compound_tag(root, file)
        .map_err(|e| WorldInfoError::SerializationError(e.to_string()))
}

pub fn write_scoreboard_stub(level_folder: &Path, data_version: i32) -> Result<(), WorldInfoError> {
    let dir = ensure_minecraft_data_dir(level_folder)?;
    let path = dir.join("scoreboard.dat");
    if path.exists() {
        return Ok(());
    }

    let mut root = NbtCompound::new();
    root.put_int("DataVersion", data_version);
    root.put_compound("data", NbtCompound::new());

    let file = File::create(&path)?;
    pumpkin_nbt::nbt_compress::write_gzip_compound_tag(root, file)
        .map_err(|e| WorldInfoError::SerializationError(e.to_string()))
}

pub fn write_stopwatches_stub(
    level_folder: &Path,
    data_version: i32,
) -> Result<(), WorldInfoError> {
    let dir = ensure_minecraft_data_dir(level_folder)?;
    let path = dir.join("stopwatches.dat");
    if path.exists() {
        return Ok(());
    }

    let mut inner = NbtCompound::new();
    inner.put_compound("stopwatches", NbtCompound::new());
    let mut root = NbtCompound::new();
    root.put_int("DataVersion", data_version);
    root.put_compound("data", inner);

    let file = File::create(&path)?;
    pumpkin_nbt::nbt_compress::write_gzip_compound_tag(root, file)
        .map_err(|e| WorldInfoError::SerializationError(e.to_string()))
}
