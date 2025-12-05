use pumpkin_data::BlockDirection;
use pumpkin_data::HorizontalFacingExt;
use pumpkin_data::block_properties::Axis;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::DoorHinge;
use pumpkin_data::block_properties::DoubleBlockHalf;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::sound::Sound;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::Taggable;
use pumpkin_data::tag::get_tag_values;
use pumpkin_data::{Block, tag};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

use crate::block::BlockBehaviour;
use crate::block::BlockFuture;
use crate::block::CanPlaceAtArgs;
use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::NormalUseArgs;
use crate::block::OnNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use crate::block::PlacedArgs;
use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use pumpkin_protocol::java::server::play::SUseItemOn;

use crate::world::World;

type DoorProperties = pumpkin_data::block_properties::OakDoorLikeProperties;

async fn toggle_door(player: &Player, world: &Arc<World>, block_pos: &BlockPos) {
    let (block, block_state) = world.get_block_and_state_id(block_pos).await;
    let mut door_props = DoorProperties::from_state_id(block_state, block);
    door_props.open = !door_props.open;

    let other_half = match door_props.half {
        DoubleBlockHalf::Upper => BlockDirection::Down,
        DoubleBlockHalf::Lower => BlockDirection::Up,
    };
    let other_pos = block_pos.offset(other_half.to_offset());

    let (other_block, other_state_id) = world.get_block_and_state_id(&other_pos).await;
    let mut other_door_props = DoorProperties::from_state_id(other_state_id, other_block);
    other_door_props.open = door_props.open;

    world
        .play_block_sound_expect(
            player,
            get_sound(block, door_props.open),
            SoundCategory::Blocks,
            *block_pos,
        )
        .await;

    world
        .set_block_state(
            block_pos,
            door_props.to_state_id(block),
            BlockFlags::NOTIFY_LISTENERS,
        )
        .await;
    world
        .set_block_state(
            &other_pos,
            other_door_props.to_state_id(other_block),
            BlockFlags::NOTIFY_LISTENERS,
        )
        .await;
}

fn can_open_door(block: &Block) -> bool {
    if block == &Block::IRON_DOOR {
        return false;
    }

    true
}

// Todo: The sounds should be from BlockSetType
fn get_sound(block: &Block, open: bool) -> Sound {
    if open {
        if block.has_tag(&tag::Block::MINECRAFT_WOODEN_DOORS) {
            Sound::BlockWoodenDoorOpen
        } else if block == &Block::IRON_DOOR {
            Sound::BlockIronDoorOpen
        } else {
            Sound::BlockCopperDoorOpen
        }
    } else if block.has_tag(&tag::Block::MINECRAFT_WOODEN_DOORS) {
        Sound::BlockWoodenDoorClose
    } else if block == &Block::IRON_DOOR {
        Sound::BlockIronDoorClose
    } else {
        Sound::BlockCopperDoorClose
    }
}

#[allow(clippy::pedantic)]
#[inline]
async fn get_hinge(
    world: &World,
    pos: &BlockPos,
    use_item: &SUseItemOn,
    facing: HorizontalFacing,
) -> DoorHinge {
    let top_pos = pos.up();
    let left_dir = facing.rotate_counter_clockwise();
    let left_pos = pos.offset(left_dir.to_block_direction().to_offset());
    let (left_block, left_state) = world.get_block_and_state(&left_pos).await;
    let top_facing = top_pos.offset(facing.to_block_direction().to_offset());
    let top_state = world.get_block_state(&top_facing).await;
    let right_dir = facing.rotate_clockwise();
    let right_pos = pos.offset(right_dir.to_block_direction().to_offset());
    let (right_block, right_state) = world.get_block_and_state(&right_pos).await;
    let top_right = top_pos.offset(facing.to_block_direction().to_offset());
    let top_right_state = world.get_block_state(&top_right).await;

    let has_left_door = world
        .get_block(&left_pos)
        .await
        .has_tag(&tag::Block::MINECRAFT_DOORS)
        && DoorProperties::from_state_id(left_state.id, left_block).half == DoubleBlockHalf::Lower;

    let has_right_door = world
        .get_block(&right_pos)
        .await
        .has_tag(&tag::Block::MINECRAFT_DOORS)
        && DoorProperties::from_state_id(right_state.id, right_block).half
            == DoubleBlockHalf::Lower;

    let score = -(left_state.is_full_cube() as i32) - (top_state.is_full_cube() as i32)
        + right_state.is_full_cube() as i32
        + top_right_state.is_full_cube() as i32;

    if (!has_left_door || has_right_door) && score <= 0 {
        if (!has_right_door || has_left_door) && score >= 0 {
            let offset = facing.to_block_direction().to_offset();
            let hit = use_item.cursor_pos;
            if (offset.x >= 0 || hit.z > 0.5)
                && (offset.x <= 0 || hit.z < 0.5)
                && (offset.z >= 0 || hit.x < 0.5)
                && (offset.z <= 0 || hit.x > 0.5)
            {
                DoorHinge::Left
            } else {
                DoorHinge::Right
            }
        } else {
            DoorHinge::Left
        }
    } else {
        DoorHinge::Right
    }
}

#[pumpkin_block_from_tag("minecraft:doors")]
pub struct DoorBlock;

impl BlockBehaviour for DoorBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let powered = block_receives_redstone_power(args.world, args.position).await
                || block_receives_redstone_power(args.world, &args.position.up()).await;

            let direction = args.player.living_entity.entity.get_horizontal_facing();
            let hinge = get_hinge(args.world, args.position, args.use_item_on, direction).await;

            let mut door_props = DoorProperties::default(args.block);
            door_props.half = DoubleBlockHalf::Lower;
            door_props.facing = direction;
            door_props.hinge = hinge;
            door_props.powered = powered;
            door_props.open = powered;

            door_props.to_state_id(args.block)
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move { can_place_at(args.block_accessor, args.position).await })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let mut door_props = DoorProperties::from_state_id(args.state_id, args.block);
            door_props.half = DoubleBlockHalf::Upper;

            args.world
                .set_block_state(
                    &args.position.offset(BlockDirection::Up.to_offset()),
                    door_props.to_state_id(args.block),
                    BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
                )
                .await;
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if !can_open_door(args.block) {
                return BlockActionResult::Pass;
            }

            toggle_door(args.player, args.world, args.position).await;

            BlockActionResult::Success
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let block_state = args.world.get_block_state(args.position).await;
            let mut door_props = DoorProperties::from_state_id(block_state.id, args.block);

            let other_half = match door_props.half {
                DoubleBlockHalf::Upper => BlockDirection::Down,
                DoubleBlockHalf::Lower => BlockDirection::Up,
            };
            let other_pos = args.position.offset(other_half.to_offset());
            let (other_block, other_state_id) = args.world.get_block_and_state_id(&other_pos).await;

            let powered = block_receives_redstone_power(args.world, args.position).await
                || block_receives_redstone_power(args.world, &other_pos).await;

            if args.block.id == other_block.id && powered != door_props.powered {
                let mut other_door_props =
                    DoorProperties::from_state_id(other_state_id, other_block);
                door_props.powered = !door_props.powered;
                other_door_props.powered = door_props.powered;

                if powered != door_props.open {
                    door_props.open = door_props.powered;
                    other_door_props.open = other_door_props.powered;

                    args.world
                        .play_block_sound(
                            get_sound(args.block, powered),
                            SoundCategory::Blocks,
                            *args.position,
                        )
                        .await;
                }

                args.world
                    .set_block_state(
                        args.position,
                        door_props.to_state_id(args.block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
                args.world
                    .set_block_state(
                        &other_pos,
                        other_door_props.to_state_id(other_block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            }
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let lv = DoorProperties::from_state_id(args.state_id, args.block).half;
            if args.direction.to_axis() != Axis::Y
                || (lv == DoubleBlockHalf::Lower) != (args.direction == BlockDirection::Up)
            {
                if lv == DoubleBlockHalf::Lower
                    && args.direction == BlockDirection::Down
                    && !can_place_at(args.world, args.position).await
                {
                    return 0;
                }
            } else if Block::from_state_id(args.neighbor_state_id).id == args.block.id
                && DoorProperties::from_state_id(args.neighbor_state_id, args.block).half != lv
            {
                let mut new_state =
                    DoorProperties::from_state_id(args.neighbor_state_id, args.block);
                new_state.half = lv;
                return new_state.to_state_id(args.block);
            } else {
                return 0;
            }
            args.state_id
        })
    }
}

async fn can_place_at(world: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    world.get_block_state(&block_pos.up()).await.replaceable()
        && world
            .get_block_state(&block_pos.down())
            .await
            .is_side_solid(BlockDirection::Up)
}
