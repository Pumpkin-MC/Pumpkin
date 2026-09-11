use std::collections::HashMap;
use std::path::Path;

use crate::CURRENT_MC_VERSION;
use pumpkin_data::game_rules::GameRuleRegistry;
use pumpkin_util::{Difficulty, serde_enum_as_integer, world_seed::Seed};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod anvil;
pub mod data_files;

// Constraint: disk biome palette serialization changed in 1.21.5
pub const MINIMUM_SUPPORTED_WORLD_DATA_VERSION: i32 = 4435; // 1.21.9
pub const MAXIMUM_SUPPORTED_WORLD_DATA_VERSION: i32 = 4903; // 26.2

pub const MINIMUM_SUPPORTED_LEVEL_VERSION: i32 = 19132; // 1.21.9
pub const MAXIMUM_SUPPORTED_LEVEL_VERSION: i32 = 19133; // 1.21.9

pub trait WorldInfoReader {
    fn read_world_info(&self, level_folder: &Path) -> Result<LevelData, WorldInfoError>;
}

pub trait WorldInfoWriter: Sync + Send {
    fn write_world_info(&self, info: &LevelData, level_folder: &Path)
    -> Result<(), WorldInfoError>;
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct LevelData {
    #[serde(rename = "allowCommands", default)]
    pub allow_commands: bool,
    #[serde(default)]
    pub border_center_x: f64,
    #[serde(default)]
    pub border_center_z: f64,
    #[serde(default = "default_border_damage_per_block")]
    pub border_damage_per_block: f64,
    #[serde(default = "default_border_size")]
    pub border_size: f64,
    #[serde(default = "default_border_safe_zone")]
    pub border_safe_zone: f64,
    #[serde(default = "default_border_size")]
    pub border_size_lerp_target: f64,
    #[serde(default)]
    pub border_size_lerp_time: i64,
    #[serde(default = "default_border_warning_blocks")]
    pub border_warning_blocks: f64,
    #[serde(default = "default_border_warning_time")]
    pub border_warning_time: f64,
    #[serde(default = "default_data_packs")]
    pub data_packs: DataPacks,
    pub data_version: i32,
    #[serde(with = "serde_enum_as_integer", default = "default_difficulty")]
    pub difficulty: Difficulty,
    #[serde(default)]
    pub difficulty_locked: bool,
    #[serde(default)]
    pub last_played: i64,
    #[serde(default = "default_level_name")]
    pub level_name: String,
    #[serde(default)]
    pub spawn_x: i32,
    #[serde(default = "default_spawn_y")]
    pub spawn_y: i32,
    #[serde(default)]
    pub spawn_z: i32,
    #[serde(alias = "SpawnAngle", default)]
    pub spawn_yaw: f32,
    #[serde(default)]
    pub spawn_pitch: f32,
    #[serde(rename = "Version", default)]
    pub world_version: WorldVersion,
    #[serde(rename = "version", default = "default_level_version")]
    pub level_version: i32,
    #[serde(rename = "map_id", default)]
    pub map_id: i32,

    // These are NOT serialized to level.dat, but are still deserialized from it if present.
    // They are loaded and saved by AnvilLevelInfo via the data_files module.
    /// Game rules – persisted to `data/minecraft/game_rules.dat`.
    #[serde(skip_serializing, default)]
    pub game_rules: GameRuleRegistry,

    /// World generation settings – persisted to `data/minecraft/world_gen_settings.dat`.
    #[serde(skip_serializing, default)]
    pub world_gen_settings: WorldGenSettings,

    /// In-game time of day (overworld dimension clock).
    /// Persisted to `data/minecraft/world_clocks.dat`.
    #[serde(skip_serializing, default)]
    pub day_time: i64,

    /// Remaining ticks of forced-clear weather.
    /// Persisted to `data/minecraft/weather.dat`.
    #[serde(rename = "clearWeatherTime", skip_serializing, default)]
    pub clear_weather_time: i32,
}

const DEFAULT_BORDER_DAMAGE_PER_BLOCK: f64 = 0.2;
const DEFAULT_BORDER_SIZE: f64 = 60_000_000.0;
const DEFAULT_BORDER_SAFE_ZONE: f64 = 5.0;
const DEFAULT_BORDER_WARNING_BLOCKS: f64 = 5.0;
const DEFAULT_BORDER_WARNING_TIME: f64 = 15.0;
const DEFAULT_DIFFICULTY: Difficulty = Difficulty::Normal;
const DEFAULT_LEVEL_NAME: &str = "world";
const DEFAULT_SPAWN_Y: i32 = 200;
const DEFAULT_ENABLED_DATA_PACK: &str = "vanilla";
const DEFAULT_WORLD_VERSION_SERIES: &str = "main";

const fn default_border_damage_per_block() -> f64 {
    DEFAULT_BORDER_DAMAGE_PER_BLOCK
}
const fn default_border_size() -> f64 {
    DEFAULT_BORDER_SIZE
}
const fn default_border_safe_zone() -> f64 {
    DEFAULT_BORDER_SAFE_ZONE
}
const fn default_border_warning_blocks() -> f64 {
    DEFAULT_BORDER_WARNING_BLOCKS
}
const fn default_border_warning_time() -> f64 {
    DEFAULT_BORDER_WARNING_TIME
}
fn default_enabled_data_packs() -> Vec<String> {
    vec![DEFAULT_ENABLED_DATA_PACK.to_string()]
}
fn default_data_packs() -> DataPacks {
    DataPacks {
        disabled: vec![],
        enabled: default_enabled_data_packs(),
    }
}
const fn default_difficulty() -> Difficulty {
    DEFAULT_DIFFICULTY
}
fn default_level_name() -> String {
    DEFAULT_LEVEL_NAME.to_string()
}
const fn default_spawn_y() -> i32 {
    DEFAULT_SPAWN_Y
}
const fn default_level_version() -> i32 {
    MAXIMUM_SUPPORTED_LEVEL_VERSION
}
fn default_world_version_name() -> String {
    CURRENT_MC_VERSION.to_string()
}
const fn default_world_version_id() -> i32 {
    MAXIMUM_SUPPORTED_WORLD_DATA_VERSION
}
fn default_world_version_series() -> String {
    DEFAULT_WORLD_VERSION_SERIES.to_string()
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct WorldGenSettings {
    // the numerical seed of the world
    pub seed: i64,
    #[serde(default)]
    pub dimensions: Dimensions,
}

impl Default for WorldGenSettings {
    fn default() -> Self {
        // Use seed 0 as placeholder; actual seed comes from config or world_gen_settings.dat
        Self::new(pumpkin_util::world_seed::Seed(0))
    }
}

pub type Dimensions = HashMap<String, Dimension>;
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Dimension {
    pub generator: Generator,
    #[serde(rename = "type")]
    pub dimension_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Generator {
    #[serde(default)]
    pub settings: Option<GeneratorSettings>,
    #[serde(default)]
    pub biome_source: Option<BiomeSource>,
    #[serde(rename = "type")]
    pub generator_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum GeneratorSettings {
    Reference(String),
    Compound(serde_json::Value),
}

impl GeneratorSettings {
    #[must_use]
    pub fn as_flat_settings(&self) -> Option<FlatPresetSettings> {
        match self {
            Self::Reference(preset_name) => {
                FlatLevelGeneratorPreset::from_name(preset_name).map(|p| p.settings)
            }
            Self::Compound(val) => serde_json::from_value(val.clone()).ok(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum BiomeSource {
    WithPreset {
        preset: String,
        #[serde(rename = "type")]
        biome_type: String,
    },
    Fixed {
        biome: String,
        #[serde(rename = "type")]
        biome_type: String,
    },
    Simple {
        #[serde(rename = "type")]
        biome_type: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct WorldPreset {
    pub dimensions: Dimensions,
}

impl WorldPreset {
    pub const NORMAL_RAW: &'static str = include_str!(
        "../../../../assets/datapacks/26_2/data/minecraft/worldgen/world_preset/normal.json"
    );
    pub const AMPLIFIED_RAW: &'static str = include_str!(
        "../../../../assets/datapacks/26_2/data/minecraft/worldgen/world_preset/amplified.json"
    );
    pub const LARGE_BIOMES_RAW: &'static str = include_str!(
        "../../../../assets/datapacks/26_2/data/minecraft/worldgen/world_preset/large_biomes.json"
    );
    pub const FLAT_RAW: &'static str = include_str!(
        "../../../../assets/datapacks/26_2/data/minecraft/worldgen/world_preset/flat.json"
    );
    pub const FLAT_ALL_DIMENSIONS_RAW: &'static str = include_str!(
        "../../../../assets/datapacks/26_2/data/minecraft/worldgen/world_preset/flat_all_dimensions.json"
    );
    pub const SINGLE_BIOME_SURFACE_RAW: &'static str = include_str!(
        "../../../../assets/datapacks/26_2/data/minecraft/worldgen/world_preset/single_biome_surface.json"
    );
    pub const DEBUG_ALL_BLOCK_STATES_RAW: &'static str = include_str!(
        "../../../../assets/datapacks/26_2/data/minecraft/worldgen/world_preset/debug_all_block_states.json"
    );

    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.strip_prefix("minecraft:").unwrap_or(name);
        let raw = match name {
            "normal" => Self::NORMAL_RAW,
            "amplified" => Self::AMPLIFIED_RAW,
            "large_biomes" => Self::LARGE_BIOMES_RAW,
            "flat" => Self::FLAT_RAW,
            "flat_all_dimensions" => Self::FLAT_ALL_DIMENSIONS_RAW,
            "single_biome_surface" => Self::SINGLE_BIOME_SURFACE_RAW,
            "debug_all_block_states" => Self::DEBUG_ALL_BLOCK_STATES_RAW,
            _ => return None,
        };
        serde_json::from_str(raw).ok()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum StructureOverrides {
    Single(String),
    Multiple(Vec<String>),
}

impl StructureOverrides {
    #[must_use]
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            Self::Single(s) => vec![s.clone()],
            Self::Multiple(list) => list.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FlatPresetLayer {
    pub block: String,
    #[serde(default = "default_layer_height")]
    pub height: i32,
}

const fn default_layer_height() -> i32 {
    1
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FlatPresetSettings {
    pub biome: String,
    #[serde(default)]
    pub features: bool,
    #[serde(default)]
    pub lakes: bool,
    #[serde(default)]
    pub layers: Vec<FlatPresetLayer>,
    #[serde(default)]
    pub structure_overrides: Option<StructureOverrides>,
}

impl FlatPresetSettings {
    #[must_use]
    pub fn to_flat_layers(&self) -> Vec<crate::generation::generator::FlatLayer> {
        self.layers
            .iter()
            .map(|l| crate::generation::generator::FlatLayer {
                block: l.block.clone(),
                height: l.height,
            })
            .collect()
    }

    #[must_use]
    pub fn structure_overrides_vec(&self) -> Option<Vec<String>> {
        self.structure_overrides
            .as_ref()
            .map(StructureOverrides::to_vec)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FlatLevelGeneratorPreset {
    pub display: String,
    pub settings: FlatPresetSettings,
}

impl FlatLevelGeneratorPreset {
    pub const BOTTOMLESS_PIT_RAW: &'static str = include_str!(
        "../../../../assets/datapacks/26_2/data/minecraft/worldgen/flat_level_generator_preset/bottomless_pit.json"
    );
    pub const CLASSIC_FLAT_RAW: &'static str = include_str!(
        "../../../../assets/datapacks/26_2/data/minecraft/worldgen/flat_level_generator_preset/classic_flat.json"
    );
    pub const DESERT_RAW: &'static str = include_str!(
        "../../../../assets/datapacks/26_2/data/minecraft/worldgen/flat_level_generator_preset/desert.json"
    );
    pub const OVERWORLD_RAW: &'static str = include_str!(
        "../../../../assets/datapacks/26_2/data/minecraft/worldgen/flat_level_generator_preset/overworld.json"
    );
    pub const REDSTONE_READY_RAW: &'static str = include_str!(
        "../../../../assets/datapacks/26_2/data/minecraft/worldgen/flat_level_generator_preset/redstone_ready.json"
    );
    pub const SNOWY_KINGDOM_RAW: &'static str = include_str!(
        "../../../../assets/datapacks/26_2/data/minecraft/worldgen/flat_level_generator_preset/snowy_kingdom.json"
    );
    pub const THE_VOID_RAW: &'static str = include_str!(
        "../../../../assets/datapacks/26_2/data/minecraft/worldgen/flat_level_generator_preset/the_void.json"
    );
    pub const TUNNELERS_DREAM_RAW: &'static str = include_str!(
        "../../../../assets/datapacks/26_2/data/minecraft/worldgen/flat_level_generator_preset/tunnelers_dream.json"
    );
    pub const WATER_WORLD_RAW: &'static str = include_str!(
        "../../../../assets/datapacks/26_2/data/minecraft/worldgen/flat_level_generator_preset/water_world.json"
    );

    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.strip_prefix("minecraft:").unwrap_or(name);
        let raw = match name {
            "bottomless_pit" => Self::BOTTOMLESS_PIT_RAW,
            "classic_flat" => Self::CLASSIC_FLAT_RAW,
            "desert" => Self::DESERT_RAW,
            "overworld" => Self::OVERWORLD_RAW,
            "redstone_ready" => Self::REDSTONE_READY_RAW,
            "snowy_kingdom" => Self::SNOWY_KINGDOM_RAW,
            "the_void" => Self::THE_VOID_RAW,
            "tunnelers_dream" => Self::TUNNELERS_DREAM_RAW,
            "water_world" => Self::WATER_WORLD_RAW,
            _ => return None,
        };
        serde_json::from_str(raw).ok()
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct DataPacks {
    // List of disabled data packs.
    #[serde(default)]
    pub disabled: Vec<String>,
    // List of enabled data packs. By default, this is populated with a single string "vanilla".
    #[serde(default = "default_enabled_data_packs")]
    pub enabled: Vec<String>,
}

impl WorldGenSettings {
    #[must_use]
    pub fn new(seed: Seed) -> Self {
        Self::from_preset_name("minecraft:normal", seed).unwrap_or_else(|| {
            let mut dimensions = Dimensions::new();
            dimensions.insert(
                "minecraft:overworld".to_string(),
                Dimension {
                    generator: Generator {
                        settings: Some(GeneratorSettings::Reference(
                            "minecraft:overworld".to_string(),
                        )),
                        biome_source: Some(BiomeSource::WithPreset {
                            preset: "minecraft:overworld".to_string(),
                            biome_type: "minecraft:multi_noise".to_string(),
                        }),
                        generator_type: "minecraft:noise".to_string(),
                    },
                    dimension_type: "minecraft:overworld".to_string(),
                },
            );
            dimensions.insert(
                "minecraft:the_nether".to_string(),
                Dimension {
                    generator: Generator {
                        settings: Some(GeneratorSettings::Reference(
                            "minecraft:nether".to_string(),
                        )),
                        biome_source: Some(BiomeSource::WithPreset {
                            preset: "minecraft:nether".to_string(),
                            biome_type: "minecraft:multi_noise".to_string(),
                        }),
                        generator_type: "minecraft:noise".to_string(),
                    },
                    dimension_type: "minecraft:the_nether".to_string(),
                },
            );
            dimensions.insert(
                "minecraft:the_end".to_string(),
                Dimension {
                    generator: Generator {
                        settings: Some(GeneratorSettings::Reference("minecraft:end".to_string())),
                        biome_source: Some(BiomeSource::Simple {
                            biome_type: "minecraft:the_end".to_string(),
                        }),
                        generator_type: "minecraft:noise".to_string(),
                    },
                    dimension_type: "minecraft:the_end".to_string(),
                },
            );

            Self {
                dimensions,
                seed: seed.0 as i64,
            }
        })
    }

    #[must_use]
    pub fn from_preset(preset: &WorldPreset, seed: Seed) -> Self {
        Self {
            dimensions: preset.dimensions.clone(),
            seed: seed.0 as i64,
        }
    }

    #[must_use]
    pub fn from_preset_name(preset_name: &str, seed: Seed) -> Option<Self> {
        WorldPreset::from_name(preset_name).map(|preset| Self::from_preset(&preset, seed))
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct WorldVersion {
    // The version name as a string, e.g. "15w32b".
    #[serde(default = "default_world_version_name")]
    pub name: String,
    // An integer displaying the data version.
    #[serde(default = "default_world_version_id")]
    pub id: i32,
    // Whether the version is a snapshot or not.
    #[serde(default)]
    pub snapshot: bool,
    // Developing series. In 1.18 experimental snapshots, it was set to "ccpreview". In others, set to "main".
    #[serde(default = "default_world_version_series")]
    pub series: String,
}

impl Default for WorldVersion {
    fn default() -> Self {
        Self {
            name: default_world_version_name(),
            id: default_world_version_id(),
            snapshot: false,
            series: default_world_version_series(),
        }
    }
}

impl LevelData {
    #[must_use]
    pub fn default(seed: Seed) -> Self {
        Self {
            allow_commands: true,
            border_center_x: 0.0,
            border_center_z: 0.0,
            border_damage_per_block: DEFAULT_BORDER_DAMAGE_PER_BLOCK,
            border_size: DEFAULT_BORDER_SIZE,
            border_safe_zone: DEFAULT_BORDER_SAFE_ZONE,
            border_size_lerp_target: DEFAULT_BORDER_SIZE,
            border_size_lerp_time: 0,
            border_warning_blocks: DEFAULT_BORDER_WARNING_BLOCKS,
            border_warning_time: DEFAULT_BORDER_WARNING_TIME,
            data_packs: default_data_packs(),
            data_version: MAXIMUM_SUPPORTED_WORLD_DATA_VERSION,
            difficulty: DEFAULT_DIFFICULTY,
            difficulty_locked: false,
            last_played: -1,
            level_name: DEFAULT_LEVEL_NAME.to_string(),
            spawn_x: 0,
            spawn_y: DEFAULT_SPAWN_Y,
            spawn_z: 0,
            spawn_yaw: 0.0,
            spawn_pitch: 0.0,
            world_version: WorldVersion::default(),
            level_version: MAXIMUM_SUPPORTED_LEVEL_VERSION,
            map_id: 0,
            // fields now in data/minecraft/*.dat
            game_rules: GameRuleRegistry::default(),
            world_gen_settings: WorldGenSettings::new(seed),
            day_time: 0,
            clear_weather_time: -1,
        }
    }

    #[must_use]
    pub fn default_with_preset(seed: Seed, preset_name: &str) -> Self {
        let mut data = Self::default(seed);
        if let Some(wgs) = WorldGenSettings::from_preset_name(preset_name, seed) {
            data.world_gen_settings = wgs;
        }
        data
    }

    #[must_use]
    pub fn from_world_generator(
        seed: Seed,
        generator: &crate::generation::generator::VanillaGenerator,
    ) -> Self {
        let mut data = Self::default(seed);
        let spawn_pos = generator.find_spawn_position();
        data.spawn_x = spawn_pos.0.x;
        data.spawn_z = spawn_pos.0.z;
        data
    }

    pub const fn set_pos(&mut self, x: i32, z: i32) {
        self.spawn_x = x;
        self.spawn_z = z;
    }
}

#[derive(Error, Debug)]
pub enum WorldInfoError {
    #[error("Io error: {0}")]
    IoError(std::io::ErrorKind),
    #[error("Info not found!")]
    InfoNotFound,
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error(
        "No world seed found: neither level.dat nor data/minecraft/world_gen_settings.dat contains one"
    )]
    MissingWorldSeed,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Unsupported world data version: {0}")]
    UnsupportedDataVersion(i32),
    #[error("Unsupported world level version: {0}")]
    UnsupportedLevelVersion(i32),
}

impl From<std::io::Error> for WorldInfoError {
    fn from(value: std::io::Error) -> Self {
        match value.kind() {
            std::io::ErrorKind::NotFound => Self::InfoNotFound,
            value => Self::IoError(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_presets_parse() {
        let preset_names = [
            "normal",
            "amplified",
            "large_biomes",
            "flat",
            "flat_all_dimensions",
            "single_biome_surface",
            "debug_all_block_states",
        ];

        for name in preset_names {
            let preset = WorldPreset::from_name(name)
                .unwrap_or_else(|| panic!("failed to load preset {name}"));
            assert!(
                !preset.dimensions.is_empty(),
                "preset {name} has no dimensions"
            );

            let with_prefix = format!("minecraft:{name}");
            assert!(
                WorldPreset::from_name(&with_prefix).is_some(),
                "failed to load preset with prefix {with_prefix}"
            );
        }
    }

    #[test]
    fn world_gen_settings_from_preset() {
        let seed = Seed(12345);
        let normal = WorldGenSettings::from_preset_name("normal", seed).unwrap();
        assert_eq!(normal.seed, 12345);
        let overworld = normal.dimensions.get("minecraft:overworld").unwrap();
        assert_eq!(overworld.generator.generator_type, "minecraft:noise");
        assert_eq!(
            overworld.generator.settings,
            Some(GeneratorSettings::Reference(
                "minecraft:overworld".to_string()
            ))
        );

        let amplified = WorldGenSettings::from_preset_name("amplified", seed).unwrap();
        let amp_overworld = amplified.dimensions.get("minecraft:overworld").unwrap();
        assert_eq!(
            amp_overworld.generator.settings,
            Some(GeneratorSettings::Reference(
                "minecraft:amplified".to_string()
            ))
        );

        let large_biomes = WorldGenSettings::from_preset_name("large_biomes", seed).unwrap();
        let lb_overworld = large_biomes.dimensions.get("minecraft:overworld").unwrap();
        assert_eq!(
            lb_overworld.generator.settings,
            Some(GeneratorSettings::Reference(
                "minecraft:large_biomes".to_string()
            ))
        );

        let flat = WorldGenSettings::from_preset_name("flat", seed).unwrap();
        let flat_overworld = flat.dimensions.get("minecraft:overworld").unwrap();
        assert_eq!(flat_overworld.generator.generator_type, "minecraft:flat");
        assert!(matches!(
            flat_overworld.generator.settings,
            Some(GeneratorSettings::Compound(_))
        ));
        let flat_settings = flat_overworld
            .generator
            .settings
            .as_ref()
            .unwrap()
            .as_flat_settings()
            .unwrap();
        assert_eq!(flat_settings.biome, "minecraft:plains");
        assert_eq!(flat_settings.layers.len(), 3);
        assert_eq!(flat_settings.to_flat_layers().len(), 3);
    }

    #[test]
    fn flat_level_generator_presets_parse() {
        let preset_names = [
            "bottomless_pit",
            "classic_flat",
            "desert",
            "overworld",
            "redstone_ready",
            "snowy_kingdom",
            "the_void",
            "tunnelers_dream",
            "water_world",
        ];

        for name in preset_names {
            let preset = FlatLevelGeneratorPreset::from_name(name)
                .unwrap_or_else(|| panic!("failed to load flat preset {name}"));
            assert!(!preset.settings.biome.is_empty());
            assert!(!preset.settings.layers.is_empty());
            assert_eq!(
                preset.settings.layers.len(),
                preset.settings.to_flat_layers().len()
            );

            let with_prefix = format!("minecraft:{name}");
            assert!(
                FlatLevelGeneratorPreset::from_name(&with_prefix).is_some(),
                "failed to load flat preset with prefix {with_prefix}"
            );

            let from_ref = GeneratorSettings::Reference(with_prefix)
                .as_flat_settings()
                .expect("failed to parse reference as flat settings");
            assert_eq!(from_ref.biome, preset.settings.biome);
        }

        // Test classic flat structure overrides (single string "minecraft:villages")
        let classic = FlatLevelGeneratorPreset::from_name("classic_flat").unwrap();
        assert_eq!(
            classic.settings.structure_overrides_vec(),
            Some(vec!["minecraft:villages".to_string()])
        );

        // Test tunnelers dream structure overrides (multiple)
        let tunnelers = FlatLevelGeneratorPreset::from_name("tunnelers_dream").unwrap();
        assert_eq!(
            tunnelers.settings.structure_overrides_vec(),
            Some(vec![
                "minecraft:mineshafts".to_string(),
                "minecraft:strongholds".to_string(),
            ])
        );

        // Test the void structure overrides (empty list)
        let void = FlatLevelGeneratorPreset::from_name("the_void").unwrap();
        assert_eq!(
            void.settings.structure_overrides_vec(),
            Some(Vec::<String>::new())
        );
    }
}
