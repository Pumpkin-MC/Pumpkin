use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, GetStateForNeighborUpdateArgs, OnPlaceArgs,
};
use pumpkin_data::block_properties::{BlockProperties, BrownMushroomBlockLikeProperties};
use pumpkin_data::{BlockDirection, BlockId, BlockStateId};

/// Shared behaviour for `red_mushroom_block`, `brown_mushroom_block`, and `mushroom_stem`.
///
/// Vanilla: `net.minecraft.world.level.block.HugeMushroomBlock`.
type MushroomProperties = BrownMushroomBlockLikeProperties;

pub struct HugeMushroomBlock;

impl BlockMetadata for HugeMushroomBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::BROWN_MUSHROOM_BLOCK,
            BlockId::RED_MUSHROOM_BLOCK,
            BlockId::MUSHROOM_STEM,
        ]
        .into()
    }
}

/// Sets the boolean face property corresponding to `direction` on `props`.
///
/// Mirrors `HugeMushroomBlock.PROPERTY_BY_DIRECTION` (backed by `PipeBlock.PROPERTY_BY_DIRECTION`).
const fn set_face(props: &mut MushroomProperties, direction: BlockDirection, value: bool) {
    match direction {
        BlockDirection::Up => props.up = value,
        BlockDirection::Down => props.down = value,
        BlockDirection::North => props.north = value,
        BlockDirection::South => props.south = value,
        BlockDirection::East => props.east = value,
        BlockDirection::West => props.west = value,
    }
}

impl BlockBehaviour for HugeMushroomBlock {
    /// `HugeMushroomBlock.getStateForPlacement`: every face starts "outward" (true) except
    /// the ones touching an already-placed block of the exact same type (checked with `is(this)`,
    /// i.e. same block id, not just any huge-mushroom block).
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = MushroomProperties::default(args.block);
            for direction in BlockDirection::all() {
                let neighbor_pos = args.position.offset(direction.to_offset());
                let neighbor_block = args.world.get_block(&neighbor_pos);
                set_face(&mut props, direction, neighbor_block != args.block);
            }
            props.to_state_id(args.block)
        })
    }

    /// `HugeMushroomBlock.updateShape`: only ever clears a face to `false` when the neighbor in
    /// that direction is the exact same block; it never restores `true` (a one-way quirk that
    /// matches vanilla and must not be "fixed").
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let neighbor_block = args.world.get_block(args.neighbor_position);
            if neighbor_block != args.block {
                return args.state_id;
            }
            let mut props = MushroomProperties::from_state_id(args.state_id, args.block);
            set_face(&mut props, args.direction, false);
            props.to_state_id(args.block)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::Block;

    #[test]
    fn set_face_only_touches_requested_direction() {
        let mut props = MushroomProperties::default(&Block::BROWN_MUSHROOM_BLOCK);
        assert!(props.up && props.down && props.north);

        set_face(&mut props, BlockDirection::Up, false);
        assert!(!props.up);
        assert!(props.down && props.north && props.south && props.east && props.west);
    }
}
