use std::sync::Arc;

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockMetadata, BrokenArgs, CanPlaceAtArgs,
        GetStateForNeighborUpdateArgs, OnPlaceArgs, PlacedArgs,
    },
    world::World,
};
use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{
        BlockProperties, PointedDripstoneLikeProperties, Thickness, VerticalDirection,
    },
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    world::{BlockAccessor, BlockFlags},
};
pub struct DripstoneBlock;

impl BlockMetadata for DripstoneBlock {
    fn ids() -> Box<[u16]> {
        [Block::POINTED_DRIPSTONE.id].into()
    }
}

impl BlockBehaviour for DripstoneBlock {
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            can_place_at_pos(args.block_accessor, args.position, args.direction).await
        })
    }
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut dripstone_props = PointedDripstoneLikeProperties::default(args.block);
            dripstone_props.waterlogged = args.replacing.water_source();
            dripstone_props.vertical_direction =
                flip_dir(block_direction_to_vertical_direction(args.direction));
            dripstone_props.to_state_id(&Block::POINTED_DRIPSTONE)
        })
    }
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let (len, vertical_dir) =
                get_stalagmite_or_stalactice_len_and_dir_from_tip_pos(args.world, args.position)
                    .await;
            println!("place: len: {len} ver_dir: {:?}", vertical_dir);
            match vertical_dir {
                VerticalDirection::Up => {
                    update_stalagmite(args.world, len, args.position).await;
                }
                VerticalDirection::Down => {
                    update_stalactite(args.world, len, args.position).await;
                }
            }
        })
    }
    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let broken_dripstone_props =
                PointedDripstoneLikeProperties::from_state_id(args.state.id, args.block);
            let new_tip_pos = match broken_dripstone_props.vertical_direction {
                VerticalDirection::Up => args.position.down(),
                VerticalDirection::Down => args.position.up(),
            };

            let (len, vertical_dir) =
                get_stalagmite_or_stalactice_len_and_dir_from_tip_pos(args.world, &new_tip_pos)
                    .await;
            println!("break: len: {len} ver_dir: {:?}", vertical_dir);
            match vertical_dir {
                VerticalDirection::Up => {
                    update_stalagmite(args.world, len, &new_tip_pos).await;
                }
                VerticalDirection::Down => {
                    update_stalactite(args.world, len, &new_tip_pos).await;
                }
            }
        })
    }
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if !can_place_at_pos(args.world, args.position, None).await {
                return Block::AIR.default_state.id;
            }
            args.state_id
        })
    }
}
async fn update_stalagmite<'a>(world: &'a Arc<World>, stalagmite_len: u8, tip_pos: &BlockPos) {
    match stalagmite_len {
        1 => {
            modify_dripstone_thickness_to(world, tip_pos, Thickness::Tip).await;
        }
        2 => {
            modify_dripstone_thickness_to(world, tip_pos, Thickness::Tip).await;
            modify_dripstone_thickness_to(world, &tip_pos.down_height(1), Thickness::Frustum).await
        }
        3 => {
            modify_dripstone_thickness_to(world, tip_pos, Thickness::Tip).await;
            modify_dripstone_thickness_to(world, &tip_pos.down_height(1), Thickness::Frustum).await;
            modify_dripstone_thickness_to(world, &tip_pos.down_height(2), Thickness::Base).await;
        }
        4 => {
            modify_dripstone_thickness_to(world, tip_pos, Thickness::Tip).await;
            modify_dripstone_thickness_to(world, &tip_pos.down_height(1), Thickness::Frustum).await;
            modify_dripstone_thickness_to(world, &tip_pos.down_height(2), Thickness::Middle).await;
            modify_dripstone_thickness_to(world, &tip_pos.down_height(3), Thickness::Base).await;
        }
        5 => {
            modify_dripstone_thickness_to(world, tip_pos, Thickness::Tip).await;
            modify_dripstone_thickness_to(world, &tip_pos.down_height(1), Thickness::Frustum).await;
            modify_dripstone_thickness_to(world, &tip_pos.down_height(2), Thickness::Middle).await;
            modify_dripstone_thickness_to(world, &tip_pos.down_height(3), Thickness::Middle).await;
        }
        _ => {}
    };
}

async fn update_stalactite<'a>(world: &'a Arc<World>, stalagmite_len: u8, tip_pos: &BlockPos) {
    match stalagmite_len {
        1 => {
            modify_dripstone_thickness_to(world, tip_pos, Thickness::Tip).await;
        }
        2 => {
            modify_dripstone_thickness_to(world, tip_pos, Thickness::Tip).await;
            modify_dripstone_thickness_to(world, &tip_pos.up_height(1), Thickness::Frustum).await;
        }
        3 => {
            modify_dripstone_thickness_to(world, tip_pos, Thickness::Tip).await;
            modify_dripstone_thickness_to(world, &tip_pos.up_height(1), Thickness::Frustum).await;
            modify_dripstone_thickness_to(world, &tip_pos.up_height(2), Thickness::Base).await;
        }
        4 => {
            modify_dripstone_thickness_to(world, tip_pos, Thickness::Tip).await;
            modify_dripstone_thickness_to(world, &tip_pos.up_height(1), Thickness::Frustum).await;
            modify_dripstone_thickness_to(world, &tip_pos.up_height(2), Thickness::Middle).await;
            modify_dripstone_thickness_to(world, &tip_pos.up_height(3), Thickness::Base).await;
        }
        5 => {
            modify_dripstone_thickness_to(world, tip_pos, Thickness::Tip).await;
            modify_dripstone_thickness_to(world, &tip_pos.up_height(1), Thickness::Frustum).await;
            modify_dripstone_thickness_to(world, &tip_pos.up_height(2), Thickness::Middle).await;
            modify_dripstone_thickness_to(world, &tip_pos.up_height(3), Thickness::Middle).await;
        }
        _ => {}
    };
}
async fn get_stalagmite_or_stalactice_len_and_dir_from_tip_pos<'a>(
    world: &'a Arc<World>,
    position: &BlockPos,
) -> (u8, VerticalDirection) {
    let (block, state) = world.get_block_and_state(position).await;

    let props = PointedDripstoneLikeProperties::from_state_id(state.id, block);

    let mut dripstone_len = 1;
    let mut next_dripstone_pos = offset_pos_by_vertical_dir(position, props.vertical_direction);
    while dripstone_len < 5 {
        if world.get_block(&next_dripstone_pos).await != &Block::POINTED_DRIPSTONE {
            break;
        }
        next_dripstone_pos =
            offset_pos_by_vertical_dir(&next_dripstone_pos, props.vertical_direction);
        dripstone_len += 1;
    }
    (dripstone_len, props.vertical_direction)
}
fn offset_pos_by_vertical_dir(pos: &BlockPos, ver_dir: VerticalDirection) -> BlockPos {
    match ver_dir {
        VerticalDirection::Up => pos.down(),
        VerticalDirection::Down => pos.up(),
    }
}
fn block_direction_to_vertical_direction(dir: BlockDirection) -> VerticalDirection {
    match dir {
        BlockDirection::Up => VerticalDirection::Up,
        BlockDirection::Down => VerticalDirection::Down,
        _ => VerticalDirection::Up,
    }
}
fn flip_dir(dir: VerticalDirection) -> VerticalDirection {
    if dir == VerticalDirection::Up {
        return VerticalDirection::Down;
    }
    VerticalDirection::Up
}
async fn can_place_at_pos(
    block_accessor: &dyn BlockAccessor,
    position: &BlockPos,
    placing_direction: Option<BlockDirection>,
) -> bool {
    // Determine support block
    let support_block_vertical_direction = {
        if placing_direction.is_none() {
            let (block, state) = block_accessor.get_block_and_state(position).await;
            if block != &Block::POINTED_DRIPSTONE {
                //this shouldn't even happen but eh.
                return false;
            }
            let props = PointedDripstoneLikeProperties::from_state_id(state.id, block);
            flip_dir(props.vertical_direction)
        } else {
            block_direction_to_vertical_direction(placing_direction.unwrap())
        }
    };
    println!("ver_dir: {:?}", support_block_vertical_direction);
    let support_pos = match support_block_vertical_direction {
        VerticalDirection::Up => position.up(),
        VerticalDirection::Down => position.down(),
    };
    let support_block = block_accessor.get_block(&support_pos).await;
    // If placing the base kelp block, allow placement on water or on other kelp segments.
    if support_block == &Block::DRIPSTONE_BLOCK || support_block == &Block::POINTED_DRIPSTONE {
        return true;
    }
    if support_block.default_state.is_full_cube() {
        return true;
    }
    false
}
async fn modify_dripstone_thickness_to(
    world: &Arc<World>,
    pos: &BlockPos,
    new_thickness: Thickness,
) {
    let (block, support_block_state_id) = world.get_block_and_state_id(&pos).await;

    if block != &Block::POINTED_DRIPSTONE {
        //this shouldn't happen
        return;
    }
    let mut support_props =
        PointedDripstoneLikeProperties::from_state_id(support_block_state_id, block);
    if support_props.thickness == new_thickness {
        return;
    }
    support_props.thickness = new_thickness;
    world
        .set_block_state(
            &pos,
            support_props.to_state_id(&Block::POINTED_DRIPSTONE),
            BlockFlags::empty(),
        )
        .await;
}
