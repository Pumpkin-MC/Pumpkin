use pumpkin_codecs::{DataResult, Decode, DynamicOps, MapLike, codec::FieldDecode};
use pumpkin_data::dimension::Dimension;
use pumpkin_nbt::{nbt_ops::NbtOps, tag::NbtTag};
use pumpkin_registry::DataKey;

use super::generator::ChunkGeneratorType;

pub struct ChunkGeneratorConfig {
    pub generator_type: DataKey<ChunkGeneratorType>,
    pub input: NbtTag,
}

impl ChunkGeneratorConfig {
    pub fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<Self> {
        ops.get_map(&input.clone()).flat_map(|map| {
            String::decode_field::<O>("type", &map, ops).map(|kind| Self {
                generator_type: DataKey::owned(format!(
                    "minecraft:worldgen/minecraft:chunk_generator_type/{kind}"
                )),
                input: ops.convert_to(&NbtOps, input),
            })
        })
    }
}

pub struct DimensionStem {
    pub dimension_type: DataKey<Dimension>,
    pub generator: ChunkGeneratorConfig,
}

impl Decode for DimensionStem {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            let dimension_type = String::decode_field::<O>("type", &map, ops)
                .map(|kind| DataKey::owned(format!("minecraft:dimension_type/{kind}")));

            let generator = map.get_str("generator").cloned().map_or_else(
                || DataResult::new_error("Missing dimension generator"),
                |input| ChunkGeneratorConfig::decode(input, ops),
            );

            dimension_type.apply_2(
                |dimension_type, generator| {
                    (
                        Self {
                            dimension_type,
                            generator,
                        },
                        ops.empty(),
                    )
                },
                generator,
            )
        })
    }
}
