use std::sync::Arc;

use pumpkin_codecs::{DataResult, Decode, DynamicOps, MapLike, codec::FieldDecode};
use pumpkin_data::chunk::Biome;
use pumpkin_nbt::{nbt_ops::NbtOps, tag::NbtTag};
use pumpkin_registry::{
    DataKey, Registry, RegistryBuilder, bootstrap::RegistryEntry, bootstrap_provider,
};
use pumpkin_util::identifier::Identifier;

use super::{BiomeSupplier, MultiNoiseBiomeSupplier, end::TheEndBiomeSupplier};
use crate::generation::noise::router::multi_noise_sampler::MultiNoiseSampler;

type DecodeBiomeSource = fn(NbtTag) -> DataResult<Box<dyn BiomeSupplier>>;

pub struct BiomeSourceType {
    decode: DecodeBiomeSource,
}

impl BiomeSourceType {
    #[must_use]
    pub const fn new<T>() -> Self
    where
        T: BiomeSupplier + Decode + 'static,
    {
        Self {
            decode: decode_biome_source::<T>,
        }
    }

    pub fn decode<O: DynamicOps>(
        &self,
        input: O::Value,
        ops: &'static O,
    ) -> DataResult<Box<dyn BiomeSupplier>> {
        (self.decode)(ops.convert_to(&NbtOps, input))
    }
}

fn decode_biome_source<T>(input: NbtTag) -> DataResult<Box<dyn BiomeSupplier>>
where
    T: BiomeSupplier + Decode + 'static,
{
    T::parse(input, &NbtOps).map(|source| Box::new(source) as Box<dyn BiomeSupplier>)
}

pub struct BiomeSourceConfig {
    pub source_type: DataKey<BiomeSourceType>,
    pub input: NbtTag,
}

impl BiomeSourceConfig {
    pub fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<Self> {
        ops.get_map(&input.clone()).flat_map(|map| {
            let Some(kind_value) = map.get_str("type").cloned() else {
                return DataResult::new_error("Missing biome source type");
            };

            ops.get_string(&kind_value).map(|kind| {
                let source_type = DataKey::owned(format!(
                    "minecraft:worldgen/minecraft:biome_source_type/{kind}"
                ));
                Self {
                    source_type,
                    input: ops.convert_to(&NbtOps, input),
                }
            })
        })
    }
}

pub struct FixedBiomeSupplier {
    biome: &'static Biome,
}

impl BiomeSupplier for FixedBiomeSupplier {
    fn biome(
        &self,
        _x: i32,
        _y: i32,
        _z: i32,
        _noise: &mut MultiNoiseSampler<'_>,
    ) -> &'static Biome {
        self.biome
    }
}

impl Decode for FixedBiomeSupplier {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            String::decode_field::<O>("biome", &map, ops).flat_map(|biome| {
                let name = biome.strip_prefix("minecraft:").unwrap_or(&biome);
                Biome::from_name(name).map_or_else(
                    || DataResult::new_error(format!("Unknown fixed biome: {biome}")),
                    |biome| DataResult::new_success((Self { biome }, ops.empty())),
                )
            })
        })
    }
}

impl Decode for MultiNoiseBiomeSupplier {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).flat_map(|map| {
            String::decode_field::<O>("preset", &map, ops).flat_map(|preset| {
                let preset = preset.strip_prefix("minecraft:").unwrap_or(&preset);
                match preset {
                    "overworld" => DataResult::new_success((Self::OVERWORLD, ops.empty())),
                    "nether" => DataResult::new_success((Self::NETHER, ops.empty())),
                    other => DataResult::new_error(format!(
                        "Unsupported multi_noise biome source preset: {other}"
                    )),
                }
            })
        })
    }
}

impl Decode for TheEndBiomeSupplier {
    fn decode<O: DynamicOps>(input: O::Value, ops: &'static O) -> DataResult<(Self, O::Value)> {
        ops.get_map(&input).map(|_| (Self, ops.empty()))
    }
}

bootstrap_provider! {
    BIOME_SOURCE_TYPES: BiomeSourceType => "minecraft:worldgen/biome_source_type" => {
        "minecraft:fixed" => BiomeSourceType::new::<FixedBiomeSupplier>(),
        "minecraft:multi_noise" => BiomeSourceType::new::<MultiNoiseBiomeSupplier>(),
        "minecraft:the_end" => BiomeSourceType::new::<TheEndBiomeSupplier>(),
    }
}

bootstrap_provider! {
    BIOME_SOURCE_TYPE_REGISTRY: Arc<dyn Registry> => "minecraft:worldgen",
    || {
        let Ok(registry) = RegistryBuilder::<BiomeSourceType>::frozen(
            &Identifier::parse_static("minecraft:worldgen/biome_source_type"),
        ) else {
            return Vec::new();
        };

        vec![RegistryEntry::new(
            Identifier::vanilla_static("biome_source_type"),
            registry.arc_dyn(),
        )]
    }
}
