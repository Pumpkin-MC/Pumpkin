use pumpkin_data::Block;
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
            stronghold::{
                EntranceType, PieceWeight, StoneBrickRandomizer, StrongholdPiece,
                StrongholdPieceType,
            },
        },
    },
};

#[derive(Clone)]
pub struct CorridorPiece {
    piece: StrongholdPiece,
    left_exit: bool,
    right_exit: bool,
}

impl CorridorPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bounding_box = BlockBox::rotated(x, y, z, -1, -1, 0, 5, 5, 7, &orientation);

        if !StrongholdPiece::is_in_bounds(&bounding_box)
            || collector.get_intersecting(&bounding_box).is_some()
        {
            return None;
        }

        let mut piece = StrongholdPiece::new(
            StructurePieceType::StrongholdCorridor,
            chain_length,
            bounding_box,
        );
        piece.piece.set_facing(Some(orientation));
        piece.entry_door = EntranceType::get_random(random);

        Some(Box::new(Self {
            piece,
            left_exit: random.next_bounded_i32(2) == 0,
            right_exit: random.next_bounded_i32(2) == 0,
        }))
    }
}

impl StructurePieceBase for CorridorPiece {
    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece.piece
    }

    fn fill_openings(
        &self,
        start: &StructurePiece,
        random: &mut RandomGenerator,
        weights: &mut Vec<PieceWeight>,
        last_piece_type: &mut Option<StrongholdPieceType>,
        _has_portal_room: &mut bool,

        collector: &mut StructurePiecesCollector,
        pieces_to_process: &mut Vec<Box<dyn StructurePieceBase>>,
    ) {
        self.piece.fill_forward_opening(
            start,
            collector,
            random,
            weights,
            last_piece_type,
            1,
            1,
            pieces_to_process,
            None,
        );

        if self.left_exit {
            self.piece.fill_nw_opening(
                start,
                collector,
                random,
                weights,
                last_piece_type,
                1,
                2,
                pieces_to_process,
            );
        }

        if self.right_exit {
            self.piece.fill_se_opening(
                start,
                collector,
                random,
                weights,
                last_piece_type,
                1,
                2,
                pieces_to_process,
            );
        }
    }

    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let randomizer = StoneBrickRandomizer;
        let box_limit = *chunk_box;
        let p = &self.piece;
        let inner = &p.piece;
        let air = Block::AIR.default_state;

        inner.fill_outline_random(
            0,
            0,
            0,
            4,
            4,
            6,
            &randomizer,
            chunk,
            true,
            random,
            &box_limit,
        );

        p.generate_entrance(chunk, &box_limit, self.piece.entry_door, 1, 1, 0);
        p.generate_entrance(chunk, &box_limit, EntranceType::Opening, 1, 1, 6);

        // let torch_east = Block::WALL_TORCH.default_state.with("facing", "east");
        // let torch_west = Block::WALL_TORCH.default_state.with("facing", "west");

        // inner.add_block_with_random_threshold(chunk, &box_limit, random, 0.1, 1, 2, 1, &torch_east);
        // inner.add_block_with_random_threshold(chunk, &box_limit, random, 0.1, 3, 2, 1, &torch_west);
        // inner.add_block_with_random_threshold(chunk, &box_limit, random, 0.1, 1, 2, 5, &torch_east);
        // inner.add_block_with_random_threshold(chunk, &box_limit, random, 0.1, 3, 2, 5, &torch_west);

        if self.left_exit {
            inner.fill_with_outline(chunk, &box_limit, false, 0, 1, 2, 0, 3, 4, air, air);
        }
        if self.right_exit {
            inner.fill_with_outline(chunk, &box_limit, false, 4, 1, 2, 4, 3, 4, air, air);
        }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new((*self).clone())
    }
}

#[derive(Clone)]
pub struct SmallCorridorPiece {
    pub piece: StrongholdPiece,
    pub length: i32,
}

impl SmallCorridorPiece {
    #[must_use]
    pub const fn new(chain_length: u32, bbox: BlockBox, orientation: BlockDirection) -> Self {
        let length = match orientation {
            BlockDirection::North | BlockDirection::South => bbox.max.z - bbox.min.z + 1,
            _ => bbox.max.x - bbox.min.x + 1,
        };

        let mut piece = StrongholdPiece::new(
            StructurePieceType::StrongholdSmallCorridor,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(orientation));

        Self { piece, length }
    }

    /// Port of the specialized static 'create' logic
    pub fn create_box(
        holder: &mut StructurePiecesCollector,
        x: i32,
        y: i32,
        z: i32,
        orientation: &BlockDirection,
    ) -> Option<BlockBox> {
        // Initial attempt: 5x5x4 box
        let mut block_box = BlockBox::rotated(x, y, z, -1, -1, 0, 5, 5, 4, orientation);

        // Find if we hit something
        let intersecting = holder.get_intersecting(&block_box)?;

        // Vanilla Logic: If we hit something at the same height, try to shorten the corridor
        // so it perfectly abuts the intersecting piece.
        if intersecting.bounding_box().min.y == block_box.min.y {
            for j in (1..=2).rev() {
                block_box = BlockBox::rotated(x, y, z, -1, -1, 0, 5, 5, j, orientation);

                if !intersecting.bounding_box().intersects(&block_box) {
                    // Found a shorter length that doesn't intersect!
                    // Return the next size up (j + 1) to seal the gap.
                    return Some(BlockBox::rotated(
                        x,
                        y,
                        z,
                        -1,
                        -1,
                        0,
                        5,
                        5,
                        j + 1,
                        orientation,
                    ));
                }
            }
        }

        None
    }
}

impl StructurePieceBase for SmallCorridorPiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new((*self).clone())
    }

    fn get_structure_piece(&self) -> &crate::generation::structure::structures::StructurePiece {
        &self.piece.piece
    }

    fn get_structure_piece_mut(
        &mut self,
    ) -> &mut crate::generation::structure::structures::StructurePiece {
        &mut self.piece.piece
    }

    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let box_limit = *chunk_box;
        let p = &self.piece.piece;

        let stone = Block::STONE_BRICKS.default_state;
        let cave_air = Block::CAVE_AIR.default_state;

        for i in 0..self.length {
            // 1. Floor (Y=0)
            p.add_block(chunk, stone, 0, 0, i, &box_limit);
            p.add_block(chunk, stone, 1, 0, i, &box_limit);
            p.add_block(chunk, stone, 2, 0, i, &box_limit);
            p.add_block(chunk, stone, 3, 0, i, &box_limit);
            p.add_block(chunk, stone, 4, 0, i, &box_limit);

            for j in 1..=3 {
                p.add_block(chunk, stone, 0, j, i, &box_limit); // Left Wall
                p.add_block(chunk, cave_air, 1, j, i, &box_limit); // Air Gap
                p.add_block(chunk, cave_air, 2, j, i, &box_limit); // Air Gap
                p.add_block(chunk, cave_air, 3, j, i, &box_limit); // Air Gap
                p.add_block(chunk, stone, 4, j, i, &box_limit); // Right Wall
            }

            p.add_block(chunk, stone, 0, 4, i, &box_limit);
            p.add_block(chunk, stone, 1, 4, i, &box_limit);
            p.add_block(chunk, stone, 2, 4, i, &box_limit);
            p.add_block(chunk, stone, 3, 4, i, &box_limit);
            p.add_block(chunk, stone, 4, 4, i, &box_limit);
        }
    }
}
