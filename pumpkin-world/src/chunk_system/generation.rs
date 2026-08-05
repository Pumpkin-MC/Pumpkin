use pumpkin_data::dimension::Dimension;

use crate::ProtoChunk;
use crate::generation::generator::WorldGenerator;
use crate::world::WorldPortalExt;
use pumpkin_config::lighting::LightingEngineConfig;

use super::{Cache, Chunk, StagedChunkEnum};

pub fn generate_single_chunk(
    dimension: &Dimension,
    biome_mixer_seed: i64,
    generator: &WorldGenerator,
    block_registry: &dyn WorldPortalExt,
    chunk_x: i32,
    chunk_z: i32,
    target_stage: StagedChunkEnum,
) -> Chunk {
    generate_single_chunk_with_radius(
        dimension,
        biome_mixer_seed,
        generator,
        block_registry,
        chunk_x,
        chunk_z,
        target_stage,
        target_stage.get_direct_radius(),
    )
}

#[expect(clippy::too_many_arguments)]
pub fn generate_single_chunk_with_radius(
    _dimension: &Dimension,
    _biome_mixer_seed: i64,
    generator: &WorldGenerator,
    block_registry: &dyn WorldPortalExt,
    chunk_x: i32,
    chunk_z: i32,
    target_stage: StagedChunkEnum,
    radius: i32,
) -> Chunk {
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

        if matches!(
            stage,
            StagedChunkEnum::Biomes
                | StagedChunkEnum::StructureStart
                | StagedChunkEnum::StructureReferences
        ) {
            cache.advance_all(
                stage,
                generator,
                block_registry,
                &LightingEngineConfig::Default,
            );
        } else {
            cache.advance(
                stage,
                generator,
                block_registry,
                &LightingEngineConfig::Default,
            );
        }
    }

    let mid = ((cache.size * cache.size) >> 1) as usize;
    cache.chunks.swap_remove(mid)
}

#[cfg(test)]
mod tests {
    use crate::biome::hash_seed;
    use crate::chunk::ChunkHeightmapType;
    use crate::chunk_system::Chunk;
    use crate::chunk_system::{
        StagedChunkEnum, generate_single_chunk, generation::generate_single_chunk_with_radius,
    };
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
        let world_gen = get_world_gen(seed, dimension.clone(), false, Vec::new(), String::new());
        let biome_mixer_seed = hash_seed(world_gen.seed());

        let chunk = generate_single_chunk(
            &dimension,
            biome_mixer_seed,
            &world_gen,
            block_registry.as_ref(),
            0,
            0,
            StagedChunkEnum::Full,
        );
        let Chunk::Level(chunk) = chunk else {
            panic!("full generation must return a level chunk");
        };
        let recalculated = chunk.calculate_heightmap();
        let generated = chunk.heightmap.lock().unwrap();
        for x in 0..16 {
            for z in 0..16 {
                for heightmap_type in ChunkHeightmapType::ALL {
                    assert_eq!(
                        generated.get(heightmap_type, x, z, chunk.section.min_y),
                        recalculated.get(heightmap_type, x, z, chunk.section.min_y),
                    );
                }
            }
        }
    }

    #[test]
    fn configured_seed_generates_vanilla_ancient_city_chunk() {
        let dimension = Dimension::OVERWORLD;
        let seed = Seed(1_782_124_772_053_846_960);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension.clone(), false, Vec::new(), String::new());
        let biome_mixer_seed = hash_seed(world_gen.seed());

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

    #[test]
    fn seed_zero_generates_the_vanilla_pillager_outpost_chunk() {
        let dimension = Dimension::OVERWORLD;
        let seed = Seed(1_782_124_772_053_846_960);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension.clone(), false, Vec::new(), String::new());
        let biome_mixer_seed = hash_seed(world_gen.seed());

        let chunk = generate_single_chunk_with_radius(
            &dimension,
            biome_mixer_seed,
            &world_gen,
            block_registry.as_ref(),
            73,
            -82,
            StagedChunkEnum::Spawn,
            16,
        );
        let super::Chunk::Proto(chunk) = chunk else {
            panic!("spawn stage should return a proto chunk");
        };
        let mut outpost_blocks = 0;
        let mut jigsaw_blocks = 0;
        for x in 1168..1184 {
            for z in -1328..-1312 {
                for y in -64..320 {
                    let block = chunk
                        .get_block_state(&pumpkin_util::math::vector3::Vector3::new(x, y, z))
                        .to_block_id();
                    if [
                        pumpkin_data::Block::DARK_OAK_LOG.id,
                        pumpkin_data::Block::DARK_OAK_PLANKS.id,
                        pumpkin_data::Block::DARK_OAK_FENCE.id,
                    ]
                    .contains(&block)
                    {
                        outpost_blocks += 1;
                    }
                    if block == pumpkin_data::Block::JIGSAW.id {
                        jigsaw_blocks += 1;
                    }
                }
            }
        }

        assert!(
            outpost_blocks > 0,
            "reference chunk contains no outpost blocks"
        );
        assert_eq!(jigsaw_blocks, 0, "jigsaw blocks were not replaced");
    }

    #[test]
    fn fixed_seed_generates_vanilla_end_ship_chunk() {
        // Vanilla 26.2 places this seed's ship in chunk (-306, -275).
        let dimension = Dimension::THE_END;
        let seed = Seed(12_345);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension.clone(), false, Vec::new(), String::new());
        let biome_mixer_seed = hash_seed(world_gen.seed());
        let chunk = generate_single_chunk_with_radius(
            &dimension,
            biome_mixer_seed,
            &world_gen,
            block_registry.as_ref(),
            -306,
            -275,
            StagedChunkEnum::Features,
            16,
        );
        let Chunk::Proto(chunk) = chunk else {
            panic!("features stage should return a proto chunk");
        };
        let mut non_air = 0;
        for y in 123..=146 {
            for x in -4896..=-4881 {
                for z in -4405..=-4393 {
                    let state =
                        chunk.get_block_state(&pumpkin_util::math::vector3::Vector3::new(x, y, z));
                    non_air += usize::from(!state.to_state().is_air());
                }
            }
        }
        assert_eq!(non_air, 14);
        assert!(chunk.pending_block_entities.iter().any(|nbt| {
            nbt.get_string("id") == Some("minecraft:skull")
                && nbt.get_int("x") == Some(-4888)
                && nbt.get_int("y") == Some(131)
                && nbt.get_int("z") == Some(-4399)
        }));
        assert_eq!(
            chunk
                .pending_block_entities
                .iter()
                .filter(
                    |nbt| nbt.get_string("LootTable") == Some("minecraft:chests/end_city_treasure")
                )
                .count(),
            2
        );
    }

    #[test]
    fn pillager_outpost_features_shape_ground_at_vanilla_height() {
        let dimension = Dimension::OVERWORLD;
        let seed = Seed(1_782_124_772_053_846_960);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension.clone(), false, Vec::new(), String::new());
        let biome_mixer_seed = hash_seed(world_gen.seed());

        let chunk = generate_single_chunk(
            &dimension,
            biome_mixer_seed,
            &world_gen,
            block_registry.as_ref(),
            73,
            -82,
            StagedChunkEnum::Features,
        );
        let super::Chunk::Proto(chunk) = chunk else {
            panic!("features stage should return a proto chunk");
        };

        for (x, y, z) in [(1173, 70, -1311), (1173, 70, -1305)] {
            let state = chunk.get_block_state(&pumpkin_util::math::vector3::Vector3::new(x, y, z));
            assert_eq!(state.to_block_id(), pumpkin_data::Block::GRASS_BLOCK.id);
        }

        let cage_chunk = generate_single_chunk(
            &dimension,
            biome_mixer_seed,
            &world_gen,
            block_registry.as_ref(),
            73,
            -84,
            StagedChunkEnum::Features,
        );
        let super::Chunk::Proto(cage_chunk) = cage_chunk else {
            panic!("features stage should return a proto chunk");
        };
        let state =
            cage_chunk.get_block_state(&pumpkin_util::math::vector3::Vector3::new(1183, 68, -1330));
        assert_eq!(state.to_block_id(), pumpkin_data::Block::GRASS_BLOCK.id);
    }

    /// Regression test: `Chunk::build_level_sections` (in `chunk_state.rs`) originally sized its
    /// section-count loop off `Dimension::THE_NETHER.height` (256) while `ProtoChunk`'s internal
    /// storage was sized off `GenerationSettings::NETHER.shape.height` (128) - indexing 16
    /// sections into a `flat_block_map` sized for only 8 panicked with an out-of-bounds index on
    /// every Nether chunk finalized to `Full`, exactly what live testing hit entering the Nether.
    /// The first fix for that (shrinking the section loop to 8, matching the generated height)
    /// stopped the panic but under-sized the *network-facing* chunk: the client derives the
    /// section count it reads from the dimension registry's height (256 - real vanilla data,
    /// confirmed via `Dimension::THE_NETHER.height` in `pumpkin-data/src/generated/dimension.rs`,
    /// which is auto-generated from the game's own registry), not from how much of that space
    /// worldgen populated. Sending only 8 sections' worth of bytes while the client reads for 16
    /// desyncs the chunk-data/light packets, which live testing then hit as a "Network Protocol
    /// Error" disconnect on every Nether entry. The real fix keeps the full 16-section,
    /// dimension-height chunk, with the ungenerated upper 8 sections as air - matching vanilla's
    /// actual buildable-but-ungenerated space above the Nether's terrain ceiling.
    #[test]
    fn nether_full_generation_produces_a_level_chunk() {
        let dimension = Dimension::THE_NETHER;
        let seed = Seed(42);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension.clone(), false, Vec::new(), String::new());
        let biome_mixer_seed = hash_seed(world_gen.seed());

        let chunk = generate_single_chunk(
            &dimension,
            biome_mixer_seed,
            &world_gen,
            block_registry.as_ref(),
            0,
            0,
            StagedChunkEnum::Full,
        );
        let Chunk::Level(chunk) = chunk else {
            panic!("full generation must return a level chunk");
        };

        // Full dimension height (256 / 16 = 16 sections), not just the 128-block generation
        // height (8 sections) - this is what the client actually expects from the registry.
        let block_sections = chunk.section.block_sections.read().unwrap();
        assert_eq!(block_sections.len(), 16);
        // The light engine must be padded out to the same section count, or the light portion
        // of the chunk-data packet desyncs from the block portion even though each is
        // individually self-consistent.
        assert_eq!(
            chunk.light_engine.lock().unwrap().sky_light.len(),
            16,
            "light section count must match block section count for network serialization"
        );

        // Sections above the generated height (index 8 and up) must be air: worldgen never
        // populates them, and they must not leak stale/garbage data from the generation buffer.
        for section in block_sections.iter().skip(8) {
            for block_id in section {
                assert_eq!(
                    block_id,
                    pumpkin_data::Block::AIR.default_state.id,
                    "ungenerated section above the Nether's terrain ceiling must be air"
                );
            }
        }
    }

    /// Regression test for the `KelpFeature` head-cap bug
    /// (`generation::feature::features::kelp::can_cap_with_head`): every kelp column
    /// generated by worldgen must rest on solid ground or on more kelp beneath it - never
    /// on air or open water - and every column must be capped with a `KELP` head, not left
    /// as a bare `KELP_PLANT` body segment.
    #[test]
    fn generated_kelp_columns_are_supported_and_capped() {
        let dimension = Dimension::OVERWORLD;
        let seed = Seed(42);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension.clone(), false, Vec::new(), String::new());
        let biome_mixer_seed = hash_seed(world_gen.seed());

        // Chunk (2, 4) on this seed contains multiple generated kelp columns (verified by
        // direct probing).
        let chunk = generate_single_chunk(
            &dimension,
            biome_mixer_seed,
            &world_gen,
            block_registry.as_ref(),
            2,
            4,
            StagedChunkEnum::Features,
        );
        let crate::chunk_system::Chunk::Proto(chunk) = chunk else {
            panic!("features stage should return a proto chunk");
        };

        let is_kelp = |id: pumpkin_data::BlockId| {
            id == pumpkin_data::Block::KELP.id || id == pumpkin_data::Block::KELP_PLANT.id
        };

        let mut columns_checked = 0;
        for x in 32..48 {
            for z in 64..80 {
                let mut y = -64;
                while y < 100 {
                    let here = chunk
                        .get_block_state(&pumpkin_util::math::vector3::Vector3::new(x, y, z))
                        .to_block_id();
                    if is_kelp(here) {
                        // Found the bottom of a kelp column: the block directly below must be
                        // solid ground (not air, not water) - a floating column is the exact
                        // symptom this test guards against.
                        let below = chunk
                            .get_block_state(&pumpkin_util::math::vector3::Vector3::new(
                                x,
                                y - 1,
                                z,
                            ))
                            .to_block_id();
                        assert!(
                            below != pumpkin_data::Block::AIR.id
                                && below != pumpkin_data::Block::WATER.id,
                            "kelp column at ({x}, {y}, {z}) is floating: block below is {below:?}"
                        );

                        // Walk to the top of the contiguous column and confirm it ends in a
                        // KELP head, not a bare KELP_PLANT body (the bug this test targets).
                        let mut top_y = y;
                        let mut top_block = here;
                        loop {
                            let next = chunk
                                .get_block_state(&pumpkin_util::math::vector3::Vector3::new(
                                    x,
                                    top_y + 1,
                                    z,
                                ))
                                .to_block_id();
                            if !is_kelp(next) {
                                break;
                            }
                            top_y += 1;
                            top_block = next;
                        }
                        assert_eq!(
                            top_block,
                            pumpkin_data::Block::KELP.id,
                            "kelp column at ({x}, {y}, {z}) ends at y={top_y} without a KELP head cap"
                        );
                        columns_checked += 1;

                        y = top_y + 1;
                        continue;
                    }
                    y += 1;
                }
            }
        }
        assert!(
            columns_checked > 0,
            "reference chunk contains no kelp columns to verify"
        );
    }

    /// Regression test for `HugeFungusFeature`
    /// (`generation::feature::features::huge_fungus`). The feature used to be a hand-rolled
    /// approximation that ignored its configuration entirely: it coin-flipped crimson vs
    /// warped per placement, never checked `validBaseState` (so every one of the
    /// `count_on_every_layer(8)` candidates placed, overwriting terrain), and painted a
    /// solid 5x5 box of wart block for the hat.
    ///
    /// Vanilla (`HugeFungusFeature` / `TreeFeatures.CRIMSON_FUNGUS`, Mojang-named 1.21.4
    /// decompile) requires the block below the origin to be the configured nylium, takes its
    /// stem/hat/decor states from the configuration rather than a coin flip, and builds a
    /// mostly-hollow hat shell (interior columns get only a 0.2 chance of hat block).
    #[test]
    fn crimson_forest_huge_fungi_are_crimson_and_hollow_capped() {
        let dimension = Dimension::THE_NETHER;
        let seed = Seed(42);
        let block_registry = Arc::new(BlockRegistry);
        let world_gen = get_world_gen(seed, dimension.clone(), false, Vec::new(), String::new());
        let biome_mixer_seed = hash_seed(world_gen.seed());

        // Chunk (5, 6) on this seed is solidly crimson forest: it carries crimson nylium and
        // a large number of huge fungi (verified by direct probing).
        let chunk = generate_single_chunk(
            &dimension,
            biome_mixer_seed,
            &world_gen,
            block_registry.as_ref(),
            5,
            6,
            StagedChunkEnum::Features,
        );
        let crate::chunk_system::Chunk::Proto(chunk) = chunk else {
            panic!("features stage should return a proto chunk");
        };

        let at = |x: i32, y: i32, z: i32| {
            chunk
                .get_block_state(&pumpkin_util::math::vector3::Vector3::new(x, y, z))
                .to_block_id()
        };

        let mut crimson_hat = 0;
        let mut warped_blocks = 0;
        let mut hollow_interior = 0;
        for x in 80..96 {
            for z in 96..112 {
                for y in 1..127 {
                    let here = at(x, y, z);
                    if here == pumpkin_data::Block::NETHER_WART_BLOCK.id {
                        crimson_hat += 1;
                    }
                    if here == pumpkin_data::Block::WARPED_WART_BLOCK.id
                        || here == pumpkin_data::Block::WARPED_STEM.id
                        || here == pumpkin_data::Block::WARPED_NYLIUM.id
                    {
                        warped_blocks += 1;
                    }
                    // A hollow hat has air enclosed on all four horizontal sides by hat
                    // blocks at the same height. A solid painted box never does.
                    if here == pumpkin_data::Block::AIR.id
                        && x > 80
                        && x < 95
                        && z > 96
                        && z < 111
                        && at(x - 1, y, z) == pumpkin_data::Block::NETHER_WART_BLOCK.id
                        && at(x + 1, y, z) == pumpkin_data::Block::NETHER_WART_BLOCK.id
                        && at(x, y, z - 1) == pumpkin_data::Block::NETHER_WART_BLOCK.id
                        && at(x, y, z + 1) == pumpkin_data::Block::NETHER_WART_BLOCK.id
                    {
                        hollow_interior += 1;
                    }
                }
            }
        }

        assert!(
            crimson_hat > 0,
            "reference chunk contains no huge fungus hat blocks to verify"
        );
        assert_eq!(
            warped_blocks, 0,
            "a crimson forest chunk must not contain warped fungus blocks: the configured \
             feature selects the variant, it is not rolled per placement"
        );
        assert!(
            hollow_interior > 0,
            "huge fungus hats must be hollow shells, not solid boxes of hat block"
        );
    }
}
