use pumpkin_data::{block_properties::{BlockProperties, VineLikeProperties}, BlockDirection};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::{generation::Direction, BlockStateId};
use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs,
};



#[pumpkin_block("minecraft:vine")]
pub struct VineBlock;

impl BlockBehaviour for VineBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = VineLikeProperties::default(args.block);
            vine_direction_mapper(&args.direction, &mut props);
            props.to_state_id(args.block)
        })
    }
        fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let props = VineLikeProperties::from_state_id(args.state.id, args.block);
            let placed_on_state = args.block_accessor
                .get_block_state(&args.use_item_on.unwrap().position)
                .await;

                match args.direction {
                    Some(BlockDirection::Down) => false,
                    Some(dir) => {
                        placed_on_state.is_side_solid(dir)
                    },
                    None => false
                }
        })
    }
}


fn vine_block_direction(props: &VineLikeProperties) -> Option<BlockDirection> {
    if props.north { Some(BlockDirection::North) }
    else if props.south { Some(BlockDirection::South) }
    else if props.east { Some(BlockDirection::East) }
    else if props.west { Some(BlockDirection::West) }
    else if props.up { Some(BlockDirection::Up) }
    else { None }
}
fn vine_direction_mapper(direction: &BlockDirection, props: &mut VineLikeProperties) {
    match direction {
        BlockDirection::Down => (),
        BlockDirection::Up => props.up = true,
        BlockDirection::North => props.north = true,
        BlockDirection::South => props.south = true,
        BlockDirection::West => props.west = true,
        BlockDirection::East => props.east = true,
    }
}