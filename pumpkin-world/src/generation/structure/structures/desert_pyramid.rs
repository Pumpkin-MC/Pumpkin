use std::sync::Arc;

use pumpkin_data::{
    Block, BlockState,
    block_properties::{BlockProperties, HorizontalFacing, OakStairsLikeProperties},
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    BlockDirection, HeightMap,
    math::{block_box::BlockBox, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos::{start_block_x, start_block_z},
        structure::{
            piece::StructurePieceType,
            structures::{
                StructureGenerator, StructureGeneratorContext, StructurePiece, StructurePieceBase,
                StructurePiecesCollector, StructurePosition,
            },
        },
    },
};

const WIDTH: i32 = 21;
const HEIGHT: i32 = 15;
const DEPTH: i32 = 21;

pub struct DesertPyramidGenerator;

impl StructureGenerator for DesertPyramidGenerator {
    fn get_structure_position(
        &self,
        mut context: StructureGeneratorContext,
    ) -> Option<StructurePosition> {
        let x = start_block_x(context.chunk_x);
        let z = start_block_z(context.chunk_z);

        let facing = BlockDirection::get_random_horizontal_direction(&mut context.random);

        let mut piece = StructurePiece::new(
            StructurePieceType::DesertTemple,
            BlockBox::create_box(x, 64, z, facing.get_axis(), WIDTH, HEIGHT, DEPTH),
            0,
        );
        piece.set_facing(Some(facing));

        let mut collector = StructurePiecesCollector::default();
        collector.add_piece(Box::new(DesertPyramidPiece {
            piece,
            height_adjusted: false,
            has_placed_chest: [false; 4],
        }));

        Some(StructurePosition {
            start_pos: BlockPos::new(x + (WIDTH / 2), 64, z + (DEPTH / 2)),
            collector: Arc::new(collector.into()),
        })
    }
}

#[derive(Clone)]
pub struct DesertPyramidPiece {
    piece: StructurePiece,
    height_adjusted: bool,
    has_placed_chest: [bool; 4],
}

impl DesertPyramidPiece {
    fn sandstone_stairs(facing: HorizontalFacing) -> &'static BlockState {
        let mut props = OakStairsLikeProperties::default(&Block::SANDSTONE_STAIRS);
        props.facing = facing;
        BlockState::from_id(props.to_state_id(&Block::SANDSTONE_STAIRS))
    }

    fn adjust_height(&mut self, chunk: &ProtoChunk, random: &mut RandomGenerator) -> bool {
        if self.height_adjusted {
            return true;
        }

        let ground_offset = -(random.next_bounded_i32(3));
        let bb = self.piece.bounding_box;
        let mut lowest = i32::MAX;

        for z in bb.min.z..=bb.max.z {
            for x in bb.min.x..=bb.max.x {
                let y = chunk.get_top_y(&HeightMap::MotionBlockingNoLeaves, x, z);
                lowest = lowest.min(y);
            }
        }

        if lowest == i32::MAX {
            return false;
        }

        let shift_y = lowest - self.piece.bounding_box.min.y + ground_offset;
        self.piece.bounding_box.move_pos(0, shift_y, 0);
        self.height_adjusted = true;
        true
    }

    #[expect(clippy::too_many_arguments)]
    fn gen_box(
        &self,
        chunk: &mut ProtoChunk,
        bb: &BlockBox,
        x0: i32,
        y0: i32,
        z0: i32,
        x1: i32,
        y1: i32,
        z1: i32,
        edge: &BlockState,
        fill: &BlockState,
        skip_air: bool,
    ) {
        self.piece
            .fill_with_outline(chunk, bb, skip_air, x0, y0, z0, x1, y1, z1, edge, fill);
    }

    fn pb(
        &self,
        chunk: &mut ProtoChunk,
        bb: &BlockBox,
        state: &BlockState,
        x: i32,
        y: i32,
        z: i32,
    ) {
        self.piece.add_block(chunk, state, x, y, z, bb);
    }

    fn fill_down(
        &self,
        chunk: &mut ProtoChunk,
        bb: &BlockBox,
        state: &BlockState,
        x: i32,
        y: i32,
        z: i32,
    ) {
        self.piece.fill_downwards(chunk, state, x, y, z, bb);
    }

    fn place_sand(&self, chunk: &mut ProtoChunk, bb: &BlockBox, x: i32, y: i32, z: i32) {
        self.pb(chunk, bb, Block::SAND.default_state, x, y, z);
    }

    #[expect(clippy::too_many_arguments)]
    fn place_sand_box(
        &self,
        chunk: &mut ProtoChunk,
        bb: &BlockBox,
        x0: i32,
        y0: i32,
        z0: i32,
        x1: i32,
        y1: i32,
        z1: i32,
    ) {
        for y in y0..=y1 {
            for x in x0..=x1 {
                for z in z0..=z1 {
                    self.place_sand(chunk, bb, x, y, z);
                }
            }
        }
    }

    #[expect(clippy::too_many_arguments)]
    fn place_collapsed_roof(
        &self,
        chunk: &mut ProtoChunk,
        bb: &BlockBox,
        random: &mut RandomGenerator,
        x0: i32,
        y0: i32,
        z0: i32,
        x1: i32,
        z1: i32,
    ) {
        for x in x0..=x1 {
            for z in z0..=z1 {
                let state = if random.next_f32() < 0.33 {
                    Block::SANDSTONE.default_state
                } else {
                    Block::SAND.default_state
                };
                self.pb(chunk, bb, state, x, y0, z);
            }
        }
        // TODO: Pick a random collapsed roof position for sus sand placement in
        // the structure-level afterPlace pass when brushable support exists.
    }

    fn try_place_chest(
        &mut self,
        chunk: &mut ProtoChunk,
        bb: &BlockBox,
        index: usize,
        x: i32,
        y: i32,
        z: i32,
    ) {
        if self.has_placed_chest[index] {
            return;
        }

        let world_pos = self.piece.offset_pos(x, y, z);
        if !bb.contains_pos(&world_pos) {
            return;
        }

        self.pb(chunk, bb, Block::CHEST.default_state, x, y, z);

        let mut nbt = NbtCompound::new();
        nbt.put_int("x", world_pos.x);
        nbt.put_int("y", world_pos.y);
        nbt.put_int("z", world_pos.z);
        nbt.put_string("id", "minecraft:chest".to_string());
        // TODO: attach desert_pyramid loot table once structure container loot exists.
        chunk.add_pending_block_entity(nbt);
        self.has_placed_chest[index] = true;
    }

    fn add_cellar(&self, chunk: &mut ProtoChunk, bb: &BlockBox, random: &mut RandomGenerator) {
        self.add_cellar_stairs(chunk, bb, random);
        self.add_cellar_room(chunk, bb, random);
    }

    fn add_cellar_stairs(
        &self,
        chunk: &mut ProtoChunk,
        bb: &BlockBox,
        random: &mut RandomGenerator,
    ) {
        let west_stairs = Self::sandstone_stairs(HorizontalFacing::West);
        self.pb(chunk, bb, west_stairs, 13, -1, 17);
        self.pb(chunk, bb, west_stairs, 14, -2, 17);
        self.pb(chunk, bb, west_stairs, 15, -3, 17);

        let sand = Block::SAND.default_state;
        let sandstone = Block::SANDSTONE.default_state;
        let (x, y, z) = (16, -4, 13);
        let variant = random.next_bool();

        self.pb(chunk, bb, sand, x - 4, y + 4, z + 4);
        self.pb(chunk, bb, sand, x - 3, y + 4, z + 4);
        self.pb(chunk, bb, sand, x - 2, y + 4, z + 4);
        self.pb(chunk, bb, sand, x - 1, y + 4, z + 4);
        self.pb(chunk, bb, sand, x, y + 4, z + 4);
        self.pb(chunk, bb, sand, x - 2, y + 3, z + 4);
        self.pb(
            chunk,
            bb,
            if variant { sand } else { sandstone },
            x - 1,
            y + 3,
            z + 4,
        );
        self.pb(
            chunk,
            bb,
            if variant { sandstone } else { sand },
            x,
            y + 3,
            z + 4,
        );
        self.pb(chunk, bb, sand, x - 1, y + 2, z + 4);
        self.pb(chunk, bb, sandstone, x, y + 2, z + 4);
        self.pb(chunk, bb, sand, x, y + 1, z + 4);
    }

    fn add_cellar_room(&self, chunk: &mut ProtoChunk, bb: &BlockBox, random: &mut RandomGenerator) {
        let (x, y, z) = (16, -4, 13);
        let cut = Block::CUT_SANDSTONE.default_state;
        let chiseled = Block::CHISELED_SANDSTONE.default_state;
        let orange = Block::ORANGE_TERRACOTTA.default_state;
        let blue = Block::BLUE_TERRACOTTA.default_state;
        self.gen_box(
            chunk,
            bb,
            x - 3,
            y + 1,
            z - 3,
            x - 3,
            y + 1,
            z + 2,
            cut,
            cut,
            true,
        );
        self.gen_box(
            chunk,
            bb,
            x + 3,
            y + 1,
            z - 3,
            x + 3,
            y + 1,
            z + 2,
            cut,
            cut,
            true,
        );
        self.gen_box(
            chunk,
            bb,
            x - 3,
            y + 1,
            z - 3,
            x + 3,
            y + 1,
            z - 2,
            cut,
            cut,
            true,
        );
        self.gen_box(
            chunk,
            bb,
            x - 3,
            y + 1,
            z + 3,
            x + 3,
            y + 1,
            z + 3,
            cut,
            cut,
            true,
        );
        self.gen_box(
            chunk,
            bb,
            x - 3,
            y + 2,
            z - 3,
            x - 3,
            y + 2,
            z + 2,
            chiseled,
            chiseled,
            true,
        );
        self.gen_box(
            chunk,
            bb,
            x + 3,
            y + 2,
            z - 3,
            x + 3,
            y + 2,
            z + 2,
            chiseled,
            chiseled,
            true,
        );
        self.gen_box(
            chunk,
            bb,
            x - 3,
            y + 2,
            z - 3,
            x + 3,
            y + 2,
            z - 2,
            chiseled,
            chiseled,
            true,
        );
        self.gen_box(
            chunk,
            bb,
            x - 3,
            y + 2,
            z + 3,
            x + 3,
            y + 2,
            z + 3,
            chiseled,
            chiseled,
            true,
        );
        self.gen_box(
            chunk,
            bb,
            x - 3,
            -1,
            z - 3,
            x - 3,
            -1,
            z + 2,
            cut,
            cut,
            true,
        );
        self.gen_box(
            chunk,
            bb,
            x + 3,
            -1,
            z - 3,
            x + 3,
            -1,
            z + 2,
            cut,
            cut,
            true,
        );
        self.gen_box(
            chunk,
            bb,
            x - 3,
            -1,
            z - 3,
            x + 3,
            -1,
            z - 2,
            cut,
            cut,
            true,
        );
        self.gen_box(
            chunk,
            bb,
            x - 3,
            -1,
            z + 3,
            x + 3,
            -1,
            z + 3,
            cut,
            cut,
            true,
        );

        self.place_sand_box(chunk, bb, x - 2, y + 1, z - 2, x + 2, y + 3, z + 2);
        self.place_collapsed_roof(chunk, bb, random, x - 2, y + 4, z - 2, x + 2, z + 2);
        self.pb(chunk, bb, blue, x, y, z);
        self.pb(chunk, bb, orange, x + 1, y, z - 1);
        self.pb(chunk, bb, orange, x + 1, y, z + 1);
        self.pb(chunk, bb, orange, x - 1, y, z - 1);
        self.pb(chunk, bb, orange, x - 1, y, z + 1);
        self.pb(chunk, bb, orange, x + 2, y, z);
        self.pb(chunk, bb, orange, x - 2, y, z);
        self.pb(chunk, bb, orange, x, y, z + 2);
        self.pb(chunk, bb, orange, x, y, z - 2);

        self.pb(chunk, bb, orange, x + 3, y, z);
        self.place_sand(chunk, bb, x + 3, y + 1, z);
        self.place_sand(chunk, bb, x + 3, y + 2, z);
        self.pb(chunk, bb, cut, x + 4, y + 1, z);
        self.pb(chunk, bb, chiseled, x + 4, y + 2, z);

        self.pb(chunk, bb, orange, x - 3, y, z);
        self.place_sand(chunk, bb, x - 3, y + 1, z);
        self.place_sand(chunk, bb, x - 3, y + 2, z);
        self.pb(chunk, bb, cut, x - 4, y + 1, z);
        self.pb(chunk, bb, chiseled, x - 4, y + 2, z);

        self.pb(chunk, bb, orange, x, y, z + 3);
        self.place_sand(chunk, bb, x, y + 1, z + 3);
        self.place_sand(chunk, bb, x, y + 2, z + 3);

        self.pb(chunk, bb, orange, x, y, z - 3);
        self.place_sand(chunk, bb, x, y + 1, z - 3);
        self.place_sand(chunk, bb, x, y + 2, z - 3);
        self.pb(chunk, bb, cut, x, y + 1, z - 4);
        self.pb(chunk, bb, chiseled, x, -2, z - 4);
    }
}

impl StructurePieceBase for DesertPyramidPiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }

    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece
    }

    #[expect(clippy::too_many_lines)]
    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        if !self.adjust_height(chunk, random) {
            return;
        }

        let bb = chunk_box;
        let ss = Block::SANDSTONE.default_state;
        let air = Block::AIR.default_state;
        let cut = Block::CUT_SANDSTONE.default_state;
        let chiseled = Block::CHISELED_SANDSTONE.default_state;
        let orange = Block::ORANGE_TERRACOTTA.default_state;
        let blue = Block::BLUE_TERRACOTTA.default_state;
        let slab = Block::SANDSTONE_SLAB.default_state;

        let north_stairs = Self::sandstone_stairs(HorizontalFacing::North);
        let south_stairs = Self::sandstone_stairs(HorizontalFacing::South);
        let east_stairs = Self::sandstone_stairs(HorizontalFacing::East);
        let west_stairs = Self::sandstone_stairs(HorizontalFacing::West);

        self.gen_box(chunk, bb, 0, -4, 0, WIDTH - 1, 0, DEPTH - 1, ss, ss, false);

        for pos in 1..=9 {
            self.gen_box(
                chunk,
                bb,
                pos,
                pos,
                pos,
                WIDTH - 1 - pos,
                pos,
                DEPTH - 1 - pos,
                ss,
                ss,
                false,
            );
            self.gen_box(
                chunk,
                bb,
                pos + 1,
                pos,
                pos + 1,
                WIDTH - 2 - pos,
                pos,
                DEPTH - 2 - pos,
                air,
                air,
                false,
            );
        }

        for x in 0..WIDTH {
            for z in 0..DEPTH {
                self.fill_down(chunk, bb, ss, x, -5, z);
            }
        }

        self.gen_box(chunk, bb, 0, 0, 0, 4, 9, 4, ss, air, false);
        self.gen_box(chunk, bb, 1, 10, 1, 3, 10, 3, ss, ss, false);
        self.pb(chunk, bb, north_stairs, 2, 10, 0);
        self.pb(chunk, bb, south_stairs, 2, 10, 4);
        self.pb(chunk, bb, east_stairs, 0, 10, 2);
        self.pb(chunk, bb, west_stairs, 4, 10, 2);

        self.gen_box(chunk, bb, WIDTH - 5, 0, 0, WIDTH - 1, 9, 4, ss, air, false);
        self.gen_box(chunk, bb, WIDTH - 4, 10, 1, WIDTH - 2, 10, 3, ss, ss, false);
        self.pb(chunk, bb, north_stairs, WIDTH - 3, 10, 0);
        self.pb(chunk, bb, south_stairs, WIDTH - 3, 10, 4);
        self.pb(chunk, bb, east_stairs, WIDTH - 5, 10, 2);
        self.pb(chunk, bb, west_stairs, WIDTH - 1, 10, 2);

        self.gen_box(chunk, bb, 8, 0, 0, 12, 4, 4, ss, air, false);
        self.gen_box(chunk, bb, 9, 1, 0, 11, 3, 4, air, air, false);
        self.pb(chunk, bb, cut, 9, 1, 1);
        self.pb(chunk, bb, cut, 9, 2, 1);
        self.pb(chunk, bb, cut, 9, 3, 1);
        self.pb(chunk, bb, cut, 10, 3, 1);
        self.pb(chunk, bb, cut, 11, 3, 1);
        self.pb(chunk, bb, cut, 11, 2, 1);
        self.pb(chunk, bb, cut, 11, 1, 1);

        self.gen_box(chunk, bb, 4, 1, 1, 8, 3, 3, ss, air, false);
        self.gen_box(chunk, bb, 4, 1, 2, 8, 2, 2, air, air, false);
        self.gen_box(chunk, bb, 12, 1, 1, 16, 3, 3, ss, air, false);
        self.gen_box(chunk, bb, 12, 1, 2, 16, 2, 2, air, air, false);

        self.gen_box(chunk, bb, 5, 4, 5, WIDTH - 6, 4, DEPTH - 6, ss, ss, false);
        self.gen_box(chunk, bb, 9, 4, 9, 11, 4, 11, air, air, false);

        self.gen_box(chunk, bb, 8, 1, 8, 8, 3, 8, cut, cut, false);
        self.gen_box(chunk, bb, 12, 1, 8, 12, 3, 8, cut, cut, false);
        self.gen_box(chunk, bb, 8, 1, 12, 8, 3, 12, cut, cut, false);
        self.gen_box(chunk, bb, 12, 1, 12, 12, 3, 12, cut, cut, false);

        self.gen_box(chunk, bb, 1, 1, 5, 4, 4, 11, ss, ss, false);
        self.gen_box(chunk, bb, WIDTH - 5, 1, 5, WIDTH - 2, 4, 11, ss, ss, false);

        self.gen_box(chunk, bb, 6, 7, 9, 6, 7, 11, ss, ss, false);
        self.gen_box(chunk, bb, WIDTH - 7, 7, 9, WIDTH - 7, 7, 11, ss, ss, false);
        self.gen_box(chunk, bb, 5, 5, 9, 5, 7, 11, cut, cut, false);
        self.gen_box(
            chunk,
            bb,
            WIDTH - 6,
            5,
            9,
            WIDTH - 6,
            7,
            11,
            cut,
            cut,
            false,
        );

        self.pb(chunk, bb, air, 5, 5, 10);
        self.pb(chunk, bb, air, 5, 6, 10);
        self.pb(chunk, bb, air, 6, 6, 10);
        self.pb(chunk, bb, air, WIDTH - 6, 5, 10);
        self.pb(chunk, bb, air, WIDTH - 6, 6, 10);
        self.pb(chunk, bb, air, WIDTH - 7, 6, 10);

        self.gen_box(chunk, bb, 2, 4, 4, 2, 6, 4, air, air, false);
        self.gen_box(chunk, bb, WIDTH - 3, 4, 4, WIDTH - 3, 6, 4, air, air, false);
        self.pb(chunk, bb, north_stairs, 2, 4, 5);
        self.pb(chunk, bb, north_stairs, 2, 3, 4);
        self.pb(chunk, bb, north_stairs, WIDTH - 3, 4, 5);
        self.pb(chunk, bb, north_stairs, WIDTH - 3, 3, 4);

        self.gen_box(chunk, bb, 1, 1, 3, 2, 2, 3, ss, ss, false);
        self.gen_box(chunk, bb, WIDTH - 3, 1, 3, WIDTH - 2, 2, 3, ss, ss, false);
        self.pb(chunk, bb, ss, 1, 1, 2);
        self.pb(chunk, bb, ss, WIDTH - 2, 1, 2);
        self.pb(chunk, bb, slab, 1, 2, 2);
        self.pb(chunk, bb, slab, WIDTH - 2, 2, 2);
        self.pb(chunk, bb, west_stairs, 2, 1, 2);
        self.pb(chunk, bb, east_stairs, WIDTH - 3, 1, 2);

        self.gen_box(chunk, bb, 4, 3, 5, 4, 3, 17, ss, ss, false);
        self.gen_box(chunk, bb, WIDTH - 5, 3, 5, WIDTH - 5, 3, 17, ss, ss, false);
        self.gen_box(chunk, bb, 3, 1, 5, 4, 2, 16, air, air, false);
        self.gen_box(
            chunk,
            bb,
            WIDTH - 6,
            1,
            5,
            WIDTH - 5,
            2,
            16,
            air,
            air,
            false,
        );

        for z in (5..=17).step_by(2) {
            self.pb(chunk, bb, cut, 4, 1, z);
            self.pb(chunk, bb, chiseled, 4, 2, z);
            self.pb(chunk, bb, cut, WIDTH - 5, 1, z);
            self.pb(chunk, bb, chiseled, WIDTH - 5, 2, z);
        }

        self.pb(chunk, bb, orange, 10, 0, 7);
        self.pb(chunk, bb, orange, 10, 0, 8);
        self.pb(chunk, bb, orange, 9, 0, 9);
        self.pb(chunk, bb, orange, 11, 0, 9);
        self.pb(chunk, bb, orange, 8, 0, 10);
        self.pb(chunk, bb, orange, 12, 0, 10);
        self.pb(chunk, bb, orange, 7, 0, 10);
        self.pb(chunk, bb, orange, 13, 0, 10);
        self.pb(chunk, bb, orange, 9, 0, 11);
        self.pb(chunk, bb, orange, 11, 0, 11);
        self.pb(chunk, bb, orange, 10, 0, 12);
        self.pb(chunk, bb, orange, 10, 0, 13);
        self.pb(chunk, bb, blue, 10, 0, 10);

        for x in [0, WIDTH - 1] {
            self.pb(chunk, bb, cut, x, 2, 1);
            self.pb(chunk, bb, orange, x, 2, 2);
            self.pb(chunk, bb, cut, x, 2, 3);
            self.pb(chunk, bb, cut, x, 3, 1);
            self.pb(chunk, bb, orange, x, 3, 2);
            self.pb(chunk, bb, cut, x, 3, 3);
            self.pb(chunk, bb, orange, x, 4, 1);
            self.pb(chunk, bb, chiseled, x, 4, 2);
            self.pb(chunk, bb, orange, x, 4, 3);
            self.pb(chunk, bb, cut, x, 5, 1);
            self.pb(chunk, bb, orange, x, 5, 2);
            self.pb(chunk, bb, cut, x, 5, 3);
            self.pb(chunk, bb, orange, x, 6, 1);
            self.pb(chunk, bb, chiseled, x, 6, 2);
            self.pb(chunk, bb, orange, x, 6, 3);
            self.pb(chunk, bb, orange, x, 7, 1);
            self.pb(chunk, bb, orange, x, 7, 2);
            self.pb(chunk, bb, orange, x, 7, 3);
            self.pb(chunk, bb, cut, x, 8, 1);
            self.pb(chunk, bb, cut, x, 8, 2);
            self.pb(chunk, bb, cut, x, 8, 3);
        }

        for x in [2, WIDTH - 3] {
            self.pb(chunk, bb, cut, x - 1, 2, 0);
            self.pb(chunk, bb, orange, x, 2, 0);
            self.pb(chunk, bb, cut, x + 1, 2, 0);
            self.pb(chunk, bb, cut, x - 1, 3, 0);
            self.pb(chunk, bb, orange, x, 3, 0);
            self.pb(chunk, bb, cut, x + 1, 3, 0);
            self.pb(chunk, bb, orange, x - 1, 4, 0);
            self.pb(chunk, bb, chiseled, x, 4, 0);
            self.pb(chunk, bb, orange, x + 1, 4, 0);
            self.pb(chunk, bb, cut, x - 1, 5, 0);
            self.pb(chunk, bb, orange, x, 5, 0);
            self.pb(chunk, bb, cut, x + 1, 5, 0);
            self.pb(chunk, bb, orange, x - 1, 6, 0);
            self.pb(chunk, bb, chiseled, x, 6, 0);
            self.pb(chunk, bb, orange, x + 1, 6, 0);
            self.pb(chunk, bb, orange, x - 1, 7, 0);
            self.pb(chunk, bb, orange, x, 7, 0);
            self.pb(chunk, bb, orange, x + 1, 7, 0);
            self.pb(chunk, bb, cut, x - 1, 8, 0);
            self.pb(chunk, bb, cut, x, 8, 0);
            self.pb(chunk, bb, cut, x + 1, 8, 0);
        }

        self.gen_box(chunk, bb, 8, 4, 0, 12, 6, 0, cut, cut, false);
        self.pb(chunk, bb, air, 8, 6, 0);
        self.pb(chunk, bb, air, 12, 6, 0);
        self.pb(chunk, bb, orange, 9, 5, 0);
        self.pb(chunk, bb, chiseled, 10, 5, 0);
        self.pb(chunk, bb, orange, 11, 5, 0);

        self.gen_box(chunk, bb, 8, -14, 8, 12, -11, 12, cut, cut, false);
        self.gen_box(chunk, bb, 8, -10, 8, 12, -10, 12, chiseled, chiseled, false);
        self.gen_box(chunk, bb, 8, -9, 8, 12, -9, 12, cut, cut, false);
        self.gen_box(chunk, bb, 8, -8, 8, 12, -1, 12, ss, ss, false);
        self.gen_box(chunk, bb, 9, -11, 9, 11, -1, 11, air, air, false);
        self.pb(
            chunk,
            bb,
            Block::STONE_PRESSURE_PLATE.default_state,
            10,
            -11,
            10,
        );
        self.gen_box(
            chunk,
            bb,
            9,
            -13,
            9,
            11,
            -13,
            11,
            Block::TNT.default_state,
            air,
            false,
        );

        self.pb(chunk, bb, air, 8, -11, 10);
        self.pb(chunk, bb, air, 8, -10, 10);
        self.pb(chunk, bb, chiseled, 7, -10, 10);
        self.pb(chunk, bb, cut, 7, -11, 10);
        self.pb(chunk, bb, air, 12, -11, 10);
        self.pb(chunk, bb, air, 12, -10, 10);
        self.pb(chunk, bb, chiseled, 13, -10, 10);
        self.pb(chunk, bb, cut, 13, -11, 10);
        self.pb(chunk, bb, air, 10, -11, 8);
        self.pb(chunk, bb, air, 10, -10, 8);
        self.pb(chunk, bb, chiseled, 10, -10, 7);
        self.pb(chunk, bb, cut, 10, -11, 7);
        self.pb(chunk, bb, air, 10, -11, 12);
        self.pb(chunk, bb, air, 10, -10, 12);
        self.pb(chunk, bb, chiseled, 10, -10, 13);
        self.pb(chunk, bb, cut, 10, -11, 13);

        self.try_place_chest(chunk, bb, 0, 10, -11, 12);
        self.try_place_chest(chunk, bb, 1, 8, -11, 10);
        self.try_place_chest(chunk, bb, 2, 10, -11, 8);
        self.try_place_chest(chunk, bb, 3, 12, -11, 10);

        self.add_cellar(chunk, bb, random);
    }
}
