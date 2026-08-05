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
    root.put_compound("data", data_comp);
    pumpkin_nbt::nbt_compress::write_gzip_compound_tag(root, BufWriter::new(file))
        .map_err(|e| WorldInfoError::SerializationError(e.to_string()))
}

pub fn read_world_gen_settings(level_folder: &Path) -> Option<WorldGenSettings> {
    let path = minecraft_data_dir(level_folder).join("world_gen_settings.dat");
    if !path.exists() {
        return None;
    }
    match File::open(&path) {
        Ok(f) => match read_gzip_compound_tag(f) {
            Ok(compound) => {
                let seed = compound
                    .get_compound("data")
                    .and_then(|c| c.get_long("seed"))
                    .unwrap_or(0);
                Some(WorldGenSettings {
                    seed,
                    dimensions: std::collections::HashMap::new(),
                })
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
                WanderingTraderData {
                    spawn_delay: c.get_int("WanderingTraderSpawnDelay").unwrap_or(24_000),
                    spawn_chance: c.get_int("WanderingTraderSpawnChance").unwrap_or(25),
                    data_version: c.get_int("DataVersion").unwrap_or(0),
                }
            }
            Err(e) => {
                warn!("Failed to deserialize wandering_trader.dat, using defaults: {e}");
                WanderingTraderData::default()
            }
        },
        Err(e) => {
            warn!("Failed to open wandering_trader.dat: {e}");
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
    data_comp.put_int("WanderingTraderSpawnDelay", data.spawn_delay);
    data_comp.put_int("WanderingTraderSpawnChance", data.spawn_chance);
    let mut root = NbtCompound::new();
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
    // Only create if absent; actual boss-bar persistence lives elsewhere.
    if path.exists() {
        return Ok(());
    }

    let mut inner = NbtCompound::new();
    inner.put_int("DataVersion", data_version);
    let mut root = NbtCompound::new();
    root.put_compound("data", inner);

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
    inner.put_int("DataVersion", data_version);
    let mut root = NbtCompound::new();
    root.put_compound("data", inner);

    let file = File::create(&path)?;

    pumpkin_nbt::nbt_compress::write_gzip_compound_tag(root, file)
        .map_err(|e| WorldInfoError::SerializationError(e.to_string()))
}

fn default_criteria_name() -> String {
    "dummy".to_string()
}

fn default_render_type() -> String {
    "integer".to_string()
}

fn default_visibility() -> String {
    "always".to_string()
}

fn default_collision_rule() -> String {
    "always".to_string()
}

fn default_empty_text_component() -> pumpkin_util::text::TextComponentBase {
    pumpkin_util::text::TextComponent::empty().0
}

/// Serializable scoreboard data for `data/minecraft/scoreboard.dat`.
///
/// Field names and structure mirror vanilla's `ScoreboardSaveData.Packed` record
/// exactly (see `net.minecraft.world.scores.ScoreboardSaveData`, decompiled 26.2
/// source), so a `scoreboard.dat` written here loads in vanilla and vice versa.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct ScoreboardData {
    #[serde(rename = "Objectives", default)]
    pub objectives: Vec<SerializableObjective>,
    #[serde(rename = "PlayerScores", default)]
    pub scores: Vec<SerializableScore>,
    /// Display slot bindings: slot id (e.g. "list", "sidebar", "sidebar.team.red",
    /// matching `net.minecraft.world.scores.DisplaySlot#getSerializedName`) -> objective name.
    #[serde(rename = "DisplaySlots", default)]
    pub display_slots: std::collections::HashMap<String, String>,
    #[serde(rename = "Teams", default)]
    pub teams: Vec<SerializableTeam>,
}

/// Mirrors `net.minecraft.world.scores.Objective.Packed`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SerializableObjective {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "CriteriaName", default = "default_criteria_name")]
    pub criteria_name: String,
    #[serde(rename = "DisplayName")]
    pub display_name: pumpkin_util::text::TextComponentBase,
    #[serde(rename = "RenderType", default = "default_render_type")]
    pub render_type: String,
    /// NOTE: vanilla's key here is genuinely `snake_case` (`display_auto_update`)
    /// while the surrounding keys are `PascalCase`; this is not a typo.
    #[serde(rename = "display_auto_update", default)]
    pub display_auto_update: bool,
    /// Number format override, JSON-encoded. Vanilla encodes this as a proper
    /// `NumberFormatTypes` sum type (blank/styled/fixed); modeling that fully is
    /// deferred, so this field is Pumpkin-internal only and will not round-trip
    /// through vanilla. See `net.minecraft.network.chat.numbers.NumberFormatTypes`.
    #[serde(rename = "format", default, skip_serializing_if = "Option::is_none")]
    pub number_format: Option<String>,
}

/// Mirrors `net.minecraft.world.scores.Scoreboard.PackedScore` + `Score.Packed`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SerializableScore {
    #[serde(rename = "Name")]
    pub entity_name: String,
    #[serde(rename = "Objective")]
    pub objective_name: String,
    #[serde(rename = "Score", default)]
    pub value: i32,
    #[serde(rename = "Locked", default)]
    pub locked: bool,
    #[serde(rename = "display", default, skip_serializing_if = "Option::is_none")]
    pub display: Option<pumpkin_util::text::TextComponentBase>,
    /// See the note on `SerializableObjective::number_format`.
    #[serde(rename = "format", default, skip_serializing_if = "Option::is_none")]
    pub number_format: Option<String>,
}

/// Mirrors `net.minecraft.world.scores.PlayerTeam.Packed`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SerializableTeam {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(
        rename = "DisplayName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub display_name: Option<pumpkin_util::text::TextComponentBase>,
    /// One of vanilla's `TeamColor` serialized names (`black`, `dark_blue`, ...),
    /// absent when the team has no color.
    #[serde(rename = "TeamColor", default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(rename = "AllowFriendlyFire", default = "default_true")]
    pub friendly_fire: bool,
    #[serde(rename = "SeeFriendlyInvisibles", default = "default_true")]
    pub see_friendly_invisibles: bool,
    #[serde(rename = "MemberNamePrefix", default = "default_empty_text_component")]
    pub player_prefix: pumpkin_util::text::TextComponentBase,
    #[serde(rename = "MemberNameSuffix", default = "default_empty_text_component")]
    pub player_suffix: pumpkin_util::text::TextComponentBase,
    #[serde(rename = "NameTagVisibility", default = "default_visibility")]
    pub nametag_visibility: String,
    #[serde(rename = "DeathMessageVisibility", default = "default_visibility")]
    pub death_message_visibility: String,
    #[serde(rename = "CollisionRule", default = "default_collision_rule")]
    pub collision_rule: String,
    #[serde(rename = "Players", default)]
    pub players: Vec<String>,
}

/// Manual NBT codec for `scoreboard.dat`.
///
/// Upstream `7350fba3` removed the serde bridge from `pumpkin-nbt`, so the
/// `DataFileRoot<ScoreboardData>` round-trip has to be spelled out. The key
/// names below are unchanged from the serde attributes on the structs above.
mod scoreboard_nbt {
    use super::{ScoreboardData, SerializableObjective, SerializableScore, SerializableTeam};
    use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
    use pumpkin_util::text::TextComponentBase;

    fn text(compound: &NbtCompound, key: &str) -> Option<TextComponentBase> {
        compound
            .get_compound(key)
            .map(TextComponentBase::from_nbt_compound)
    }

    fn compounds<'a>(compound: &'a NbtCompound, key: &str) -> Vec<&'a NbtCompound> {
        compound.get_list(key).map_or_else(Vec::new, |list| {
            list.iter()
                .filter_map(|tag| match tag {
                    NbtTag::Compound(c) => Some(c),
                    _ => None,
                })
                .collect()
        })
    }

    fn put_opt_string(compound: &mut NbtCompound, key: &str, value: Option<&String>) {
        if let Some(value) = value {
            compound.put_string(key, value.clone());
        }
    }

    fn put_opt_text(compound: &mut NbtCompound, key: &str, value: Option<&TextComponentBase>) {
        if let Some(value) = value {
            compound.put_compound(key, value.to_nbt_compound());
        }
    }

    fn objective_to_nbt(objective: &SerializableObjective) -> NbtCompound {
        let mut compound = NbtCompound::new();
        compound.put_string("Name", objective.name.clone());
        compound.put_string("CriteriaName", objective.criteria_name.clone());
        compound.put_compound("DisplayName", objective.display_name.to_nbt_compound());
        compound.put_string("RenderType", objective.render_type.clone());
        compound.put_bool("display_auto_update", objective.display_auto_update);
        put_opt_string(&mut compound, "format", objective.number_format.as_ref());
        compound
    }

    fn objective_from_nbt(compound: &NbtCompound) -> Option<SerializableObjective> {
        Some(SerializableObjective {
            name: compound.get_string("Name")?.to_string(),
            criteria_name: compound
                .get_string("CriteriaName")
                .unwrap_or("dummy")
                .to_string(),
            display_name: text(compound, "DisplayName")?,
            render_type: compound
                .get_string("RenderType")
                .unwrap_or("integer")
                .to_string(),
            display_auto_update: compound.get_bool("display_auto_update").unwrap_or_default(),
            number_format: compound.get_string("format").map(ToString::to_string),
        })
    }

    fn score_to_nbt(score: &SerializableScore) -> NbtCompound {
        let mut compound = NbtCompound::new();
        compound.put_string("Name", score.entity_name.clone());
        compound.put_string("Objective", score.objective_name.clone());
        compound.put_int("Score", score.value);
        compound.put_bool("Locked", score.locked);
        put_opt_text(&mut compound, "display", score.display.as_ref());
        put_opt_string(&mut compound, "format", score.number_format.as_ref());
        compound
    }

    fn score_from_nbt(compound: &NbtCompound) -> Option<SerializableScore> {
        Some(SerializableScore {
            entity_name: compound.get_string("Name")?.to_string(),
            objective_name: compound.get_string("Objective")?.to_string(),
            value: compound.get_int("Score").unwrap_or_default(),
            locked: compound.get_bool("Locked").unwrap_or_default(),
            display: text(compound, "display"),
            number_format: compound.get_string("format").map(ToString::to_string),
        })
    }

    fn team_to_nbt(team: &SerializableTeam) -> NbtCompound {
        let mut compound = NbtCompound::new();
        compound.put_string("Name", team.name.clone());
        put_opt_text(&mut compound, "DisplayName", team.display_name.as_ref());
        put_opt_string(&mut compound, "TeamColor", team.color.as_ref());
        compound.put_bool("AllowFriendlyFire", team.friendly_fire);
        compound.put_bool("SeeFriendlyInvisibles", team.see_friendly_invisibles);
        compound.put_compound("MemberNamePrefix", team.player_prefix.to_nbt_compound());
        compound.put_compound("MemberNameSuffix", team.player_suffix.to_nbt_compound());
        compound.put_string("NameTagVisibility", team.nametag_visibility.clone());
        compound.put_string(
            "DeathMessageVisibility",
            team.death_message_visibility.clone(),
        );
        compound.put_string("CollisionRule", team.collision_rule.clone());
        compound.put_list(
            "Players",
            team.players
                .iter()
                .map(|p| NbtTag::String(p.clone().into_boxed_str()))
                .collect(),
        );
        compound
    }

    fn team_from_nbt(compound: &NbtCompound) -> Option<SerializableTeam> {
        Some(SerializableTeam {
            name: compound.get_string("Name")?.to_string(),
            display_name: text(compound, "DisplayName"),
            color: compound.get_string("TeamColor").map(ToString::to_string),
            friendly_fire: compound.get_bool("AllowFriendlyFire").unwrap_or(true),
            see_friendly_invisibles: compound.get_bool("SeeFriendlyInvisibles").unwrap_or(true),
            player_prefix: text(compound, "MemberNamePrefix")
                .unwrap_or_else(|| pumpkin_util::text::TextComponent::text(String::new()).0),
            player_suffix: text(compound, "MemberNameSuffix")
                .unwrap_or_else(|| pumpkin_util::text::TextComponent::text(String::new()).0),
            nametag_visibility: compound
                .get_string("NameTagVisibility")
                .unwrap_or("always")
                .to_string(),
            death_message_visibility: compound
                .get_string("DeathMessageVisibility")
                .unwrap_or("always")
                .to_string(),
            collision_rule: compound
                .get_string("CollisionRule")
                .unwrap_or("always")
                .to_string(),
            players: compound.get_list("Players").map_or_else(Vec::new, |list| {
                list.iter()
                    .filter_map(|tag| match tag {
                        NbtTag::String(s) => Some(s.to_string()),
                        _ => None,
                    })
                    .collect()
            }),
        })
    }

    pub fn to_nbt(data: &ScoreboardData) -> NbtCompound {
        let mut compound = NbtCompound::new();
        compound.put_list(
            "Objectives",
            data.objectives
                .iter()
                .map(|o| NbtTag::Compound(objective_to_nbt(o)))
                .collect(),
        );
        compound.put_list(
            "PlayerScores",
            data.scores
                .iter()
                .map(|s| NbtTag::Compound(score_to_nbt(s)))
                .collect(),
        );
        let mut display_slots = NbtCompound::new();
        for (slot, objective) in &data.display_slots {
            display_slots.put_string(slot, objective.clone());
        }
        compound.put_compound("DisplaySlots", display_slots);
        compound.put_list(
            "Teams",
            data.teams
                .iter()
                .map(|t| NbtTag::Compound(team_to_nbt(t)))
                .collect(),
        );
        compound
    }

    pub fn from_nbt(compound: &NbtCompound) -> ScoreboardData {
        ScoreboardData {
            objectives: compounds(compound, "Objectives")
                .into_iter()
                .filter_map(objective_from_nbt)
                .collect(),
            scores: compounds(compound, "PlayerScores")
                .into_iter()
                .filter_map(score_from_nbt)
                .collect(),
            display_slots: compound.get_compound("DisplaySlots").map_or_else(
                Default::default,
                |slots| {
                    slots
                        .child_tags
                        .iter()
                        .filter_map(|(slot, tag)| match tag {
                            NbtTag::String(objective) => {
                                Some((slot.to_string(), objective.to_string()))
                            }
                            _ => None,
                        })
                        .collect()
                },
            ),
            teams: compounds(compound, "Teams")
                .into_iter()
                .filter_map(team_from_nbt)
                .collect(),
        }
    }
}

pub fn read_scoreboard(level_folder: &Path) -> ScoreboardData {
    let path = minecraft_data_dir(level_folder).join("scoreboard.dat");
    if !path.exists() {
        return ScoreboardData::default();
    }
    match File::open(&path) {
        Ok(f) => match read_gzip_compound_tag(std::io::BufReader::new(f)) {
            Ok(root) => root
                .get_compound("data")
                .map_or_else(ScoreboardData::default, scoreboard_nbt::from_nbt),
            Err(e) => {
                warn!("Failed to deserialize scoreboard.dat, using defaults: {e}");
                ScoreboardData::default()
            }
        },
        Err(e) => {
            warn!("Failed to open scoreboard.dat, using defaults: {e}");
            ScoreboardData::default()
        }
    }
}

pub fn write_scoreboard(level_folder: &Path, data: &ScoreboardData) -> Result<(), WorldInfoError> {
    let dir = ensure_minecraft_data_dir(level_folder)?;
    let path = dir.join("scoreboard.dat");
    let file = File::create(&path)?;
    let mut root = NbtCompound::new();
    root.put_compound("data", scoreboard_nbt::to_nbt(data));
    pumpkin_nbt::nbt_compress::write_gzip_compound_tag(root, BufWriter::new(file))
        .map_err(|e| WorldInfoError::SerializationError(e.to_string()))
}

#[cfg(test)]
mod scoreboard_test {
    use super::{ScoreboardData, SerializableObjective, SerializableScore, SerializableTeam};
    use pumpkin_util::text::TextComponent;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn sample_data() -> ScoreboardData {
        let mut display_slots = HashMap::new();
        display_slots.insert("sidebar".to_string(), "test_objective".to_string());
        display_slots.insert("sidebar.team.red".to_string(), "red_objective".to_string());

        ScoreboardData {
            objectives: vec![SerializableObjective {
                name: "test_objective".to_string(),
                criteria_name: "dummy".to_string(),
                display_name: TextComponent::text("Test Objective").0,
                render_type: "integer".to_string(),
                display_auto_update: false,
                number_format: None,
            }],
            scores: vec![SerializableScore {
                entity_name: "Steve".to_string(),
                objective_name: "test_objective".to_string(),
                value: 42,
                locked: false,
                display: None,
                number_format: None,
            }],
            display_slots,
            teams: vec![SerializableTeam {
                name: "red_team".to_string(),
                display_name: Some(TextComponent::text("Red Team").0),
                color: Some("red".to_string()),
                friendly_fire: true,
                see_friendly_invisibles: true,
                player_prefix: TextComponent::empty().0,
                player_suffix: TextComponent::empty().0,
                nametag_visibility: "always".to_string(),
                death_message_visibility: "hideForOtherTeams".to_string(),
                collision_rule: "pushOwnTeam".to_string(),
                players: vec!["Steve".to_string()],
            }],
        }
    }

    /// A `ScoreboardData` written to `scoreboard.dat` and read back must be
    /// byte-for-byte equivalent, since this file is meant to be cross-compatible
    /// with vanilla's own `scoreboard.dat` (see `ScoreboardSaveData` in the
    /// decompiled 26.2 source).
    #[test]
    fn scoreboard_dat_round_trip() {
        let temp_dir = TempDir::new().unwrap();
        let data = sample_data();

        super::write_scoreboard(temp_dir.path(), &data).unwrap();
        let loaded = super::read_scoreboard(temp_dir.path());

        assert_eq!(loaded, data);
    }

    /// Missing `scoreboard.dat` (a fresh world) must not error, and must yield an
    /// empty (default) scoreboard.
    #[test]
    fn missing_scoreboard_dat_yields_default() {
        let temp_dir = TempDir::new().unwrap();
        let loaded = super::read_scoreboard(temp_dir.path());
        assert_eq!(loaded, ScoreboardData::default());
    }

    /// Vanilla NBT field names are exact and case-sensitive; verify the on-disk
    /// keys match `ScoreboardSaveData.Packed`'s codec field names, not just that
    /// our own round-trip is internally consistent.
    #[test]
    fn field_names_match_vanilla() {
        let temp_dir = TempDir::new().unwrap();
        let data = sample_data();
        super::write_scoreboard(temp_dir.path(), &data).unwrap();

        let path = super::minecraft_data_dir(temp_dir.path()).join("scoreboard.dat");
        let file = std::fs::File::open(&path).unwrap();
        let root = pumpkin_nbt::nbt_compress::read_gzip_compound_tag(file).unwrap();
        let inner = root.get_compound("data").expect("missing 'data' compound");

        assert!(inner.get_list("Objectives").is_some());
        assert!(inner.get_list("PlayerScores").is_some());
        assert!(inner.get_compound("DisplaySlots").is_some());
        assert!(inner.get_list("Teams").is_some());
    }
}
