use pumpkin_codecs::{DataResult, Decode, DynamicOps};
use pumpkin_data::dimension::Dimension;
use pumpkin_nbt::{nbt_ops::NbtOps, tag::NbtTag};
use pumpkin_util::world_seed::Seed;

use crate::{ProtoChunk, chunk_system::generation_cache::Cache, world::WorldPortalExt};

pub trait ChunkGenerator: Send + Sync {
    fn dimension(&self) -> &Dimension;

    fn seed(&self) -> u64;

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
