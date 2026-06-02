use pumpkin_data::dimension::Dimension;

use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_world::ProtoChunk;
use pumpkin_world::generation::generator::VanillaGenerator;
use pumpkin_world::world::WorldPortalExt;

use super::{Cache, Chunk, StagedChunkEnum};

pub fn generate_single_chunk(
    _dimension: &Dimension,
    _biome_mixer_seed: i64,
    generator: &VanillaGenerator,
    block_registry: &dyn WorldPortalExt,
    chunk_x: i32,
    chunk_z: i32,
    target_stage: StagedChunkEnum,
) -> Chunk {
    let radius = target_stage.get_direct_radius();

    let mut cache = Cache::new(chunk_x - radius, chunk_z - radius, radius * 2 + 1);

    for dx in -radius..=radius {
        for dz in -radius..=radius {
            let new_x = chunk_x + dx;
            let new_z = chunk_z + dz;

            let proto_chunk = Box::new(ProtoChunk::new(new_x, new_z, generator));

            cache.chunks.push(Chunk::Proto(proto_chunk));
        }
    }

    let stages = [
        StagedChunkEnum::Biomes,
        StagedChunkEnum::StructureStart,
        StagedChunkEnum::StructureReferences,
        StagedChunkEnum::Noise,
        StagedChunkEnum::Surface,
        StagedChunkEnum::Carvers,
        StagedChunkEnum::Features,
        StagedChunkEnum::Lighting,
        StagedChunkEnum::Spawn,
        StagedChunkEnum::Full,
    ];

    for &stage in &stages {
        if stage as u8 > target_stage as u8 {
            break;
        }

        cache.advance(
            stage,
            generator,
            block_registry,
            &LightingEngineConfig::Default,
        );
    }

    let mid = ((cache.size * cache.size) >> 1) as usize;
    cache.chunks.swap_remove(mid)
}

#[cfg(test)]
mod tests {
    use crate::chunk_system::{StagedChunkEnum, generate_single_chunk};
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::world_seed::Seed;
    use pumpkin_world::biome::hash_seed;
    use pumpkin_world::generation::get_world_gen;
    use pumpkin_world::generation::proto_chunk::GenerationCache;
    use pumpkin_world::world::{BlockAccessor, WorldPortalExt};
    use std::sync::Arc;

    struct BlockRegistry;
    impl WorldPortalExt for BlockRegistry {
        fn can_place_at(
            &self,
            _block: &pumpkin_data::Block,
            _state: &pumpkin_data::BlockState,
            _block_accessor: &dyn BlockAccessor,
            _block_pos: &BlockPos,
        ) -> bool {
            true
        }

        fn spawn_mobs_for_chunk_generation(
            &self,
            _cache: &mut dyn GenerationCache,
            _biome: &'static pumpkin_data::chunk::Biome,
            _chunk_x: i32,
            _chunk_z: i32,
        ) {
        }
    }

    #[test]
    fn generate_chunk_should_return() {
        let dimension = Dimension::OVERWORLD;
        let seed = Seed(42);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension.clone());
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
