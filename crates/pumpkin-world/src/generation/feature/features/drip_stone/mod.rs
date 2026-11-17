use crate::generation::proto_chunk::GenerationCache;
use pumpkin_data::block_properties::{
    PointedDripstoneLikeProperties, SpeleothemThickness, VerticalDirection,
};
use pumpkin_data::tag;
use pumpkin_data::{Block, BlockDirection, BlockId, BlockState};
use pumpkin_util::math::position::BlockPos;

pub mod cluster;
pub mod large;
pub mod small;

pub(super) fn can_replace(id: BlockId) -> bool {
    id == BlockId::DRIPSTONE_BLOCK || id.has_tag(tag::Block::MINECRAFT_DRIPSTONE_REPLACEABLE_BLOCKS)
}

pub(super) fn gen_dripstone<T: GenerationCache>(chunk: &mut T, pos: BlockPos) -> bool {
    let block = GenerationCache::get_block_state(chunk, &pos.0).to_block_id();
    if block.has_tag(tag::Block::MINECRAFT_DRIPSTONE_REPLACEABLE_BLOCKS) {
        chunk.set_block_state(&pos.0, Block::DRIPSTONE_BLOCK.default_state);
        return true;
    }
    false
}

pub(super) fn is_empty_or_water<T: GenerationCache>(chunk: &T, pos: BlockPos) -> bool {
    let block = GenerationCache::get_block_state(chunk, &pos.0).to_block_id();
    block == BlockId::AIR
        || block == BlockId::CAVE_AIR
        || block == BlockId::VOID_AIR
        || block == BlockId::WATER
}

pub(super) fn grow_pointed_dripstone<T: GenerationCache>(
    chunk: &mut T,
    start_pos: BlockPos,
    tip_direction: BlockDirection,
    height: i32,
    merged_tip: bool,
) {
    let mut cur = start_pos;
    let vert_dir = match tip_direction {
        BlockDirection::Down => VerticalDirection::Down,
        _ => VerticalDirection::Up,
    };

    let mut thicknesses = Vec::with_capacity(height as usize);
    if height >= 3 {
        thicknesses.push(SpeleothemThickness::Base);
        for _ in 0..(height - 3) {
            thicknesses.push(SpeleothemThickness::Middle);
        }
    }
    if height >= 2 {
        thicknesses.push(SpeleothemThickness::Frustum);
    }
    if height >= 1 {
        thicknesses.push(if merged_tip {
            SpeleothemThickness::TipMerge
        } else {
            SpeleothemThickness::Tip
        });
    }

    for thickness in thicknesses {
        let is_water =
            GenerationCache::get_block_state(chunk, &cur.0).to_block_id() == BlockId::WATER;
        let mut props = PointedDripstoneLikeProperties::default(&Block::POINTED_DRIPSTONE);
        props.thickness = thickness;
        props.vertical_direction = vert_dir;
        props.waterlogged = is_water;
        let state_id = props.to_state_id(&Block::POINTED_DRIPSTONE);
        chunk.set_block_state(&cur.0, BlockState::from_id(state_id));
        cur = cur.offset(tip_direction.to_offset());
    }
}
