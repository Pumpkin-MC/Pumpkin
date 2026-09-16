use std::collections::HashSet;

use pumpkin_data::{Block, BlockState, BlockStateId, dimension::Dimension};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator, world_seed::Seed};

use super::processor::{HeightmapType, PosRuleTest, ProcessorRule, RuleTest};
use super::*;
use crate::chunk_system::StagedChunkEnum;
use crate::generation::get_world_gen;
use crate::generation::structure::piece::StructurePieceType;
use crate::generation::structure::structures::jigsaw::{
    JigsawProjection, PoolElementStructurePiece, TemplatePool, place_pool_element_templates,
};
use crate::generation::structure::structures::jigsaw_placement::LiquidSettings;
use crate::generation::structure::structures::{StructurePiece, StructurePieceBase};
use crate::world::WorldPortalExt;
use pumpkin_util::math::block_box::BlockBox;

struct RecordingPlacer {
    chunk: ProtoChunk,
    writes: Vec<(Vector3<i32>, BlockStateId)>,
}

impl BlockPlacer for RecordingPlacer {
    fn get_block_state(&self, pos: &Vector3<i32>) -> BlockStateId {
        self.chunk.get_block_state(pos)
    }

    fn column_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32 {
        self.chunk.column_height(heightmap, x, z)
    }

    fn set_block_state(&mut self, pos: &Vector3<i32>, state: &BlockState) {
        self.writes.push((*pos, state.id));
        self.chunk.set_block_state(pos.x, pos.y, pos.z, state);
    }

    fn add_block_entity(&mut self, nbt: NbtCompound) {
        self.chunk.add_block_entity(nbt);
    }
}

struct BlockRegistry;

impl WorldPortalExt for BlockRegistry {
    fn can_place_at(
        &self,
        _block: &Block,
        _state: &BlockState,
        _accessor: &dyn crate::world::BlockAccessor,
        _pos: &BlockPos,
    ) -> bool {
        true
    }

    fn mirror(&self, block: &Block, state: BlockStateId, mirror: Mirror) -> &'static BlockState {
        block.mirror(state, mirror)
    }

    fn rotate(
        &self,
        block: &Block,
        state: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        block.rotate(state, rotation)
    }

    fn spawn_mobs_for_chunk_generation(
        &self,
        _cache: &mut dyn crate::generation::proto_chunk::GenerationCache,
        _biome: &'static pumpkin_data::chunk::Biome,
        _x: i32,
        _z: i32,
    ) {
    }
}

fn empty_terrain() -> RecordingPlacer {
    let generator = get_world_gen(
        Seed(0),
        Dimension::OVERWORLD,
        true,
        Vec::new(),
        String::new(),
    );
    RecordingPlacer {
        chunk: ProtoChunk::new(0, 0, &generator),
        writes: Vec::new(),
    }
}

fn template(blocks: &[(i32, i32, i32, &str)]) -> StructureTemplate {
    StructureTemplate {
        size: Vector3::new(16, 8, 16),
        palette: blocks
            .iter()
            .map(|(_, _, _, name)| PaletteEntry::from_string(name))
            .collect(),
        blocks: blocks
            .iter()
            .enumerate()
            .map(|(i, &(x, y, z, _))| TemplateBlock {
                pos: Vector3::new(x, y, z),
                state: i as u32,
                nbt: None,
            })
            .collect(),
        ..StructureTemplate::default()
    }
}

fn gravity(heightmap: HeightmapType, offset: i32) -> StructureProcessor {
    StructureProcessor::Gravity { heightmap, offset }
}

fn place(
    placer: &mut RecordingPlacer,
    template: &StructureTemplate,
    processors: &[StructureProcessor],
) {
    place_template(
        placer,
        template,
        Vector3::new(0, 40, 0),
        (0, 0),
        Rotation::None,
        true,
        true,
        processors,
        Some(&BlockBox::new(0, -64, 0, 15, 319, 15)),
    );
}

#[test]
fn gravity_uses_a_stable_surface_and_preserves_local_heights() {
    for reverse in [false, true] {
        let mut terrain = empty_terrain();
        terrain
            .chunk
            .set_block_state(1, 60, 1, Block::STONE.default_state);
        let mut blocks = template(&[(1, 0, 1, "minecraft:stone"), (1, 2, 1, "minecraft:chest")]);
        if reverse {
            blocks.blocks.reverse();
        }
        place(
            &mut terrain,
            &blocks,
            &[gravity(HeightmapType::WorldSurfaceWg, 1)],
        );
        assert_eq!(
            terrain.chunk.get_block_state(&Vector3::new(1, 62, 1)),
            Block::STONE.default_state.id
        );
        assert_eq!(
            terrain.chunk.get_block_state(&Vector3::new(1, 64, 1)),
            Block::CHEST.default_state.id
        );
        let entities = terrain.chunk.take_pending_block_entities();
        assert_eq!(entities.len(), 1);
        assert_eq!(entities[0].get_int("y"), Some(64));
    }
}

#[test]
fn gravity_obeys_heightmap_selection_and_empty_columns() {
    let mut terrain = empty_terrain();
    terrain
        .chunk
        .set_block_state(1, 60, 1, Block::STONE.default_state);
    terrain
        .chunk
        .set_block_state(1, 64, 1, Block::WATER.default_state);
    terrain
        .chunk
        .set_block_state(1, 68, 1, Block::OAK_LEAVES.default_state);
    terrain
        .chunk
        .set_block_state(1, 70, 1, Block::SHORT_GRASS.default_state);
    for (heightmap, expected) in [
        (HeightmapType::WorldSurfaceWg, 71),
        (HeightmapType::WorldSurface, 71),
        (HeightmapType::OceanFloorWg, 69),
        (HeightmapType::OceanFloor, 69),
        (HeightmapType::MotionBlocking, 69),
        (HeightmapType::MotionBlockingNoLeaves, 65),
    ] {
        assert_eq!(terrain.column_height(heightmap, 1, 1), expected);
        assert_eq!(terrain.column_height(heightmap, 2, 2), -64);
    }
    terrain
        .chunk
        .set_block_state(1, 70, 1, Block::AIR.default_state);
    terrain
        .chunk
        .set_block_state(1, 68, 1, Block::AIR.default_state);
    for (heightmap, expected) in [
        (HeightmapType::WorldSurfaceWg, 65),
        (HeightmapType::OceanFloorWg, 61),
        (HeightmapType::MotionBlocking, 65),
        (HeightmapType::MotionBlockingNoLeaves, 65),
    ] {
        assert_eq!(terrain.column_height(heightmap, 1, 1), expected);
    }
}

#[test]
fn village_paths_ignore_blocks_above_finished_terrain() {
    for (obstacle, offset) in [
        (Block::SHORT_GRASS.default_state, 1),
        (Block::TALL_GRASS.default_state, 2),
        (Block::OAK_LEAVES.default_state, 5),
        (Block::OAK_STAIRS.default_state, 8),
    ] {
        let mut terrain = empty_terrain();
        for x in 1..=2 {
            terrain
                .chunk
                .set_block_state(x, 60 + x, 1, Block::STONE.default_state);
        }
        terrain.chunk.stage = StagedChunkEnum::Carvers;
        for x in 1..=2 {
            terrain
                .chunk
                .set_block_state(x, 60 + x + offset, 1, obstacle);
        }
        place(
            &mut terrain,
            &template(&[
                (1, 0, 1, "minecraft:dirt_path"),
                (2, 0, 1, "minecraft:grass_block"),
            ]),
            &[gravity(HeightmapType::WorldSurfaceWg, -1)],
        );
        assert_eq!(
            terrain.writes,
            [
                (Vector3::new(1, 61, 1), Block::DIRT_PATH.default_state.id),
                (Vector3::new(2, 62, 1), Block::GRASS_BLOCK.default_state.id),
            ]
        );
        for x in 1..=2 {
            assert_eq!(
                terrain
                    .chunk
                    .get_block_state(&Vector3::new(x, 60 + x + offset, 1)),
                obstacle.id
            );
            assert_eq!(
                terrain.column_height(HeightmapType::WorldSurface, x, 1),
                61 + x + offset
            );
            assert_eq!(
                terrain.column_height(HeightmapType::WorldSurfaceWg, x, 1),
                61 + x
            );
        }
    }
}

#[test]
fn terrain_heightmaps_freeze_after_carving_and_survive_cleared_columns() {
    let mut terrain = empty_terrain();
    terrain.chunk.stage = StagedChunkEnum::Surface;
    terrain
        .chunk
        .set_block_state(1, 60, 1, Block::STONE.default_state);
    terrain
        .chunk
        .set_block_state(1, 64, 1, Block::WATER.default_state);
    terrain
        .chunk
        .set_block_state(1, 70, 1, Block::STONE.default_state);
    terrain
        .chunk
        .set_block_state(1, 70, 1, Block::AIR.default_state);
    terrain.chunk.stage = StagedChunkEnum::Carvers;
    terrain
        .chunk
        .set_block_state(1, 64, 1, Block::AIR.default_state);
    terrain
        .chunk
        .set_block_state(1, 60, 1, Block::AIR.default_state);
    terrain
        .chunk
        .set_block_state(2, 80, 2, Block::OAK_LEAVES.default_state);
    assert_eq!(
        terrain.column_height(HeightmapType::WorldSurfaceWg, 1, 1),
        65
    );
    assert_eq!(terrain.column_height(HeightmapType::OceanFloorWg, 1, 1), 61);
    assert_eq!(
        terrain.column_height(HeightmapType::WorldSurface, 1, 1),
        -64
    );
    assert_eq!(terrain.column_height(HeightmapType::OceanFloor, 1, 1), -64);
    assert_eq!(
        terrain.column_height(HeightmapType::WorldSurfaceWg, 2, 2),
        -64
    );
    place(
        &mut terrain,
        &template(&[(1, 0, 1, "minecraft:dirt_path")]),
        &[gravity(HeightmapType::WorldSurfaceWg, -1)],
    );
    assert_eq!(
        terrain.writes,
        [(Vector3::new(1, 64, 1), Block::DIRT_PATH.default_state.id)]
    );
}

#[test]
fn processors_observe_positions_in_order_and_waterlogging_uses_the_final_state() {
    let rule = StructureProcessor::Rule(vec![ProcessorRule {
        position_predicate: PosRuleTest::AlwaysTrue,
        input_predicate: RuleTest::AlwaysTrue,
        location_predicate: RuleTest::BlockMatch(Block::LAVA.id),
        output_state: Block::OAK_STAIRS.default_state,
        block_entity_modifier: None,
    }]);
    for project_first in [false, true] {
        let mut terrain = empty_terrain();
        terrain
            .chunk
            .set_block_state(1, 40, 1, Block::LAVA.default_state);
        terrain
            .chunk
            .set_block_state(1, 60, 1, Block::WATER.default_state);
        let mut processors = vec![rule.clone(), gravity(HeightmapType::WorldSurfaceWg, -1)];
        if project_first {
            processors.reverse();
        }
        place(
            &mut terrain,
            &template(&[(1, 0, 1, "minecraft:stone")]),
            &processors,
        );
        let expected = if project_first {
            Block::STONE.default_state
        } else {
            Block::OAK_STAIRS
                .default_state
                .set_waterlogged(true)
                .unwrap()
        };
        assert_eq!(terrain.writes, [(Vector3::new(1, 60, 1), expected.id)]);
    }
}

#[test]
fn projected_blocks_are_clipped_after_processing() {
    let mut terrain = empty_terrain();
    terrain
        .chunk
        .set_block_state(1, 319, 1, Block::STONE.default_state);
    place(
        &mut terrain,
        &template(&[(1, 1, 1, "minecraft:chest")]),
        &[gravity(HeightmapType::WorldSurfaceWg, -1)],
    );
    assert!(terrain.writes.is_empty());
    assert!(terrain.chunk.take_pending_block_entities().is_empty());
}

#[test]
fn capped_gravity_counts_position_changes() {
    let mut terrain = empty_terrain();
    for x in [1, 2] {
        terrain
            .chunk
            .set_block_state(x, 60, 1, Block::STONE.default_state);
    }
    place(
        &mut terrain,
        &template(&[(1, 0, 1, "minecraft:stone"), (2, 0, 1, "minecraft:stone")]),
        &[StructureProcessor::Capped {
            limit: pumpkin_util::math::int_provider::IntProvider::Constant(1),
            delegate: Box::new(gravity(HeightmapType::WorldSurfaceWg, -1)),
        }],
    );
    assert_eq!(
        terrain
            .writes
            .iter()
            .map(|(pos, _)| *pos)
            .collect::<Vec<_>>(),
        [Vector3::new(1, 60, 1), Vector3::new(2, 40, 1)]
    );
}

#[test]
fn projected_jigsaws_respect_keep_jigsaws_and_legacy_air() {
    for keep_jigsaws in [false, true] {
        let mut terrain = empty_terrain();
        terrain
            .chunk
            .set_block_state(1, 60, 1, Block::STONE.default_state);
        let blocks = template(&[(1, 0, 1, "minecraft:jigsaw"), (1, 1, 1, "minecraft:jigsaw")]);
        place_template_with_options(
            &mut terrain,
            &blocks,
            Vector3::new(0, 40, 0),
            (0, 0),
            Rotation::None,
            true,
            false,
            &[gravity(HeightmapType::WorldSurfaceWg, -1)],
            Some(&BlockBox::new(0, -64, 0, 15, 319, 15)),
            keep_jigsaws,
        );
        if keep_jigsaws {
            assert_eq!(
                terrain.writes,
                [
                    (Vector3::new(1, 60, 1), Block::JIGSAW.default_state.id),
                    (Vector3::new(1, 61, 1), Block::JIGSAW.default_state.id)
                ]
            );
        } else {
            assert!(terrain.writes.is_empty());
        }
    }
}

#[test]
#[allow(clippy::too_many_lines, clippy::print_stdout)]
fn village_path_projection_1024_cases() {
    let generator = get_world_gen(
        Seed(0),
        Dimension::OVERWORLD,
        true,
        Vec::new(),
        String::new(),
    );
    let pools: Vec<_> = ["plains", "desert", "savanna", "snowy", "taiga"]
        .iter()
        .map(|biome| TemplatePool::discover(&format!("minecraft:village/{biome}/streets")).unwrap())
        .collect();
    let rotations = [
        Rotation::None,
        Rotation::Clockwise90,
        Rotation::Rotate180,
        Rotation::CounterClockwise90,
    ];
    let mut checked = 0;
    let mut columns = 0;
    let mut misplaced_before = 0;
    let mut flat_cases = 0;

    for seed in 0..64u64 {
        for profile in 0..4 {
            for (rotation_index, rotation) in rotations.iter().copied().enumerate() {
                let case = seed as usize * 16 + profile * 4 + rotation_index;
                let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(seed));
                let pool = &pools[case % pools.len()];
                let element = pool.get_random_element(&mut random).clone();
                assert_eq!(element.projection, JigsawProjection::TerrainMatching);
                let template = element.first_template().unwrap();
                let chunk_x = -23 + (seed as i32 % 3) - 1;
                let chunk_z = 38 + (seed as i32 % 5) - 2;
                let base = 72 + (seed % 11) as i32;
                let build_terrain = || {
                    let mut chunk = ProtoChunk::new(chunk_x, chunk_z, &generator);
                    for x in 0..16 {
                        for z in 0..16 {
                            let height = match profile {
                                0 => base,
                                1 => base + (x + z) / 4,
                                2 => base + x * 2 - z,
                                _ => base + (x - z) / 2,
                            };
                            for y in height - 3..=height {
                                chunk.set_block_state(x, y, z, Block::STONE.default_state);
                            }
                            if profile == 3 {
                                for y in height + 1..=base + 2 {
                                    chunk.set_block_state(x, y, z, Block::WATER.default_state);
                                }
                            }
                        }
                    }
                    chunk.stage = StagedChunkEnum::Carvers;
                    chunk
                };
                let mut natural = build_terrain();
                let anchor = template
                    .blocks
                    .iter()
                    .find(|block| {
                        !matches!(
                            template.palette[block.state as usize].name.as_str(),
                            "minecraft:air"
                                | "minecraft:structure_void"
                                | "minecraft:jigsaw"
                                | "minecraft:structure_block"
                        )
                    })
                    .unwrap()
                    .pos;
                let (anchor_x, anchor_z) = rotation.rotate_offset(anchor.x, anchor.z);
                let origin = BlockPos::new(
                    chunk_x * 16 + 7 - anchor_x,
                    base,
                    chunk_z * 16 + 7 - anchor_z,
                );
                let make_piece = |projection| PoolElementStructurePiece {
                    piece: StructurePiece::new(
                        StructurePieceType::Jigsaw,
                        element.get_bounding_box(origin, rotation),
                        0,
                    ),
                    element: element.clone(),
                    pos: origin,
                    rotation,
                    mirror: Mirror::None,
                    jigsaw_blocks: Vec::new(),
                    junctions: Vec::new(),
                    ground_level_delta: 1,
                    liquid_settings: LiquidSettings::ApplyWaterlog,
                    projection,
                };
                let mut piece = make_piece(JigsawProjection::TerrainMatching);
                let bounds = BlockBox::new(
                    chunk_x * 16,
                    -64,
                    chunk_z * 16,
                    chunk_x * 16 + 15,
                    319,
                    chunk_z * 16 + 15,
                );
                let mut projected = RecordingPlacer {
                    chunk: build_terrain(),
                    writes: Vec::new(),
                };
                let mut original = RecordingPlacer {
                    chunk: build_terrain(),
                    writes: Vec::new(),
                };
                let mut expected = HashSet::new();
                for block in &template.blocks {
                    let entry = &template.palette[block.state as usize];
                    let replacement;
                    let name = if entry.name == "minecraft:jigsaw" {
                        replacement = PaletteEntry::from_string(
                            block
                                .nbt
                                .as_ref()
                                .and_then(|nbt| nbt.get_string("final_state"))
                                .unwrap_or("minecraft:air"),
                        );
                        replacement.name.as_str()
                    } else {
                        entry.name.as_str()
                    };
                    if matches!(
                        name,
                        "minecraft:air"
                            | "minecraft:structure_void"
                            | "minecraft:structure_block"
                            | "minecraft:jigsaw"
                    ) {
                        continue;
                    }
                    let (x, z) = match rotation {
                        Rotation::None => (block.pos.x, block.pos.z),
                        Rotation::Clockwise90 => (-block.pos.z, block.pos.x),
                        Rotation::Rotate180 => (-block.pos.x, -block.pos.z),
                        Rotation::CounterClockwise90 => (block.pos.z, -block.pos.x),
                    };
                    let x = origin.0.x + x;
                    let z = origin.0.z + z;
                    if !bounds.contains(x, base + block.pos.y, z) {
                        continue;
                    }
                    let surface = (-64..320)
                        .rev()
                        .find(|&y| !natural.is_air(&Vector3::new(x, y, z)))
                        .unwrap();
                    expected.insert((x, surface + block.pos.y, z));
                }
                place_pool_element_templates(&piece, &mut projected, Some(&bounds), false);
                let actual: HashSet<_> = projected
                    .writes
                    .iter()
                    .map(|(p, _)| (p.x, p.y, p.z))
                    .collect();
                assert!(!expected.is_empty(), "case {case}: empty path sample");
                assert_eq!(actual, expected, "case {case}, pool {}", pool.id);
                let mut covered = RecordingPlacer {
                    chunk: build_terrain(),
                    writes: Vec::new(),
                };
                let cover_y = expected.iter().map(|&(_, y, _)| y).max().unwrap() + 8;
                let cover = if case.is_multiple_of(2) {
                    Block::OAK_LEAVES
                } else {
                    Block::OAK_STAIRS
                };
                for x in bounds.min.x..=bounds.max.x {
                    for z in bounds.min.z..=bounds.max.z {
                        covered
                            .chunk
                            .set_block_state(x, cover_y, z, cover.default_state);
                    }
                }
                place_pool_element_templates(&piece, &mut covered, Some(&bounds), false);
                assert_eq!(
                    covered
                        .writes
                        .iter()
                        .map(|(p, _)| (p.x, p.y, p.z))
                        .collect::<HashSet<_>>(),
                    expected,
                    "cover changed path heights in case {case}"
                );
                for x in bounds.min.x..=bounds.max.x {
                    for z in bounds.min.z..=bounds.max.z {
                        assert_eq!(
                            covered.chunk.get_block_state(&Vector3::new(x, cover_y, z)),
                            cover.default_state.id
                        );
                    }
                }
                piece.place(
                    &mut natural,
                    &BlockRegistry,
                    &mut random,
                    seed as i64,
                    &bounds,
                );
                assert_eq!(
                    natural.flat_block_map, projected.chunk.flat_block_map,
                    "worldgen/helper disagree in case {case}"
                );
                place_pool_element_templates(
                    &make_piece(JigsawProjection::Rigid),
                    &mut original,
                    Some(&bounds),
                    false,
                );
                misplaced_before += original
                    .writes
                    .iter()
                    .filter(|(p, _)| !expected.contains(&(p.x, p.y, p.z)))
                    .count();
                if profile == 0 {
                    assert_eq!(
                        original.chunk.flat_block_map, projected.chunk.flat_block_map,
                        "flat terrain changed in case {case}"
                    );
                    flat_cases += 1;
                }
                columns += actual.len();
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 1024);
    assert_eq!(flat_cases, 256);
    assert!(misplaced_before > 0);
    println!(
        "village path projection: {checked} cases, {columns} block positions, {misplaced_before} misplaced before, 0 after, {flat_cases} flat cases unchanged"
    );
}
