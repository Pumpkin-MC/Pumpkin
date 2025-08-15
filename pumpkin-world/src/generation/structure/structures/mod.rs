use pumpkin_util::math::position::BlockPos;

use crate::ProtoChunk;

pub mod buried_treasure;

pub trait Structure {
    fn get_structure_position(&self, chunk: &ProtoChunk) -> StructurePosition;

    fn generate(&self, chunk: &mut crate::ProtoChunk);
}

pub struct StructurePosition {
    position: BlockPos,
    generator: StructurePiecesCollector,
}

#[derive(Default)]
pub struct StructurePiecesCollector {
    pieces_positions: Vec<BlockPos>,
}
