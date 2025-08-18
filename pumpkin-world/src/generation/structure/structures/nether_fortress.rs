use pumpkin_util::math::position::BlockPos;
use serde::Deserialize;

use crate::{
    ProtoChunk,
    generation::{
        positions::chunk_pos,
        structure::structures::{StructureGenerator, StructurePiecesCollector, StructurePosition},
    },
};

#[derive(Deserialize, Clone)]
pub struct NetherFortressGenerator;

impl StructureGenerator for NetherFortressGenerator {
    fn get_structure_position(&self, chunk: &ProtoChunk) -> StructurePosition {
        let chunk_pos = chunk.chunk_pos;
        let start_x = chunk_pos::start_block_x(&chunk_pos);
        let start_z = chunk_pos::start_block_z(&chunk_pos);
        let generator = StructurePiecesCollector {
            pieces_positions: vec![], // TODO
        };

        StructurePosition {
            position: BlockPos::new(start_x, 64, start_z),
            generator,
        }
    }
    fn generate(&self, _position: BlockPos, _chunk: &mut crate::ProtoChunk) {}
}
