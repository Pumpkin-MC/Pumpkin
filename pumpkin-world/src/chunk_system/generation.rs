use pumpkin_data::chunk_gen_settings::GenerationSettings;
use pumpkin_data::dimension::Dimension;

use crate::ProtoChunk;
use crate::generation::generator::VanillaGenerator;
use crate::world::BlockRegistryExt;
use pumpkin_config::lighting::LightingEngineConfig;

use super::{Cache, Chunk, StagedChunkEnum};

fn prepare_proto_chunk_to_noise(
    proto_chunk: &mut ProtoChunk,
    dimension: &Dimension,
    settings: &GenerationSettings,
    generator: &VanillaGenerator,
) {
    proto_chunk.set_structure_starts(
        &generator.random_config,
        settings,
        dimension,
        &generator.base_router,
        &generator.global_structure_cache,
    );
    proto_chunk.set_structure_references(
        &generator.random_config,
        settings,
        dimension,
        &generator.base_router,
        &generator.global_structure_cache,
    );
    proto_chunk.step_to_biomes(*dimension, &generator.base_router);
    proto_chunk.step_to_noise(settings, &generator.random_config, &generator.base_router);
}

fn prepare_proto_chunk_to_surface(
    proto_chunk: &mut ProtoChunk,
    dimension: &Dimension,
    settings: &GenerationSettings,
    generator: &VanillaGenerator,
) {
    prepare_proto_chunk_to_noise(proto_chunk, dimension, settings, generator);
    proto_chunk.step_to_surface(
        settings,
        &generator.random_config,
        &generator.terrain_cache,
        &generator.base_router,
    );
}

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

    let pre_surface_cache = radius > 0 && target_stage as u8 >= StagedChunkEnum::Surface as u8;
    if pre_surface_cache {
        for chunk in &mut cache.chunks {
            prepare_proto_chunk_to_surface(
                chunk.get_proto_chunk_mut(),
                dimension,
                settings,
                generator,
            );
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
        if pre_surface_cache && stage as u8 <= StagedChunkEnum::Surface as u8 {
            continue;
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
            &generator.global_structure_cache,
        );
    }

    let mid = ((cache.size * cache.size) >> 1) as usize;
    cache.chunks.swap_remove(mid)
}

#[cfg(test)]
mod tests {
    use super::prepare_proto_chunk_to_noise;
    use crate::ProtoChunk;
    use crate::biome::hash_seed;
    use crate::chunk_system::{Cache, Chunk, StagedChunkEnum, generate_single_chunk};
    use crate::generation::get_world_gen;
    use crate::world::BlockRegistryExt;
    use pumpkin_config::lighting::LightingEngineConfig;
    use pumpkin_data::chunk_gen_settings::GenerationSettings;
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

    #[test]
    fn surface_stage_advances_neighbor_chunks() {
        let dimension = Dimension::OVERWORLD;
        let seed = Seed(42);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension);
        let biome_mixer_seed = hash_seed(world_gen.random_config.seed);
        let settings = GenerationSettings::from_dimension(&dimension);

        let mut cache = Cache::new(-1, -1, 3);
        for dx in -1..=1 {
            for dz in -1..=1 {
                cache.chunks.push(Chunk::Proto(Box::new(ProtoChunk::new(
                    dx,
                    dz,
                    &dimension,
                    world_gen.default_block,
                    biome_mixer_seed,
                ))));
            }
        }

        for chunk in &mut cache.chunks {
            prepare_proto_chunk_to_noise(
                chunk.get_proto_chunk_mut(),
                &dimension,
                settings,
                &world_gen,
            );
        }

        for chunk in &cache.chunks {
            assert_eq!(chunk.get_proto_chunk().stage, StagedChunkEnum::Noise);
        }

        cache.advance(
            StagedChunkEnum::Surface,
            &LightingEngineConfig::Default,
            block_registry.as_ref(),
            settings,
            &world_gen.random_config,
            &world_gen.terrain_cache,
            &world_gen.base_router,
            dimension,
            &world_gen.global_structure_cache,
        );

        for chunk in &cache.chunks {
            assert_eq!(chunk.get_proto_chunk().stage, StagedChunkEnum::Surface);
        }
    }
}
