use async_trait::async_trait;
use pumpkin_data::Block;
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
use pumpkin_data::tag::Tagable;
use pumpkin_data::tag::get_tag_values;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::pumpkin_block::CanPlaceAtArgs;
use crate::block::pumpkin_block::GetStateForNeighborUpdateArgs;
use crate::block::pumpkin_block::NormalUseArgs;
use crate::block::pumpkin_block::OnNeighborUpdateArgs;
use crate::block::pumpkin_block::OnPlaceArgs;
use crate::block::pumpkin_block::PlacedArgs;
use crate::block::pumpkin_block::UseWithItemArgs;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use pumpkin_protocol::java::server::play::SUseItemOn;

use crate::world::World;

type DoorProperties = pumpkin_data::block_properties::OakDoorLikeProperties;

async fn toggle_door(player: &Player, world: &Arc<World>, block_pos: &BlockPos) {
    let (block, block_state) = world.get_block_and_block_state(block_pos).await;
    let mut door_props = DoorProperties::from_state_id(block_state.id, block);
    door_props.open = !door_props.open;

    let other_half = match door_props.half {
        DoubleBlockHalf::Upper => BlockDirection::Down,
        DoubleBlockHalf::Lower => BlockDirection::Up,
    };
    let other_pos = block_pos.offset(other_half.to_offset());

    let (other_block, other_state_id) = world.get_block_and_block_state(&other_pos).await;
    let mut other_door_props = DoorProperties::from_state_id(other_state_id.id, other_block);
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
    if block.id == Block::IRON_DOOR.id {
        return false;
    }

    true
}

// Todo: The sounds should be from BlockSetType
fn get_sound(block: &Block, open: bool) -> Sound {
    if open {
        if block.is_tagged_with("minecraft:wooden_doors").unwrap() {
            Sound::BlockWoodenDoorOpen
        } else if block.id == Block::IRON_DOOR.id {
            Sound::BlockIronDoorOpen
        } else {
            Sound::BlockCopperDoorOpen
        }
    } else if block.is_tagged_with("minecraft:wooden_doors").unwrap() {
        Sound::BlockWoodenDoorClose
    } else if block.id == Block::IRON_DOOR.id {
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
    let (left_block, left_state) = world.get_block_and_block_state(&left_pos).await;
    let top_facing = top_pos.offset(facing.to_block_direction().to_offset());
    let top_state = world.get_block_state(&top_facing).await;
    let right_dir = facing.rotate_clockwise();
    let right_pos = pos.offset(right_dir.to_block_direction().to_offset());
    let (right_block, right_state) = world.get_block_and_block_state(&right_pos).await;
    let top_right = top_pos.offset(facing.to_block_direction().to_offset());
    let top_right_state = world.get_block_state(&top_right).await;

    let has_left_door = world
        .get_block(&left_pos)
        .await
        .is_tagged_with("minecraft:doors")
        .unwrap()
        && DoorProperties::from_state_id(left_state.id, left_block).half == DoubleBlockHalf::Lower;

    let has_right_door = world
        .get_block(&right_pos)
        .await
        .is_tagged_with("minecraft:doors")
        .unwrap()
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

pub struct DoorBlock;
impl BlockMetadata for DoorBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:doors").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for DoorBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let powered = block_receives_redstone_power(args.world, args.location).await
            || block_receives_redstone_power(args.world, &args.location.up()).await;

        let direction = args.player.living_entity.entity.get_horizontal_facing();
        let hinge = get_hinge(args.world, args.location, args.use_item_on, direction).await;

        let mut door_props = DoorProperties::default(args.block);
        door_props.half = DoubleBlockHalf::Lower;
        door_props.facing = direction;
        door_props.hinge = hinge;
        door_props.powered = powered;
        door_props.open = powered;

        door_props.to_state_id(args.block)
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.location).await
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        let mut door_props = DoorProperties::from_state_id(args.state_id, args.block);
        door_props.half = DoubleBlockHalf::Upper;

        args.world
            .set_block_state(
                &args.location.offset(BlockDirection::Up.to_offset()),
                door_props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
            )
            .await;
    }

    async fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        if !can_open_door(args.block) {
            return BlockActionResult::Continue;
        }

        toggle_door(args.player, args.world, args.location).await;

        BlockActionResult::Consume
    }

    async fn normal_use(&self, args: NormalUseArgs<'_>) {
        if can_open_door(args.block) {
            toggle_door(args.player, args.world, args.location).await;
        }
    }

    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let block_state = args.world.get_block_state(args.location).await;
        let mut door_props = DoorProperties::from_state_id(block_state.id, args.block);

        let other_half = match door_props.half {
            DoubleBlockHalf::Upper => BlockDirection::Down,
            DoubleBlockHalf::Lower => BlockDirection::Up,
        };
        let other_pos = args.location.offset(other_half.to_offset());
        let (other_block, other_state_id) = args.world.get_block_and_block_state(&other_pos).await;

        let powered = block_receives_redstone_power(args.world, args.location).await
            || block_receives_redstone_power(args.world, &other_pos).await;

        if args.block.id == other_block.id && powered != door_props.powered {
            let mut other_door_props =
                DoorProperties::from_state_id(other_state_id.id, other_block);
            door_props.powered = !door_props.powered;
            other_door_props.powered = door_props.powered;

            if powered != door_props.open {
                door_props.open = door_props.powered;
                other_door_props.open = other_door_props.powered;

                args.world
                    .play_block_sound(
                        get_sound(args.block, powered),
                        SoundCategory::Blocks,
                        *args.location,
                    )
                    .await;
            }

            args.world
                .set_block_state(
                    args.location,
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
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let lv = DoorProperties::from_state_id(args.state_id, args.block).half;
        if args.direction.to_axis() != Axis::Y
            || (lv == DoubleBlockHalf::Lower) != (args.direction == BlockDirection::Up)
        {
            if lv == DoubleBlockHalf::Lower
                && args.direction == BlockDirection::Down
                && !can_place_at(args.world, args.location).await
            {
                return 0;
            }
        } else if Block::from_state_id(args.neighbor_state_id).unwrap().id == args.block.id
            && DoorProperties::from_state_id(args.neighbor_state_id, args.block).half != lv
        {
            let mut new_state = DoorProperties::from_state_id(args.neighbor_state_id, args.block);
            new_state.half = lv;
            return new_state.to_state_id(args.block);
        } else {
            return 0;
        }
        args.state_id
    }
}

async fn can_place_at(world: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    world.get_block_state(&block_pos.up()).await.replaceable()
        && world
            .get_block_state(&block_pos.down())
            .await
            .is_side_solid(BlockDirection::Up)
}
