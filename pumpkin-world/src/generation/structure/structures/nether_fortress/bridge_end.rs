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
            nether_fortress::{NetherFortressPiece, PieceWeight},
        },
    },
};

/// Dead-end / terminator piece for the exterior bridge (5 × 10 × 8).
/// Generated randomly on each side and used as a fallback when no valid piece
/// can be placed.
#[derive(Clone)]
pub struct BridgeEndPiece {
    pub piece: NetherFortressPiece,
    /// Seed used so the crumbling geometry is deterministic per world seed.
    pub seed: u32,
}

impl BridgeEndPiece {
    pub fn create(
        collector: &mut StructurePiecesCollector,
        random: &mut impl RandomImpl,
        x: i32,
        y: i32,
        z: i32,
        orientation: BlockDirection,
        chain_length: u32,
    ) -> Option<Box<dyn StructurePieceBase>> {
        let bbox = BlockBox::rotated(x, y, z, -1, -3, 0, 5, 10, 8, &orientation);
        if !NetherFortressPiece::is_in_bounds(&bbox) || collector.get_intersecting(&bbox).is_some()
        {
            return None;
        }

        let seed = random.next_i32() as u32;
        let mut piece = NetherFortressPiece::new(
            StructurePieceType::NetherFortressBridgeEnd,
            chain_length,
            bbox,
        );
        piece.piece.set_facing(Some(orientation));
        Some(Box::new(Self { piece, seed }))
    }
}

impl StructurePieceBase for BridgeEndPiece {
    fn get_structure_piece(&self) -> &StructurePiece {
        &self.piece.piece
    }

    fn get_structure_piece_mut(&mut self) -> &mut StructurePiece {
        &mut self.piece.piece
    }

    // BridgeEnd is a terminal piece – it never spawns children.
    fn fill_openings_nether(
        &self,
        _start: &StructurePiece,
        _random: &mut RandomGenerator,
        _bridge_pieces: &mut Vec<PieceWeight>,
        _corridor_pieces: &mut Vec<PieceWeight>,
        _collector: &mut StructurePiecesCollector,
        _pieces_to_process: &mut Vec<Box<dyn StructurePieceBase>>,
    ) {
    }

    fn place(
        &mut self,
        chunk: &mut ProtoChunk,
        _random: &mut RandomGenerator,
        _seed: i64,
        chunk_box: &BlockBox,
    ) {
        let bb = *chunk_box;
        let p = &self.piece.piece;
        let nb = Block::NETHER_BRICKS.default_state;

        // Use a seeded local random so geometry is deterministic.
        let mut rng = pumpkin_util::random::xoroshiro128::Xoroshiro::from_seed(self.seed as u64);

        for i in 0..=4i32 {
            for j in 3..=4i32 {
                let k = rng.next_bounded_i32(8);
                p.fill_with_outline(chunk, &bb, false, i, j, 0, i, j, k, nb, nb);
            }
        }

        let k0 = rng.next_bounded_i32(8);
        p.fill_with_outline(chunk, &bb, false, 0, 5, 0, 0, 5, k0, nb, nb);

        let k1 = rng.next_bounded_i32(8);
        p.fill_with_outline(chunk, &bb, false, 4, 5, 0, 4, 5, k1, nb, nb);

        for i in 0..=4i32 {
            let j = rng.next_bounded_i32(5);
            p.fill_with_outline(chunk, &bb, false, i, 2, 0, i, 2, j, nb, nb);
        }

        for i in 0..=4i32 {
            for j in 0..=1i32 {
                let k = rng.next_bounded_i32(3);
                p.fill_with_outline(chunk, &bb, false, i, j, 0, i, j, k, nb, nb);
            }
        }
    }

    fn clone_box(&self) -> Box<dyn StructurePieceBase> {
        Box::new(self.clone())
    }
}
