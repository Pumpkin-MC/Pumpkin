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
            stronghold::{EntranceType, StoneBrickRandomizer, StrongholdPiece},
        },
    },
};

#[derive(Clone)]
pub struct PrisonHallPiece {
    piece: StrongholdPiece,
}

impl PrisonHallPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        // Main Outline 9x5x11 (0..8, 0..4, 0..10)
        let bounding_box = BlockBox::rotated(x, y, z, -1, -1, 0, 9, 5, 11, &orientation);

        if !StrongholdPiece::is_in_bounds(&bounding_box)
            || collector.get_intersecting(&bounding_box).is_some()
        {
            return None;
        }

        let mut piece = StrongholdPiece::new(
            StructurePieceType::StrongholdPrisonHall,
            chain_length,
            bounding_box,
        );
        piece.piece.set_facing(Some(orientation));
        piece.entry_door = EntranceType::get_random(random);

        Some(Box::new(Self { piece }))
    }
}

impl StructurePieceBase for PrisonHallPiece {
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
        collector: &mut StructurePiecesCollector,
        pieces_to_process: &mut Vec<Box<dyn StructurePieceBase>>,
    ) {
        // Prison hall usually only has one forward exit
        self.piece
            .fill_forward_opening(start, collector, random, 1, 1, pieces_to_process, None);
    }

    fn place(&mut self, chunk: &mut ProtoChunk, random: &mut RandomGenerator, _seed: i64) {
        let randomizer = StoneBrickRandomizer;
        let box_limit = self.piece.piece.bounding_box;
        let p = &self.piece;
        let inner = &p.piece;
        let air = Block::AIR.default_state;

        // 1. Main Shell 9x5x11
        inner.fill_outline_random(
            0,
            0,
            0,
            8,
            4,
            10,
            &randomizer,
            chunk,
            true,
            random,
            &box_limit,
        );

        // 2. Entrance (Z=0)
        p.generate_entrance(chunk, &box_limit, self.piece.entry_door, 1, 1, 0);

        // 3. Exit (Forward/Z=10) - Using fill_with_outline (cant_replace_air = false)
        inner.fill_with_outline(chunk, &box_limit, false, 1, 1, 10, 3, 3, 10, air, air);

        // 4. Cell Stone Separators (X=4)
        // Note: Java passes 'false' for cant_replace_air here
        inner.fill_outline_random(
            4,
            1,
            1,
            4,
            3,
            1,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            4,
            1,
            3,
            4,
            3,
            3,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            4,
            1,
            7,
            4,
            3,
            7,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );
        inner.fill_outline_random(
            4,
            1,
            9,
            4,
            3,
            9,
            &randomizer,
            chunk,
            false,
            random,
            &box_limit,
        );

        // 5. Iron Bars with Directional Properties
        // let bars_ns = Block::IRON_BARS.default_state.with("north", "true").with("south", "true");
        // let bars_nse = bars_ns.with("east", "true");
        // let bars_we = Block::IRON_BARS.default_state.with("west", "true").with("east", "true");

        // for i in 1..=3 {
        //     inner.add_block(chunk, &bars_ns, 4, i, 4, &box_limit);
        //     inner.add_block(chunk, &bars_nse, 4, i, 5, &box_limit);
        //     inner.add_block(chunk, &bars_ns, 4, i, 6, &box_limit);

        //     inner.add_block(chunk, &bars_we, 5, i, 5, &box_limit);
        //     inner.add_block(chunk, &bars_we, 6, i, 5, &box_limit);
        //     inner.add_block(chunk, &bars_we, 7, i, 5, &box_limit);
        // }

        // // Top bars above the doors
        // inner.add_block(chunk, &bars_ns, 4, 3, 2, &box_limit);
        // inner.add_block(chunk, &bars_ns, 4, 3, 8, &box_limit);

        // // 6. Iron Doors (2-block high structure)
        // let door_bottom = Block::IRON_DOOR.default_state.with("facing", "west").with("half", "lower");
        // let door_top = Block::IRON_DOOR.default_state.with("facing", "west").with("half", "upper");

        // // Door 1
        // inner.add_block(chunk, &door_bottom, 4, 1, 2, &box_limit);
        // inner.add_block(chunk, &door_top, 4, 2, 2, &box_limit);

        // // Door 2
        // inner.add_block(chunk, &door_bottom, 4, 1, 8, &box_limit);
        // inner.add_block(chunk, &door_top, 4, 2, 8, &box_limit);
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new((*self).clone())
    }
}
