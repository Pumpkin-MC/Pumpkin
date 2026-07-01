use pumpkin_data::dimension::Dimension;

use crate::ProtoChunk;
use crate::generation::generator::VanillaGenerator;
use crate::world::WorldPortalExt;
use pumpkin_config::lighting::LightingEngineConfig;

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
    use crate::biome::hash_seed;
    use crate::chunk_system::{StagedChunkEnum, generate_single_chunk};
    use crate::generation::get_world_gen;
    use crate::world::WorldPortalExt;
    use pumpkin_data::BlockStateId;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::world_seed::Seed;
    use std::sync::Arc;

    struct BlockRegistry;
    impl WorldPortalExt for BlockRegistry {
        fn can_place_at(
            &self,
            _block: &pumpkin_data::Block,
            _state: &pumpkin_data::BlockState,
            _block_accessor: &dyn crate::world::BlockAccessor,
            _block_pos: &pumpkin_util::math::position::BlockPos,
        ) -> bool {
            true
        }

        fn mirror(
            &self,
            block: &pumpkin_data::Block,
            state_id: BlockStateId,
            mirror: pumpkin_data::Mirror,
        ) -> &'static pumpkin_data::BlockState {
            block.mirror(state_id, mirror)
        }

        fn rotate(
            &self,
            block: &pumpkin_data::Block,
            state_id: BlockStateId,
            rotation: pumpkin_data::Rotation,
        ) -> &'static pumpkin_data::BlockState {
            block.rotate(state_id, rotation)
        }

        fn spawn_mobs_for_chunk_generation(
            &self,
            _cache: &mut dyn crate::generation::proto_chunk::GenerationCache,
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

    #[test]
    fn configured_seed_generates_vanilla_ancient_city_chunk() {
        let dimension = Dimension::OVERWORLD;
        let seed = Seed(1_782_124_772_053_846_960);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension.clone());
        let biome_mixer_seed = hash_seed(world_gen.random_config.seed);

        let chunk = generate_single_chunk(
            &dimension,
            biome_mixer_seed,
            &world_gen,
            block_registry.as_ref(),
            31,
            -12,
            StagedChunkEnum::Features,
        );
        let super::Chunk::Proto(chunk) = chunk else {
            panic!("features stage should return a proto chunk");
        };

        let mut city_blocks = 0;
        let mut jigsaw_blocks = 0;
        for x in 496..512 {
            for z in -192..-176 {
                for y in -64..320 {
                    let block = chunk
                        .get_block_state(&pumpkin_util::math::vector3::Vector3::new(x, y, z))
                        .to_block_id();
                    if [
                        pumpkin_data::Block::DEEPSLATE_BRICKS.id,
                        pumpkin_data::Block::POLISHED_DEEPSLATE.id,
                        pumpkin_data::Block::REINFORCED_DEEPSLATE.id,
                        pumpkin_data::Block::SCULK.id,
                    ]
                    .contains(&block)
                    {
                        city_blocks += 1;
                    }
                    if block == pumpkin_data::Block::JIGSAW.id {
                        jigsaw_blocks += 1;
                    }
                }
            }
        }

        assert!(
            city_blocks > 0,
            "reference chunk contains no Ancient City blocks"
        );
        assert_eq!(jigsaw_blocks, 0, "jigsaw blocks were not replaced");
    }
}
