use pumpkin_data::chunk_gen_settings::GenerationSettings;
use pumpkin_data::dimension::Dimension;

use crate::ProtoChunk;
use crate::generation::generator::VanillaGenerator;
use crate::world::BlockRegistryExt;
use pumpkin_config::lighting::LightingEngineConfig;

use super::{Cache, Chunk, StagedChunkEnum};

pub fn generate_single_chunk(
    dimension: &Dimension,
    biome_mixer_seed: i64,
    generator: &VanillaGenerator,
    block_registry: &dyn BlockRegistryExt,
    chunk_x: i32,
    chunk_z: i32,
    target_stage: StagedChunkEnum,
) -> Chunk {
    let settings = GenerationSettings::from_dimension(dimension);
    let radius = target_stage.get_direct_radius();

    let mut cache = Cache::new(chunk_x - radius, chunk_z - radius, radius * 2 + 1);

    for dx in -radius..=radius {
        for dz in -radius..=radius {
            let new_x = chunk_x + dx;
            let new_z = chunk_z + dz;

            let proto_chunk = Box::new(ProtoChunk::new(
                new_x,
                new_z,
                dimension,
                generator.default_block,
                biome_mixer_seed,
            ));

            cache.chunks.push(Chunk::Proto(proto_chunk));
        }
    }

    let stages = [
        StagedChunkEnum::StructureStart,
        StagedChunkEnum::StructureReferences,
        StagedChunkEnum::Biomes,
        StagedChunkEnum::Noise,
        StagedChunkEnum::Surface,
        StagedChunkEnum::Features,
        StagedChunkEnum::Lighting,
        StagedChunkEnum::Full,
    ];

    for &stage in &stages {
        if stage as u8 > target_stage as u8 {
            break;
        }

        cache.advance(
            stage,
            &LightingEngineConfig::Default,
            block_registry,
            settings,
            &generator.random_config,
            &generator.terrain_cache,
            &generator.base_router,
            *dimension,
        );
    }

    let mid = ((cache.size * cache.size) >> 1) as usize;
    cache.chunks.swap_remove(mid)
}
