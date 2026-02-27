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

pub struct SpiralStaircasePiece {
    pub piece: StrongholdPiece,
    orientation: BlockDirection,
    is_structure_start: bool,
    // Start-specific fields
    pub last_piece_data_idx: Option<usize>,
    pub portal_room_exists: bool,
    pub pieces: Vec<Box<dyn StructurePieceBase>>,
}
impl Clone for SpiralStaircasePiece {
    fn clone(&self) -> Self {
        Self {
            // StrongholdPiece and other types must derive Clone
            piece: self.piece.clone(),
            orientation: self.orientation,
            is_structure_start: self.is_structure_start,
            last_piece_data_idx: self.last_piece_data_idx,
            portal_room_exists: self.portal_room_exists,
            // Use the clone_box helper we added to the trait!
            pieces: self.pieces.iter().map(|p| p.clone_box()).collect(),
        }
    }
}

impl SpiralStaircasePiece {
    /// Matches Start(Random random, int i, int j)
    pub fn new_start(random: &mut impl RandomImpl, x: i32, z: i32) -> Self {
        let orientation = StructurePiece::get_random_horizontal_direction(random);
        let bounding_box = BlockBox::create_box(x, 64, z, orientation.get_axis(), 5, 11, 5);
        let mut piece = StrongholdPiece::new(StructurePieceType::StrongholdStart, 0, bounding_box);
        piece.piece.set_facing(Some(orientation));
        Self {
            piece,
            orientation,
            is_structure_start: true,
            last_piece_data_idx: None,
            portal_room_exists: false,
            pieces: Vec::new(),
        }
    }

    pub fn create(
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bounding_box = BlockBox::rotated(x, y, z, -1, -7, 0, 5, 11, 5, &orientation);

        if !StrongholdPiece::is_in_bounds(&bounding_box)
            || collector.get_intersecting(&bounding_box).is_some()
        {
            return None;
        }

        let mut piece = StrongholdPiece::new(
            StructurePieceType::StrongholdSpiralStaircase,
            chain_length,
            bounding_box,
        );
        piece.piece.set_facing(Some(orientation));
        piece.entry_door = EntranceType::get_random(random);

        Some(Box::new(Self {
            piece,
            orientation,
            is_structure_start: false,
            last_piece_data_idx: None,
            portal_room_exists: false,
            pieces: Vec::new(),
        }))
    }

    #[must_use]
    pub const fn has_portal_room(&self) -> bool {
        self.portal_room_exists
    }
}

impl StructurePieceBase for SpiralStaircasePiece {
    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new((*self).clone())
    }

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
            self.is_structure_start
                .then_some(StrongholdPieceType::FiveWayCrossing),
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

        // 1. fillWithOutline (5x11x5 box)
        p.piece.fill_outline_random(
            0,
            0,
            0,
            4,
            10,
            4,
            &randomizer,
            chunk,
            true,
            random,
            &box_limit,
        );

        p.generate_entrance(chunk, &box_limit, self.piece.entry_door, 1, 7, 0);

        p.generate_entrance(chunk, &box_limit, EntranceType::Opening, 1, 1, 4);

        // 4. Spiral Staircase Blocks
        let stone = Block::STONE_BRICKS.default_state;
        let slab = Block::SMOOTH_STONE_SLAB.default_state;

        // Use the inner StructurePiece for block placement helpers
        let inner = &p.piece;
        inner.add_block(chunk, stone, 2, 6, 1, &box_limit);
        inner.add_block(chunk, stone, 1, 5, 1, &box_limit);
        inner.add_block(chunk, slab, 1, 6, 1, &box_limit);
        inner.add_block(chunk, stone, 1, 5, 2, &box_limit);
        inner.add_block(chunk, stone, 1, 4, 3, &box_limit);
        inner.add_block(chunk, slab, 1, 5, 3, &box_limit);
        inner.add_block(chunk, stone, 2, 4, 3, &box_limit);
        inner.add_block(chunk, stone, 3, 3, 3, &box_limit);
        inner.add_block(chunk, slab, 3, 4, 3, &box_limit);
        inner.add_block(chunk, stone, 3, 3, 2, &box_limit);
        inner.add_block(chunk, stone, 3, 2, 1, &box_limit);
        inner.add_block(chunk, slab, 3, 3, 1, &box_limit);
        inner.add_block(chunk, stone, 2, 2, 1, &box_limit);
        inner.add_block(chunk, stone, 1, 1, 1, &box_limit);
        inner.add_block(chunk, slab, 1, 2, 1, &box_limit);

        // Final bottom blocks from Java source
        inner.add_block(chunk, stone, 1, 1, 2, &box_limit);
        inner.add_block(chunk, slab, 1, 1, 3, &box_limit);
    }
}
