use pumpkin_data::chunk::Biome;
use pumpkin_data::chunk_gen_settings::GenerationSettings;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::{Block, BlockState};

use crate::generation::{GlobalRandomConfig, Seed};

/// The Classic Flat preset: bedrock, two layers of dirt, a grass block, and the
/// `plains` biome. Used when the preset string is empty.
const CLASSIC_FLAT_PRESET: &str =
    "minecraft:bedrock,2*minecraft:dirt,minecraft:grass_block;minecraft:plains";

/// Resolved settings for the flat (superflat) world generator.
///
/// Block and biome names from the preset string are resolved to concrete game
/// data here so chunk generation never has to perform name lookups.
pub struct FlatGeneratorSettings {
    /// The block layers, ordered bottom-to-top: `(block state, thickness)`.
    pub layers: Vec<(&'static BlockState, u32)>,
    /// The biome applied to every column of the world.
    pub biome: &'static Biome,
    /// The combined thickness of every layer, in blocks.
    pub total_height: u32,
}

impl FlatGeneratorSettings {
    /// Parses a vanilla superflat preset string into concrete layers and a
    /// biome.
    ///
    /// The format mirrors vanilla's `generator-settings` legacy string:
    /// `layers;biome`, where `layers` is a comma-separated list ordered
    /// bottom-up and each entry is `block` or `count*block`. The `;biome`
    /// suffix is optional. An empty string selects the Classic Flat preset.
    ///
    /// Unknown block names fall back to air and unknown or missing biome names
    /// fall back to [`Biome::PLAINS`], each with a single warning rather than a
    /// panic.
    #[must_use]
    pub fn from_preset(preset: &str) -> Self {
        let preset = preset.trim();
        let preset = if preset.is_empty() {
            CLASSIC_FLAT_PRESET
        } else {
            preset
        };

        let (layers_part, biome_part) = match preset.split_once(';') {
            Some((layers, biome)) => (layers, Some(biome.trim())),
            None => (preset, None),
        };

        let biome = match biome_part.filter(|b| !b.is_empty()) {
            Some(name) => Self::resolve_biome(name),
            None => &Biome::PLAINS,
        };

        let mut layers = Vec::new();
        let mut total_height = 0;
        for entry in layers_part.split(',') {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            let (count, block_name) = match entry.split_once('*') {
                Some((count, block)) => {
                    let parsed = count.trim().parse::<u32>().unwrap_or_else(|_| {
                        tracing::warn!(
                            "Invalid flat world layer count `{count}` in `{entry}`, using 1"
                        );
                        1
                    });
                    (parsed, block.trim())
                }
                None => (1, entry),
            };
            if count == 0 {
                continue;
            }
            let block = Self::resolve_block(block_name);
            layers.push((block.default_state, count));
            total_height += count;
        }

        Self {
            layers,
            biome,
            total_height,
        }
    }

    /// Resolves a block name, accepting an optional `minecraft:` prefix and
    /// warning before falling back to air.
    fn resolve_block(name: &str) -> &'static Block {
        Block::from_name(name).unwrap_or_else(|| {
            tracing::warn!("Unknown flat world block `{name}`, falling back to minecraft:air");
            &Block::AIR
        })
    }

    /// Resolves a biome name, accepting an optional `minecraft:` prefix and
    /// warning before falling back to plains.
    fn resolve_biome(name: &str) -> &'static Biome {
        let stripped = name.strip_prefix("minecraft:").unwrap_or(name);
        Biome::from_name(stripped).unwrap_or_else(|| {
            tracing::warn!("Unknown flat world biome `{name}`, falling back to minecraft:plains");
            &Biome::PLAINS
        })
    }
}

/// A superflat world generator producing fixed horizontal layers with a single
/// biome, no terrain noise, carvers, or structures.
pub struct FlatGenerator {
    /// Shared random configuration, kept for parity with the vanilla pipeline
    /// stages that still run (lighting, spawning, finalization).
    pub random_config: GlobalRandomConfig,
    /// The dimension this generator produces chunks for.
    pub dimension: Dimension,
    /// The resolved flat layers and biome.
    pub settings: FlatGeneratorSettings,
    /// The dimension's default block, required when constructing proto chunks.
    pub default_block: &'static BlockState,
    /// Seed used by the biome mixer, required when constructing proto chunks.
    pub biome_mixer_seed: i64,
}

impl FlatGenerator {
    /// Builds a flat generator for `dimension` from the given seed and preset.
    ///
    /// `preset` is the vanilla superflat `generator-settings` string; an empty
    /// value selects the Classic Flat preset. See
    /// [`FlatGeneratorSettings::from_preset`].
    #[must_use]
    pub fn new(seed: Seed, dimension: Dimension, preset: &str) -> Self {
        let gen_settings = GenerationSettings::from_dimension(&dimension);
        let random_config = GlobalRandomConfig::new(seed.0, gen_settings.legacy_random_source);
        let biome_mixer_seed = crate::biome::hash_seed(seed.0);
        let settings = FlatGeneratorSettings::from_preset(preset);

        Self {
            random_config,
            dimension,
            settings,
            default_block: gen_settings.default_block,
            biome_mixer_seed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FlatGeneratorSettings;
    use pumpkin_data::Block;
    use pumpkin_data::chunk::Biome;

    #[test]
    fn empty_preset_resolves_to_classic_flat() {
        let settings = FlatGeneratorSettings::from_preset("");

        assert_eq!(settings.biome.id, Biome::PLAINS.id);
        assert_eq!(settings.total_height, 4);

        let expected = [
            (Block::BEDROCK.default_state.id, 1),
            (Block::DIRT.default_state.id, 2),
            (Block::GRASS_BLOCK.default_state.id, 1),
        ];
        let actual: Vec<_> = settings
            .layers
            .iter()
            .map(|(state, height)| (state.id, *height))
            .collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn preset_string_with_counts_and_biome_parses() {
        let settings = FlatGeneratorSettings::from_preset(
            "minecraft:bedrock,3*minecraft:stone;minecraft:desert",
        );

        let expected = [
            (Block::BEDROCK.default_state.id, 1),
            (Block::STONE.default_state.id, 3),
        ];
        let actual: Vec<_> = settings
            .layers
            .iter()
            .map(|(state, height)| (state.id, *height))
            .collect();
        assert_eq!(actual, expected);
        assert_eq!(settings.total_height, 4);
        assert_eq!(settings.biome.id, Biome::DESERT.id);
    }

    #[test]
    fn block_names_without_namespace_resolve() {
        let settings = FlatGeneratorSettings::from_preset("2*dirt;plains");
        assert_eq!(settings.layers[0].0.id, Block::DIRT.default_state.id);
        assert_eq!(settings.layers[0].1, 2);
        assert_eq!(settings.biome.id, Biome::PLAINS.id);
    }

    #[test]
    fn unknown_block_falls_back_to_air() {
        let settings = FlatGeneratorSettings::from_preset("3*minecraft:not_a_real_block");
        assert_eq!(settings.layers[0].0.id, Block::AIR.default_state.id);
        assert_eq!(settings.layers[0].1, 3);
    }

    #[test]
    fn missing_biome_defaults_to_plains() {
        let settings = FlatGeneratorSettings::from_preset("minecraft:stone");
        assert_eq!(settings.biome.id, Biome::PLAINS.id);
    }

    #[test]
    fn zero_count_layers_are_skipped() {
        let settings = FlatGeneratorSettings::from_preset("0*minecraft:stone");
        assert!(settings.layers.is_empty());
        assert_eq!(settings.total_height, 0);
    }
}
