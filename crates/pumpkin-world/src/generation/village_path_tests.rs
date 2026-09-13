use super::*;
use crate::chunk_system::{Chunk, generation_cache::SurfaceBiomeNeighborhood};
use crate::generation::structure::structures::jigsaw::{
    JigsawProjection, PoolElementStructurePiece,
};
use crate::generation::{generator::WorldGenerator, get_world_gen};
use pumpkin_data::dimension::Dimension;
use pumpkin_util::world_seed::Seed;

struct BlockRegistry;

impl WorldPortalExt for BlockRegistry {
    fn can_place_at(
        &self,
        _block: &Block,
        _state: &BlockState,
        _accessor: &dyn BlockAccessor,
        _pos: &BlockPos,
    ) -> bool {
        true
    }
    fn mirror(
        &self,
        block: &Block,
        state: BlockStateId,
        mirror: pumpkin_data::Mirror,
    ) -> &'static BlockState {
        block.mirror(state, mirror)
    }
    fn rotate(
        &self,
        block: &Block,
        state: BlockStateId,
        rotation: pumpkin_data::Rotation,
    ) -> &'static BlockState {
        block.rotate(state, rotation)
    }
    fn spawn_mobs_for_chunk_generation(
        &self,
        _cache: &mut dyn GenerationCache,
        _biome: &'static Biome,
        _x: i32,
        _z: i32,
    ) {
    }
}

fn surface_chunk(world: &WorldGenerator, x: i32, z: i32) -> ProtoChunk {
    let WorldGenerator::Noise(generator) = world else {
        unreachable!()
    };
    let mut biomes = SurfaceBiomeNeighborhood::new(x, z);
    for dx in -1..=1 {
        for dz in -1..=1 {
            let mut neighbor = ProtoChunk::new(x + dx, z + dz, world);
            neighbor.step_to_biomes(generator);
            assert!(biomes.push_chunk(&Chunk::Proto(Box::new(neighbor))));
        }
    }
    let mut chunk = ProtoChunk::new(x, z, world);
    chunk.step_to_biomes(generator);
    chunk.set_structure_starts(generator);
    chunk.set_structure_references(generator);
    chunk.step_to_noise(generator);
    chunk.step_to_surface(generator, &biomes);
    chunk.step_to_carvers(generator);
    chunk
}

#[test]
#[allow(clippy::print_stdout)]
fn village_paths_follow_generated_surface() {
    for (seed, block_x, block_z) in [
        (1_788_686_529_849_575_733, -355, 617),
        (0, -1063, 1221),
        (7, -560, -706),
        (13_579, -1048, 297),
    ] {
        let (center_x, center_z) = (block_x >> 4, block_z >> 4);
        let world = get_world_gen(
            Seed(seed),
            Dimension::OVERWORLD,
            false,
            Vec::new(),
            String::new(),
        );
        let mut verified = 0;
        let mut displaced = 0;
        for (dx, dz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let (x, z) = (center_x + dx, center_z + dz);
            let mut chunk = surface_chunk(&world, x, z);
            let bounds = BlockBox::new(x * 16, -64, z * 16, x * 16 + 15, 319, z * 16 + 15);
            let collectors: Vec<_> = chunk
                .structure_starts
                .values()
                .map(|instance| match instance {
                    StructureInstance::Start(start) => Arc::clone(&start.collector),
                    StructureInstance::Reference(collector) => Arc::clone(collector),
                })
                .collect();
            let mut random = create_chunk_random(seed as i64, x, z);
            for collector in collectors {
                let mut collector = collector.lock().unwrap();
                for piece in &mut collector.pieces {
                    if !piece.bounding_box().intersects(&bounds) {
                        continue;
                    }
                    let expected = if let Some(path) =
                        piece.as_any().downcast_ref::<PoolElementStructurePiece>()
                        && path.projection == JigsawProjection::TerrainMatching
                    {
                        expected_path_blocks(&chunk, path, &bounds)
                    } else {
                        Vec::new()
                    };
                    piece.place(
                        &mut chunk,
                        &BlockRegistry,
                        &mut random,
                        seed as i64,
                        &bounds,
                    );
                    for (pos, expected_block, original_y) in expected {
                        displaced += usize::from(pos.y != original_y);
                        let block = chunk.get_block_state(&pos).to_block_id();
                        assert!(
                            block == expected_block
                                || (expected_block == BlockId::DIRT_PATH
                                    && matches!(
                                        block,
                                        BlockId::GRASS_BLOCK
                                            | BlockId::OAK_PLANKS
                                            | BlockId::SPRUCE_PLANKS
                                            | BlockId::ACACIA_PLANKS
                                    )),
                            "seed {seed}, path at {pos:?} is {block:?}"
                        );
                        verified += 1;
                    }
                }
            }
        }
        assert!(verified > 0, "seed {seed}: no paths checked");
        if seed == 1_788_686_529_849_575_733 {
            assert!(displaced > 0);
        }
        println!(
            "village seed={seed}, chunk=({center_x}, {center_z}): {verified} path blocks verified, {displaced} require projection"
        );
    }
}

fn expected_path_blocks(
    chunk: &ProtoChunk,
    path: &PoolElementStructurePiece,
    bounds: &BlockBox,
) -> Vec<(Vector3<i32>, BlockId, i32)> {
    use crate::generation::structure::template::processor::HeightmapType;

    let mut expected = Vec::new();
    path.element.for_each_template(|_, _, _, template| {
        for block in &template.blocks {
            let expected_block = match template.palette[block.state as usize].name.as_str() {
                "minecraft:dirt_path" => BlockId::DIRT_PATH,
                "minecraft:smooth_sandstone" => BlockId::SMOOTH_SANDSTONE,
                "minecraft:terracotta" => BlockId::TERRACOTTA,
                _ => continue,
            };
            let (dx, dz) = path.rotation.rotate_offset(block.pos.x, block.pos.z);
            let pos = Vector3::new(
                path.pos.0.x + dx,
                path.pos.0.y + block.pos.y,
                path.pos.0.z + dz,
            );
            if !bounds.contains_pos(&pos) {
                continue;
            }
            let top = (-64..320)
                .rev()
                .find(|&y| !chunk.is_air(&Vector3::new(pos.x, y, pos.z)))
                .unwrap();
            assert_eq!(
                chunk.column_height(HeightmapType::WorldSurfaceWg, pos.x, pos.z),
                top + 1,
                "stale surface at {pos:?}"
            );
            expected.push((
                Vector3::new(pos.x, top + block.pos.y, pos.z),
                expected_block,
                pos.y,
            ));
        }
    });
    expected
}
