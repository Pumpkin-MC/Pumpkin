use std::sync::Arc;

use pumpkin_codecs::{DataResult, Decode, DynamicOps};
use pumpkin_data::{Block, BlockState, dimension::Dimension};
use pumpkin_nbt::{nbt_ops::NbtOps, tag::NbtTag};
use pumpkin_registry::{Registry, RegistryBuilder, bootstrap::RegistryEntry, bootstrap_provider};
use pumpkin_util::{identifier::Identifier, world_seed::Seed};

use super::{flat::FlatGenerator, vanilla::VanillaGenerator};
use crate::{ProtoChunk, chunk_system::generation_cache::Cache, world::WorldPortalExt};

pub trait ChunkGenerator: Send + Sync {
    fn dimension(&self) -> &Dimension;

    fn seed(&self) -> u64;

    fn generation_bounds(&self) -> (u16, i8) {
        (self.dimension().height as u16, self.dimension().min_y as i8)
    }

    fn default_block(&self) -> &'static BlockState {
        Block::AIR.default_state
    }

    fn biome_mixer_seed(&self) -> i64 {
        crate::biome::hash_seed(self.seed())
    }

    fn step_to_biomes(&self, chunk: &mut ProtoChunk);

    fn step_to_structure_start(
        &self,
        cache: &mut Cache,
        chunk_index: usize,
        block_registry: &dyn WorldPortalExt,
    );

    fn step_to_structure_references(
        &self,
        cache: &mut Cache,
        chunk_index: usize,
        block_registry: &dyn WorldPortalExt,
    );

    fn rebuild_structure_starts(&self, chunk: &mut ProtoChunk);

    fn rebuild_structure_references(&self, chunk: &mut ProtoChunk);

    fn step_to_noise(&self, chunk: &mut ProtoChunk);

    fn step_to_surface(&self, chunk: &mut ProtoChunk);

    fn step_to_carvers(&self, chunk: &mut ProtoChunk);

    fn step_to_features(
        &self,
        cache: &mut Cache,
        chunk_index: usize,
        block_registry: &dyn WorldPortalExt,
    );
}

pub trait ChunkGeneratorDecode: ChunkGenerator + Sized {
    type Config: Decode;

    fn from_config(seed: Seed, dimension: Dimension, config: Self::Config) -> DataResult<Self>;
}

type DecodeChunkGenerator = fn(NbtTag, Seed, Dimension) -> DataResult<Box<dyn ChunkGenerator>>;

pub struct ChunkGeneratorType {
    decode: DecodeChunkGenerator,
}

impl ChunkGeneratorType {
    #[must_use]
    pub const fn new<T>() -> Self
    where
        T: ChunkGeneratorDecode + 'static,
    {
        Self {
            decode: decode_chunk_generator::<T>,
        }
    }

    pub fn decode<O: DynamicOps>(
        &self,
        input: O::Value,
        ops: &'static O,
        seed: Seed,
        dimension: Dimension,
    ) -> DataResult<Box<dyn ChunkGenerator>> {
        let input = ops.convert_to(&NbtOps, input);

        (self.decode)(input, seed, dimension)
    }
}

fn decode_chunk_generator<T>(
    input: NbtTag,
    seed: Seed,
    dimension: Dimension,
) -> DataResult<Box<dyn ChunkGenerator>>
where
    T: ChunkGeneratorDecode + 'static,
{
    T::Config::parse(input, &NbtOps)
        .flat_map(|config| T::from_config(seed, dimension, config))
        .map(|generator| Box::new(generator) as Box<dyn ChunkGenerator>)
}

bootstrap_provider! {
    CHUNK_GENERATOR_TYPES: ChunkGeneratorType => "minecraft:worldgen/chunk_generator_type" => {
        "minecraft:noise" => ChunkGeneratorType::new::<VanillaGenerator>(),
        "minecraft:flat" => ChunkGeneratorType::new::<FlatGenerator>(),
    }
}

bootstrap_provider! {
    CHUNK_GENERATOR_TYPE_REGISTRY: Arc<dyn Registry> => "minecraft:worldgen",
    || {
        let Ok(registry) = RegistryBuilder::<ChunkGeneratorType>::frozen(
            &Identifier::parse_static("minecraft:worldgen/chunk_generator_type"),
        ) else {
            return Vec::new();
        };

        vec![RegistryEntry::new(
            Identifier::vanilla_static("chunk_generator_type"),
            registry.arc_dyn(),
        )]
    }
}
