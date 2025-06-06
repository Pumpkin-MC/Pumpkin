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

use crate::block::BlockIsReplacing;
use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use pumpkin_data::item::Item;
use pumpkin_protocol::server::play::SUseItemOn;

use crate::server::Server;
use crate::world::World;

type DoorProperties = pumpkin_data::block_properties::OakDoorLikeProperties;

async fn toggle_door(world: &Arc<World>, block_pos: &BlockPos) {
    let (block, block_state) = world.get_block_and_block_state(block_pos).await;
    let mut door_props = DoorProperties::from_state_id(block_state.id, &block);
    door_props.open = !door_props.open;

    let other_half = match door_props.half {
        DoubleBlockHalf::Upper => BlockDirection::Down,
        DoubleBlockHalf::Lower => BlockDirection::Up,
    };
    let other_pos = block_pos.offset(other_half.to_offset());

    let (other_block, other_state_id) = world.get_block_and_block_state(&other_pos).await;
    let mut other_door_props = DoorProperties::from_state_id(other_state_id.id, &other_block);
    other_door_props.open = door_props.open;

    world
        .set_block_state(
            block_pos,
            door_props.to_state_id(&block),
            BlockFlags::NOTIFY_LISTENERS,
        )
        .await;
    world
        .set_block_state(
            &other_pos,
            other_door_props.to_state_id(&other_block),
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
    block: &Block,
    pos: &BlockPos,
    use_item: &SUseItemOn,
    facing: HorizontalFacing,
) -> DoorHinge {
    let top_pos = pos.up();
    let left_dir = facing.rotate_counter_clockwise();
    let left_pos = pos.offset(left_dir.to_block_direction().to_offset());
    let left_state = world.get_block_state(&left_pos).await;
    let top_facing = top_pos.offset(facing.to_block_direction().to_offset());
    let top_state = world.get_block_state(&top_facing).await;
    let right_dir = facing.rotate_clockwise();
    let right_pos = pos.offset(right_dir.to_block_direction().to_offset());
    let right_state = world.get_block_state(&right_pos).await;
    let top_right = top_pos.offset(facing.to_block_direction().to_offset());
    let top_right_state = world.get_block_state(&top_right).await;

    let has_left_door = world
        .get_block(&left_pos)
        .await
        .is_tagged_with("minecraft:doors")
        .unwrap()
        && DoorProperties::from_state_id(left_state.id, block).half == DoubleBlockHalf::Lower;

    let has_right_door = world
        .get_block(&right_pos)
        .await
        .is_tagged_with("minecraft:doors")
        .unwrap()
        && DoorProperties::from_state_id(right_state.id, block).half == DoubleBlockHalf::Lower;

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
    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        player: &Player,
        block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _replacing: BlockIsReplacing,
        use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let powered = block_receives_redstone_power(world, block_pos).await
            || block_receives_redstone_power(world, &block_pos.up()).await;

        let direction = player.living_entity.entity.get_horizontal_facing();
        let hinge = get_hinge(world, block, block_pos, use_item_on, direction).await;

        let mut door_props = DoorProperties::default(block);
        door_props.half = DoubleBlockHalf::Lower;
        door_props.facing = direction;
        door_props.hinge = hinge;
        door_props.powered = powered;
        door_props.open = powered;

        door_props.to_state_id(block)
    }

    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        world: Option<&World>,
        _block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        can_place_at(world.unwrap(), block_pos).await
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        block_pos: &BlockPos,
        _old_state_id: BlockStateId,
        _notify: bool,
    ) {
        let mut door_props = DoorProperties::from_state_id(state_id, block);
        door_props.half = DoubleBlockHalf::Upper;

        world
            .set_block_state(
                &block_pos.offset(BlockDirection::Up.to_offset()),
                door_props.to_state_id(block),
                BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
            )
            .await;
    }

    async fn use_with_item(
        &self,
        block: &Block,
        _player: &Player,
        location: BlockPos,
        _item: &Item,
        _server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        if !can_open_door(block) {
            return BlockActionResult::Continue;
        }

        toggle_door(world, &location).await;

        BlockActionResult::Consume
    }

    async fn normal_use(
        &self,
        block: &Block,
        _player: &Player,
        location: BlockPos,
        _server: &Server,
        world: &Arc<World>,
    ) {
        if can_open_door(block) {
            toggle_door(world, &location).await;
        }
    }

    async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
        let block_state = world.get_block_state(pos).await;
        let mut door_props = DoorProperties::from_state_id(block_state.id, block);

        let other_half = match door_props.half {
            DoubleBlockHalf::Upper => BlockDirection::Down,
            DoubleBlockHalf::Lower => BlockDirection::Up,
        };
        let other_pos = pos.offset(other_half.to_offset());
        let (other_block, other_state_id) = world.get_block_and_block_state(&other_pos).await;

        let powered = block_receives_redstone_power(world, pos).await
            || block_receives_redstone_power(world, &other_pos).await;

        if block.id == other_block.id && powered != door_props.powered {
            let mut other_door_props =
                DoorProperties::from_state_id(other_state_id.id, &other_block);
            door_props.powered = !door_props.powered;
            other_door_props.powered = door_props.powered;

            if powered != door_props.open {
                door_props.open = door_props.powered;
                other_door_props.open = other_door_props.powered;

                world
                    .play_block_sound(get_sound(block, powered), SoundCategory::Blocks, *pos)
                    .await;
            }

            world
                .set_block_state(
                    pos,
                    door_props.to_state_id(block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
            world
                .set_block_state(
                    &other_pos,
                    other_door_props.to_state_id(&other_block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
        }
    }

    async fn get_state_for_neighbor_update(
        &self,
        world: &World,
        block: &Block,
        state: u16,
        block_pos: &BlockPos,
        direction: BlockDirection,
        _neighbor_pos: &BlockPos,
        neighbor_state: u16,
    ) -> u16 {
        let lv = DoorProperties::from_state_id(state, block).half;
        if direction.to_axis() != Axis::Y
            || (lv == DoubleBlockHalf::Lower) != (direction == BlockDirection::Up)
        {
            if lv == DoubleBlockHalf::Lower
                && direction == BlockDirection::Down
                && !can_place_at(world, block_pos).await
            {
                return 0;
            }
        } else if Block::from_state_id(neighbor_state).unwrap().id == block.id
            && DoorProperties::from_state_id(neighbor_state, block).half != lv
        {
            let mut new_state = DoorProperties::from_state_id(neighbor_state, block);
            new_state.half = lv;
            return new_state.to_state_id(block);
        } else {
            return 0;
        }
        state
    }
}

async fn can_place_at(world: &World, block_pos: &BlockPos) -> bool {
    world.get_block_state(&block_pos.up()).await.replaceable()
        && world
            .get_block_state(&block_pos.down())
            .await
            .is_side_solid(BlockDirection::Up)
}
