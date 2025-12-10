use pumpkin_util::math::block_box::BlockBox;
use serde::Deserialize;

use crate::{ProtoChunk, generation::structure::structures::StructureGenerator};

#[derive(Deserialize, Clone, Debug)]
pub struct SwampHutGenerator;

impl StructureGenerator for SwampHutGenerator {
    fn get_structure_position(&self, _chunk: &ProtoChunk) -> super::StructurePosition {
        todo!()
    }

    fn generate(&self, _position: BlockBox, _chunk: &mut crate::ProtoChunk) {
        todo!()
    }
}
