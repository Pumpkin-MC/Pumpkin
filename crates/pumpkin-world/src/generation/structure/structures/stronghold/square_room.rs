use pumpkin_data::{
    Block, BlockState,
    block_properties::{HorizontalFacing, LadderLikeProperties, WallTorchLikeProperties},
};
use pumpkin_util::{
    BlockDirection,
    math::block_box::BlockBox,
    random::{RandomGenerator, RandomImpl},
};

use crate::{
    ProtoChunk,
    generation::structure::{
        piece::StructurePieceType,
        structures::{
            StructurePiece, StructurePieceBase, StructurePiecesCollector,
            stronghold::{EntranceType, StoneBrickRandomizer, StrongholdPiece},
        },
    },
};

use crate::world::WorldPortalExt;

/// Loot table of the chest in the room variant that has one, matching vanilla
/// `StrongholdPieces$RoomCrossing`.
const CROSSING_LOOT_TABLE: &str = "minecraft:chests/stronghold_crossing";

pub struct SquareRoomPiece {
    pub piece: StrongholdPiece,
    pub room_type: u32,
}

impl SquareRoomPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bounding_box = BlockBox::rotated(x, y, z, -4, -1, 0, 11, 7, 11, &orientation);

        // 2. Check world bounds and intersections
        if !StrongholdPiece::is_in_bounds(&bounding_box)
            || collector.get_intersecting(&bounding_box).is_some()
        {
            return None;
        }

        // 3. Construct the piece
        let mut piece = StrongholdPiece::new(
            StructurePieceType::StrongholdSquareRoom,
            chain_length,
            bounding_box,
        );
        piece.piece.set_facing(Some(orientation));
        piece.entry_door = EntranceType::get_random(random);

        let room_type = random.next_bounded_i32(5) as u32;

        Some(Box::new(Self { piece, room_type }))
    }
}

impl StructurePieceBase for SquareRoomPiece {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece.piece
    }

    #[expect(clippy::too_many_lines)]
    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let randomizer = StoneBrickRandomizer;
        let box_limit = *chunk_box;
        let p = &self.piece;
        let inner = &p.piece;
        let air = Block::AIR.default_state;

        // 1. Main Outline (10x6x10) - Matches Java: fillWithOutline(..., 0,0,0, 10,6,10, ...)
        inner.fill_outline_random(
            0,
            0,
            0,
            10,
            6,
            10,
            &randomizer,
            chunk,
            true,
            random,
            &box_limit,
        );

        // 2. Main Entrance (Z=0, aligned to X=4)
        // Matches Java: generateEntrance(..., 4, 1, 0)
        p.generate_entrance(chunk, &box_limit, self.piece.entry_door, 4, 1, 0);

        // 3. Clear Exits
        // Forward Exit (Z=10) -> fillWithOutline(..., 4,1,10, 6,3,10, AIR, AIR, false)
        inner.fill_with_outline(chunk, &box_limit, false, 4, 1, 10, 6, 3, 10, air, air);

        // Left Side Exit (X=0) -> fillWithOutline(..., 0,1,4, 0,3,6, AIR, AIR, false)
        inner.fill_with_outline(chunk, &box_limit, false, 0, 1, 4, 0, 3, 6, air, air);

        // Right Side Exit (X=10) -> fillWithOutline(..., 10,1,4, 10,3,6, AIR, AIR, false)
        inner.fill_with_outline(chunk, &box_limit, false, 10, 1, 4, 10, 3, 6, air, air);

        // 4. Room Specific Decorations
        match self.room_type {
            0 => {
                let stone_brick = Block::STONE_BRICKS.default_state;
                let smooth_slab = Block::SMOOTH_STONE_SLAB.default_state;

                // Central Pillar
                inner.add_block(chunk, stone_brick, 5, 1, 5, &box_limit);
                inner.add_block(chunk, stone_brick, 5, 2, 5, &box_limit);
                inner.add_block(chunk, stone_brick, 5, 3, 5, &box_limit);

                // Torches
                let mut torch_props = WallTorchLikeProperties::default(&Block::WALL_TORCH);
                torch_props.facing = HorizontalFacing::West;
                inner.add_block(
                    chunk,
                    BlockState::from_id(torch_props.to_state_id(&Block::WALL_TORCH)),
                    4,
                    3,
                    5,
                    &box_limit,
                );
                torch_props.facing = HorizontalFacing::East;
                inner.add_block(
                    chunk,
                    BlockState::from_id(torch_props.to_state_id(&Block::WALL_TORCH)),
                    6,
                    3,
                    5,
                    &box_limit,
                );
                torch_props.facing = HorizontalFacing::South;
                inner.add_block(
                    chunk,
                    BlockState::from_id(torch_props.to_state_id(&Block::WALL_TORCH)),
                    5,
                    3,
                    4,
                    &box_limit,
                );
                torch_props.facing = HorizontalFacing::North;
                inner.add_block(
                    chunk,
                    BlockState::from_id(torch_props.to_state_id(&Block::WALL_TORCH)),
                    5,
                    3,
                    6,
                    &box_limit,
                );

                // Floor Pattern (Slabs)
                inner.add_block(chunk, smooth_slab, 4, 1, 4, &box_limit);
                inner.add_block(chunk, smooth_slab, 4, 1, 5, &box_limit);
                inner.add_block(chunk, smooth_slab, 4, 1, 6, &box_limit);
                inner.add_block(chunk, smooth_slab, 6, 1, 4, &box_limit);
                inner.add_block(chunk, smooth_slab, 6, 1, 5, &box_limit);
                inner.add_block(chunk, smooth_slab, 6, 1, 6, &box_limit);
                inner.add_block(chunk, smooth_slab, 5, 1, 4, &box_limit);
                inner.add_block(chunk, smooth_slab, 5, 1, 6, &box_limit);
            }
            1 => {
                let stone_brick = Block::STONE_BRICKS.default_state;

                // Ring of bricks
                for i in 0..5 {
                    inner.add_block(chunk, stone_brick, 3, 1, 3 + i, &box_limit);
                    inner.add_block(chunk, stone_brick, 7, 1, 3 + i, &box_limit);
                    inner.add_block(chunk, stone_brick, 3 + i, 1, 3, &box_limit);
                    inner.add_block(chunk, stone_brick, 3 + i, 1, 7, &box_limit);
                }

                // Central Water Column
                inner.add_block(chunk, stone_brick, 5, 1, 5, &box_limit);
                inner.add_block(chunk, stone_brick, 5, 2, 5, &box_limit);
                inner.add_block(chunk, stone_brick, 5, 3, 5, &box_limit);
                inner.add_block(chunk, Block::WATER.default_state, 5, 4, 5, &box_limit);
            }
            2 => {
                let cobble = Block::COBBLESTONE.default_state;

                // Cobblestone shelves/walls
                for i in 1..=9 {
                    inner.add_block(chunk, cobble, 1, 3, i, &box_limit);
                    inner.add_block(chunk, cobble, 9, 3, i, &box_limit);
                }
                for i in 1..=9 {
                    inner.add_block(chunk, cobble, i, 3, 1, &box_limit);
                    inner.add_block(chunk, cobble, i, 3, 9, &box_limit);
                }

                // Central structure supports
                inner.add_block(chunk, cobble, 5, 1, 4, &box_limit);
                inner.add_block(chunk, cobble, 5, 1, 6, &box_limit);
                inner.add_block(chunk, cobble, 5, 3, 4, &box_limit);
                inner.add_block(chunk, cobble, 5, 3, 6, &box_limit);
                inner.add_block(chunk, cobble, 4, 1, 5, &box_limit);
                inner.add_block(chunk, cobble, 6, 1, 5, &box_limit);
                inner.add_block(chunk, cobble, 4, 3, 5, &box_limit);
                inner.add_block(chunk, cobble, 6, 3, 5, &box_limit);

                // Pillars
                for i in 1..=3 {
                    inner.add_block(chunk, cobble, 4, i, 4, &box_limit);
                    inner.add_block(chunk, cobble, 6, i, 4, &box_limit);
                    inner.add_block(chunk, cobble, 4, i, 6, &box_limit);
                    inner.add_block(chunk, cobble, 6, i, 6, &box_limit);
                }

                inner.add_block(chunk, Block::WALL_TORCH.default_state, 5, 3, 5, &box_limit);

                // Upper floor (Oak Planks)
                let oak = Block::OAK_PLANKS.default_state;
                for i in 2..=8 {
                    inner.add_block(chunk, oak, 2, 3, i, &box_limit);
                    inner.add_block(chunk, oak, 3, 3, i, &box_limit);

                    // Holes for ladders/structure
                    if i <= 3 || i >= 7 {
                        inner.add_block(chunk, oak, 4, 3, i, &box_limit);
                        inner.add_block(chunk, oak, 5, 3, i, &box_limit);
                        inner.add_block(chunk, oak, 6, 3, i, &box_limit);
                    }
                    inner.add_block(chunk, oak, 7, 3, i, &box_limit);
                    inner.add_block(chunk, oak, 8, 3, i, &box_limit);
                }

                // Ladder
                let mut props = LadderLikeProperties::default(&Block::LADDER);
                props.facing = HorizontalFacing::West;
                let ladder = BlockState::from_id(props.to_state_id(&Block::LADDER));
                inner.add_block(chunk, ladder, 9, 1, 3, &box_limit);
                inner.add_block(chunk, ladder, 9, 2, 3, &box_limit);
                inner.add_block(chunk, ladder, 9, 3, 3, &box_limit);

                // Chest
                inner.add_chest(chunk, &box_limit, random, 3, 4, 8, CROSSING_LOOT_TABLE);
            }
            _ => {}
        }
    }
}
