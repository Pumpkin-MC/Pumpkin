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
pub struct ChestCorridorPiece {
    pub piece: StrongholdPiece,
    pub chest_generated: bool,
}

impl ChestCorridorPiece {
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
            StructurePieceType::StrongholdChestCorridor,
            chain_length,
            bounding_box,
        );
        piece.piece.set_facing(Some(orientation));
        piece.entry_door = EntranceType::get_random(random);

        Some(Box::new(Self {
            piece,
            chest_generated: false,
        }))
    }
}

impl StructurePieceBase for ChestCorridorPiece {
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

        let stone_bricks = Block::STONE_BRICKS.default_state;
        inner.fill_with_outline(
            chunk,
            &box_limit,
            false,
            3,
            1,
            2,
            3,
            1,
            4,
            stone_bricks,
            stone_bricks,
        );

        let slab = Block::STONE_BRICK_SLAB.default_state;
        inner.add_block(chunk, slab, 3, 1, 1, &box_limit);
        inner.add_block(chunk, slab, 3, 1, 5, &box_limit);
        inner.add_block(chunk, slab, 3, 2, 2, &box_limit);
        inner.add_block(chunk, slab, 3, 2, 4, &box_limit);

        for i in 2..=4 {
            inner.add_block(chunk, slab, 2, 1, i, &box_limit);
        }

        // if !self.chest_generated {
        //     // Check if the target chest position is within the current chunk being processed
        //     let chest_pos = inner.to_world(3, 2, 3);
        //     if box_limit.contains(&chest_pos) {
        //         self.chest_generated = true;

        //         let chest_state = Block::CHEST.default_state;
        //         // Note: In a full implementation, you would use a helper to set the LootTable NBT here
        //         inner.add_block(chunk, &chest_state, 3, 2, 3, &box_limit);

        //         // chunk.set_loot_table(chest_pos, "minecraft:chests/stronghold_corridor");
        //     }
        // }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new((*self).clone())
    }
}
