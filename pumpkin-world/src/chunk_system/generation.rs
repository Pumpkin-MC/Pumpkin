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

#[cfg(test)]
mod tests {
    use crate::biome::hash_seed;
    use crate::chunk_system::{StagedChunkEnum, generate_single_chunk};
    use crate::generation::get_world_gen;
    use crate::world::BlockRegistryExt;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::world_seed::Seed;
    use std::sync::Arc;

    struct BlockRegistry;
    impl BlockRegistryExt for BlockRegistry {
        fn can_place_at(
            &self,
            _block: &pumpkin_data::Block,
            _state: &pumpkin_data::BlockState,
            _block_accessor: &dyn crate::world::BlockAccessor,
            _block_pos: &pumpkin_util::math::position::BlockPos,
        ) -> bool {
            true
        }
    }

    #[test]
    fn generate_chunk_should_return() {
        let dimension = Dimension::OVERWORLD;
        let seed = Seed(42);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension);
        let biome_mixer_seed = hash_seed(world_gen.random_config.seed);

        let _ = generate_single_chunk(
            &dimension,
            biome_mixer_seed,
            &world_gen,
            block_registry.as_ref(),
            0,
            0,
            StagedChunkEnum::Full,
        );
    }
}
