use crate::generation::proto_chunk::GenerationCache;
use pumpkin_data::Block;
use pumpkin_data::tag;
use pumpkin_util::math::position::BlockPos;

pub mod cluster;
pub mod large;
pub mod small;

pub(super) fn can_replace(block: u16) -> bool {
    block == Block::DRIPSTONE_BLOCK
        || tag::Block::MINECRAFT_DRIPSTONE_REPLACEABLE_BLOCKS
            .1
            .contains(&block)
}

pub(super) fn gen_dripstone<T: GenerationCache>(chunk: &mut T, pos: BlockPos) -> bool {
    let block = GenerationCache::get_block_state(chunk, &pos.0).to_block_id();
    if tag::Block::MINECRAFT_DRIPSTONE_REPLACEABLE_BLOCKS
        .1
        .contains(&block)
    {
        chunk.set_block_state(&pos.0, Block::DRIPSTONE_BLOCK.default_state);
        return true;
    }
    false
}
