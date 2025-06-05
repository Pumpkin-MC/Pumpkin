use pumpkin_data::{Block, block_properties::get_state_by_state_id, tag::Tagable};
use pumpkin_util::math::position::BlockPos;

use crate::ProtoChunk;

pub mod cluster;
pub mod large;
pub mod small;

pub(super) fn can_replace(block: &Block) -> bool {
    block == &Block::DRIPSTONE_BLOCK
        || block
            .is_tagged_with("minecraft:dripstone_replaceable_blocks")
            .unwrap()
}

pub(super) fn gen_dripstone(chunk: &mut ProtoChunk, pos: BlockPos) -> bool {
    let block = chunk.get_block_state(&pos.0).to_block();
    if block
        .is_tagged_with("minecraft:dripstone_replaceable_blocks")
        .unwrap()
    {
        chunk.set_block_state(
            &pos.0,
            &get_state_by_state_id(Block::DRIPSTONE_BLOCK.default_state_id).unwrap(),
        );
        return true;
    }
    false
}
