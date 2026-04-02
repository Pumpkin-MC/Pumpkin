use std::collections::HashSet;

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockIsReplacing, CanPlaceAtArgs, CanUpdateAtArgs,
        GetStateForNeighborUpdateArgs, OnPlaceArgs, UseWithItemArgs, registry::BlockActionResult,
    },
    entity::player::Player,
};
use pumpkin_data::{
    Block, BlockDirection,
    block_properties::{BlockProperties, VineLikeProperties},
    item::Item,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    world::{BlockAccessor, BlockFlags},
};

#[pumpkin_block("minecraft:vine")]
pub struct VineBlock;

impl BlockBehaviour for VineBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if let BlockIsReplacing::Itself(state_id) = args.replacing {
                let (Some(direction), _) = get_accurate_direction(
                    args.world,
                    args.position,
                    Some(args.player),
                    args.direction,
                    true,
                )
                .await
                else {
                    return Block::AIR.default_state.id;
                };
                let mut props = VineLikeProperties::from_state_id(state_id, args.block);
                vine_direction_mapper(direction, &mut props);
                return props.to_state_id(args.block);
            }
            let (Some(direction), _) = get_accurate_direction(
                args.world,
                args.position,
                Some(args.player),
                args.direction,
                false,
            )
            .await
            else {
                return Block::AIR.default_state.id;
            };
            let mut props = VineLikeProperties::default(args.block);
            vine_direction_mapper(direction, &mut props);
            props.to_state_id(args.block)
        })
    }
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            can_place_vine_at(
                args.block_accessor,
                args.position,
                args.direction,
                args.player,
                false,
            )
            .await
        })
    }
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let old_props = VineLikeProperties::from_state_id(args.state_id, args.block);
            let old_directions = get_vine_block_directions(old_props);
            let mut new_directions = old_directions.clone();
            for old_dir in old_directions {
                let support_block = args
                    .world
                    .get_block(&args.position.offset(old_dir.to_offset()))
                    .await;
                if !supports_vine(support_block)
                    && !is_top_block_full_vine(args.world, args.position).await
                {
                    new_directions.remove(&old_dir);
                }
            }
            if new_directions.is_empty() {
                return Block::AIR.id;
            }
            let mut new_props = VineLikeProperties::default(args.block);

            for new_dir in new_directions {
                vine_direction_mapper(new_dir, &mut new_props);
            }

            new_props.to_state_id(args.block)
        })
    }
    fn can_update_at<'a>(&'a self, args: CanUpdateAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            get_accurate_direction(
                args.world,
                args.position,
                Some(args.player),
                args.direction,
                true,
            )
            .await
            .0
            .is_some()
        })
    }
    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position).await;
            let mut props = VineLikeProperties::from_state_id(state.id, args.block);

            let item_lock = args.item_stack.lock().await;
            let item = item_lock.item;
            drop(item_lock);

            if item.id != Item::VINE.id {
                return BlockActionResult::Pass;
            }
            let (Some(accurate_dir), _) = get_accurate_direction(
                args.world.as_ref(),
                args.position,
                Some(args.player),
                BlockDirection::Down,
                true,
            )
            .await
            else {
                return BlockActionResult::Fail;
            };
            vine_direction_mapper(accurate_dir, &mut props);

            args.world
                .set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
            BlockActionResult::Consume
        })
    }
}
pub fn get_nearest_looking_directions(
    player: &Player,
    replace_clicked: bool,
    clicked_face: BlockDirection,
) -> [BlockDirection; 6] {
    let mut directions = ordered_by_nearest(player);

    if !replace_clicked {
        let target = clicked_face.opposite();

        let mut index = 0;

        while index < directions.len() && directions[index] != target {
            index += 1;
        }

        if index > 0 {
            directions.copy_within(0..index, 1);
            directions[0] = target;
        }
    }
    directions
}
pub fn ordered_by_nearest(player: &Player) -> [BlockDirection; 6] {
    let (yaw_degrees, pitch_degrees) = player.rotation();
    let yaw = -yaw_degrees.to_radians();
    let pitch = pitch_degrees.to_radians();
    let pitch_sin = pitch.sin();
    let pitch_cos = pitch.cos();
    let yaw_sin = yaw.sin();
    let yaw_cos = yaw.cos();

    let x_pos = yaw_sin > 0.0;
    let y_pos = pitch_sin < 0.0;
    let z_pos = yaw_cos > 0.0;

    let x_yaw = if x_pos { yaw_sin } else { -yaw_sin };
    let y_mag = if y_pos { -pitch_sin } else { pitch_sin };
    let z_yaw = if z_pos { yaw_cos } else { -yaw_cos };

    let x_mag = x_yaw * pitch_cos;
    let z_mag = z_yaw * pitch_cos;

    let axis_x = if x_pos {
        BlockDirection::East
    } else {
        BlockDirection::West
    };
    let axis_y = if y_pos {
        BlockDirection::Up
    } else {
        BlockDirection::Down
    };
    let axis_z = if z_pos {
        BlockDirection::South
    } else {
        BlockDirection::North
    };

    if x_yaw > z_yaw {
        if y_mag > x_mag {
            make_direction_array(axis_y, axis_x, axis_z)
        } else if z_mag > y_mag {
            make_direction_array(axis_x, axis_z, axis_y)
        } else {
            make_direction_array(axis_x, axis_y, axis_z)
        }
    } else if y_mag > z_mag {
        make_direction_array(axis_y, axis_z, axis_x)
    } else if x_mag > y_mag {
        make_direction_array(axis_z, axis_x, axis_y)
    } else {
        make_direction_array(axis_z, axis_y, axis_x)
    }
}
const fn make_direction_array(
    axis1: BlockDirection,
    axis2: BlockDirection,
    axis3: BlockDirection,
) -> [BlockDirection; 6] {
    [
        axis1,
        axis2,
        axis3,
        axis3.opposite(),
        axis2.opposite(),
        axis1.opposite(),
    ]
}
async fn can_place_vine_at(
    block_accessor: &dyn BlockAccessor,
    block_pos: &BlockPos,
    click_direction_wrapper: Option<BlockDirection>,
    player_wrapper: Option<&Player>,
    replacing: bool,
) -> bool {
    let Some(click_direction) = click_direction_wrapper else {
        return false;
    };
    let (Some(direction), _) = get_accurate_direction(
        block_accessor,
        block_pos,
        player_wrapper,
        click_direction,
        replacing,
    )
    .await
    else {
        return false;
    };
    let support_pos = block_pos.offset(direction.to_offset());
    let (support_block, _support_block_state) =
        block_accessor.get_block_and_state(&support_pos).await;
    if !supports_vine(support_block) && !is_top_block_full_vine(block_accessor, block_pos).await {
        return false;
    }
    true
}
const fn supports_vine(support_block: &Block) -> bool {
    if support_block.default_state.is_full_cube() {
        return true;
    }
    false
}
//returns (accurate direction, boolean)
// true if this direction is for hanging leaf
// false if it is not
async fn get_accurate_direction(
    block_accessor: &dyn BlockAccessor,
    block_pos: &BlockPos,
    player_wrapper: Option<&Player>,
    click_direction: BlockDirection,
    replacing: bool,
) -> (Option<BlockDirection>, bool) {
    let clicked_block = block_accessor
        .get_block(&block_pos.offset(click_direction.to_offset()))
        .await;
    if !replacing && clicked_block == &Block::VINE && click_direction != BlockDirection::Up {
        return (None, false);
    }

    if click_direction != BlockDirection::Down && supports_vine(clicked_block) {
        return (Some(click_direction), false);
    }
    let (replacing_block, replacing_block_state) =
        block_accessor.get_block_and_state(block_pos).await;
    let already_active_directions = if replacing_block == &Block::VINE {
        let props = VineLikeProperties::from_state_id(replacing_block_state.id, replacing_block);
        get_vine_block_directions(props)
    } else {
        HashSet::new()
    };
    if let Some(player) = player_wrapper {
        let mut up = false;
        for dir in get_nearest_looking_directions(player, replacing, click_direction) {
            if dir != BlockDirection::Down && !already_active_directions.contains(&dir) {
                let support_pos = block_pos.offset(dir.to_offset());
                let (support_block, _support_block_state) =
                    block_accessor.get_block_and_state(&support_pos).await;
                if !supports_vine(support_block) {
                    //handler for hanging vine
                    if is_top_block_full_vine(block_accessor, block_pos).await {
                        if dir == BlockDirection::Up {
                            continue;
                        }
                        return (Some(dir), true);
                    }
                    continue;
                }
                if dir == BlockDirection::Up && !replacing {
                    up = true;
                    continue;
                }

                return (Some(dir), false);
            }
        }
        if up {
            return (Some(BlockDirection::Up), false);
        }
    }
    (None, false)
}
async fn is_top_block_full_vine(block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    let (top_block, top_block_state) = block_accessor.get_block_and_state(&block_pos.up()).await;
    if top_block != &Block::VINE {
        return false;
    }
    let props = VineLikeProperties::from_state_id(top_block_state.id, top_block);
    props.up && props.west && props.east && props.north && props.south
}
fn get_vine_block_directions(props: VineLikeProperties) -> HashSet<BlockDirection> {
    let mut set = HashSet::new();
    if props.north {
        set.insert(BlockDirection::North);
    }
    if props.south {
        set.insert(BlockDirection::South);
    }
    if props.east {
        set.insert(BlockDirection::East);
    }
    if props.west {
        set.insert(BlockDirection::West);
    }
    if props.up {
        set.insert(BlockDirection::Up);
    }
    set
}
const fn vine_direction_mapper(direction: BlockDirection, props: &mut VineLikeProperties) {
    match direction {
        BlockDirection::Down => (),
        BlockDirection::Up => props.up = true,
        BlockDirection::North => props.north = true,
        BlockDirection::South => props.south = true,
        BlockDirection::West => props.west = true,
        BlockDirection::East => props.east = true,
    }
}
