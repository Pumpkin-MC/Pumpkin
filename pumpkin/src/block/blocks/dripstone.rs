use std::sync::Arc;

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockMetadata, BrokenArgs, CanPlaceAtArgs,
        GetStateForNeighborUpdateArgs, OnPlaceArgs, PlacedArgs,
    },
    entity::player::Player,
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
        println!("=====================");
        Box::pin(async move {
            can_place_at_pos(
                args.block_accessor,
                args.position,
                args.direction,
                args.player,
            )
            .await
        })
    }
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut dripstone_props = PointedDripstoneLikeProperties::default(args.block);
            dripstone_props.waterlogged = args.replacing.water_source();
            let Some(support_block_ver_dir) = get_support_block_vertical_direction(
                args.world,
                args.position,
                Some(args.direction),
                Some(args.player),
            )
            .await
            else {
                println!("something went terribly wrong!");
                return Block::LAVA.id;
            };

            dripstone_props.vertical_direction = flip_dir(support_block_ver_dir);
            dripstone_props.to_state_id(&Block::POINTED_DRIPSTONE)
        })
    }
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let (len, vertical_dir) = get_stalagmite_or_stalactice_len_and_dir_from_tip_pos(
                args.world,
                args.position,
                args.state_id,
            )
            .await;
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

            let (len, vertical_dir) = get_stalagmite_or_stalactice_len_and_dir_from_tip_pos(
                args.world,
                &new_tip_pos,
                args.state.id,
            )
            .await;
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
            if !can_place_at_pos(args.world, args.position, None, None).await {
                return Block::AIR.default_state.id;
            }
            let mut dripstone_props =
                PointedDripstoneLikeProperties::from_state_id(args.state_id, args.block);
            if dripstone_props.thickness != Thickness::TipMerge {
                return args.state_id;
            }
            match dripstone_props.vertical_direction {
                VerticalDirection::Up => {
                    let block_above = args.world.get_block(&args.position.up()).await;
                    if block_above != &Block::POINTED_DRIPSTONE {
                        dripstone_props.thickness = Thickness::Tip;
                        return dripstone_props.to_state_id(args.block);
                    }
                }
                VerticalDirection::Down => {
                    let block_below = args.world.get_block(&args.position.down()).await;
                    if block_below != &Block::POINTED_DRIPSTONE {
                        dripstone_props.thickness = Thickness::Tip;
                        return dripstone_props.to_state_id(args.block);
                    }
                }
            }
            args.state_id
        })
    }
}
async fn update_stalagmite<'a>(world: &'a Arc<World>, stalagmite_len: u8, tip_pos: &BlockPos) {
    let block_above = world.get_block(&tip_pos.up()).await;
    if block_above == &Block::POINTED_DRIPSTONE {
        modify_dripstone_thickness_to(world, tip_pos, Thickness::TipMerge).await;
        modify_dripstone_thickness_to(world, &tip_pos.up(), Thickness::TipMerge).await;
    } else {
        modify_dripstone_thickness_to(world, tip_pos, Thickness::Tip).await;
    }
    match stalagmite_len {
        2 => {
            modify_dripstone_thickness_to(world, &tip_pos.down_height(1), Thickness::Frustum).await
        }
        3 => {
            modify_dripstone_thickness_to(world, &tip_pos.down_height(1), Thickness::Frustum).await;
            modify_dripstone_thickness_to(world, &tip_pos.down_height(2), Thickness::Base).await;
        }
        4 => {
            modify_dripstone_thickness_to(world, &tip_pos.down_height(1), Thickness::Frustum).await;
            modify_dripstone_thickness_to(world, &tip_pos.down_height(2), Thickness::Middle).await;
            modify_dripstone_thickness_to(world, &tip_pos.down_height(3), Thickness::Base).await;
        }
        5 => {
            modify_dripstone_thickness_to(world, &tip_pos.down_height(1), Thickness::Frustum).await;
            modify_dripstone_thickness_to(world, &tip_pos.down_height(2), Thickness::Middle).await;
            modify_dripstone_thickness_to(world, &tip_pos.down_height(3), Thickness::Middle).await;
        }
        _ => {}
    };
}

async fn update_stalactite<'a>(world: &'a Arc<World>, stalagmite_len: u8, tip_pos: &BlockPos) {
    let block_below = world.get_block(&tip_pos.down()).await;
    if block_below == &Block::POINTED_DRIPSTONE {
        modify_dripstone_thickness_to(world, tip_pos, Thickness::TipMerge).await;
        modify_dripstone_thickness_to(world, &tip_pos.down(), Thickness::TipMerge).await;
    } else {
        modify_dripstone_thickness_to(world, tip_pos, Thickness::Tip).await;
    }
    match stalagmite_len {
        2 => {
            modify_dripstone_thickness_to(world, &tip_pos.up_height(1), Thickness::Frustum).await;
        }
        3 => {
            modify_dripstone_thickness_to(world, &tip_pos.up_height(1), Thickness::Frustum).await;
            modify_dripstone_thickness_to(world, &tip_pos.up_height(2), Thickness::Base).await;
        }
        4 => {
            modify_dripstone_thickness_to(world, &tip_pos.up_height(1), Thickness::Frustum).await;
            modify_dripstone_thickness_to(world, &tip_pos.up_height(2), Thickness::Middle).await;
            modify_dripstone_thickness_to(world, &tip_pos.up_height(3), Thickness::Base).await;
        }
        5 => {
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
    block_state_id: BlockStateId,
) -> (u8, VerticalDirection) {
    let props =
        PointedDripstoneLikeProperties::from_state_id(block_state_id, &Block::POINTED_DRIPSTONE);

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
fn block_direction_to_vertical_direction(dir: BlockDirection) -> Option<VerticalDirection> {
    match dir {
        BlockDirection::Up => Some(VerticalDirection::Up),
        BlockDirection::Down => Some(VerticalDirection::Down),
        _ => None,
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
    player_option: Option<&Player>,
) -> bool {
    // Determine support block
    let Some(support_block_vertical_direction) = get_support_block_vertical_direction(
        block_accessor,
        position,
        placing_direction,
        player_option,
    )
    .await
    else {
        return false;
    };
    let support_pos = match support_block_vertical_direction {
        VerticalDirection::Up => position.up(),
        VerticalDirection::Down => position.down(),
    };
    let support_block = block_accessor.get_block(&support_pos).await;
    println!("support_block: {}", support_block.name);
    println!(
        "place_at coords: {:?} ver_dir {:?}",
        position, support_block_vertical_direction
    );
    if can_support_dripstone(support_block) {
        return true;
    }
    println!("returning false 2");
    false
}
async fn get_support_block_vertical_direction(
    block_accessor: &dyn BlockAccessor,
    position: &BlockPos,
    placing_direction: Option<BlockDirection>,
    player_option: Option<&Player>,
) -> Option<VerticalDirection> {
    if placing_direction.is_none() {
        let (block, state) = block_accessor.get_block_and_state(position).await;
        if block != &Block::POINTED_DRIPSTONE {
            println!("returning false 1");
            return None;
        }
        let props = PointedDripstoneLikeProperties::from_state_id(state.id, block);
        Some(flip_dir(props.vertical_direction))
    } else {
        println!("placing_dir: {:?}", placing_direction.unwrap());
        match block_direction_to_vertical_direction(placing_direction.unwrap()) {
            Some(ver_dir) => match ver_dir {
                VerticalDirection::Up => {
                    let block_above = block_accessor.get_block(&position.up()).await;
                    let block_below = block_accessor.get_block(&position.down()).await;
                    if can_support_dripstone(&block_above) {
                        return Some(VerticalDirection::Up);
                    } else if can_support_dripstone(&block_below) {
                        return Some(VerticalDirection::Down);
                    }
                    return None;
                }
                VerticalDirection::Down => {
                    let block_above = block_accessor.get_block(&position.up()).await;
                    let block_below = block_accessor.get_block(&position.down()).await;
                    if can_support_dripstone(&block_below) {
                        return Some(VerticalDirection::Down);
                    } else if can_support_dripstone(&block_above) {
                        return Some(VerticalDirection::Up);
                    }
                    return None;
                }
            },
            None => match player_option {
                Some(player) => {
                    let (_, pitch) = player.rotation();
                    println!("pitch: {pitch}");
                    let (can_place_above, can_place_below) = {
                        let block_above = block_accessor.get_block(&position.up()).await;
                        let block_below = block_accessor.get_block(&position.down()).await;
                        (
                            can_support_dripstone(&block_above),
                            can_support_dripstone(&block_below),
                        )
                    };
                    println!("pitch {pitch}, {can_place_above} {can_place_below}");
                    match (can_place_above, can_place_below) {
                        (true, true) => {
                            if pitch > 0.0 {
                                Some(VerticalDirection::Down)
                            } else {
                                Some(VerticalDirection::Up)
                            }
                        }
                        (false, false) => {
                            return None;
                        }
                        (true, false) => Some(VerticalDirection::Up),
                        (false, true) => Some(VerticalDirection::Down),
                    }
                }
                None => Some(VerticalDirection::Up),
            },
        }
    }
}
fn can_support_dripstone(support_block: &Block) -> bool {
    if support_block == &Block::POINTED_DRIPSTONE {
        return true;
    }
    if support_block.default_state.is_full_cube() && support_block.default_state.is_solid_block() {
        return true;
    }
    false
}
async fn modify_dripstone_thickness_to(
    world: &Arc<World>,
    pos: &BlockPos,
    new_thickness: Thickness,
) {
    let (block, support_block_state_id) = world.get_block_and_state_id(pos).await;

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
