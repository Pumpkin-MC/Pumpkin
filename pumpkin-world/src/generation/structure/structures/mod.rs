use pumpkin_util::math::position::BlockPos;

use crate::ProtoChunk;

pub mod buried_treasure;
pub mod nether_fortress;

pub trait StructureGenerator {
    fn get_structure_position(&self, chunk: &ProtoChunk) -> StructurePosition;

    fn generate(&self, position: BlockPos, chunk: &mut crate::ProtoChunk);
}

#[derive(Clone)]
pub struct StructurePosition {
    pub position: BlockPos,
    pub generator: StructurePiecesCollector,
}

#[derive(Default, Clone)]
pub struct StructurePiecesCollector {
    pub pieces_positions: Vec<BlockPos>,
}
