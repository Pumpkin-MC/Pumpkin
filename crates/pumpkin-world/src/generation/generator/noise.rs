use pumpkin_codecs::{DataResult, Decode, DynamicOps, MapLike, codec::FieldDecode};
use pumpkin_data::chunk_gen_settings::GenerationSettings;
use pumpkin_registry::DataKey;

use crate::biome::BiomeSourceConfig;

pub struct NoiseGeneratorConfig {
    pub settings: DataKey<GenerationSettings>,
    pub biome_source: BiomeSourceConfig,
}

impl Decode for NoiseGeneratorConfig {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            let settings = String::decode_field::<O>("settings", &map, ops).map(|settings| {
                DataKey::owned(format!(
                    "minecraft:worldgen/minecraft:noise_settings/{settings}"
                ))
            });

            let biome_source = map.get_str("biome_source").cloned().map_or_else(
                || DataResult::new_error("Missing biome_source"),
                |input| BiomeSourceConfig::decode(input, ops),
            );

            settings.apply_2(
                |settings, biome_source| {
                    (
                        Self {
                            settings,
                            biome_source,
                        },
                        ops.empty(),
                    )
                },
                biome_source,
            )
        })
    }
}
