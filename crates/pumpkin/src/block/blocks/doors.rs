use pumpkin_data::BlockDirection;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::Axis;
use pumpkin_data::block_properties::DoorHinge;
use pumpkin_data::block_properties::DoubleBlockHalf;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::sound::Sound;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

use crate::block::BlockBehaviour;
use crate::block::BrokenArgs;
use crate::block::CanPlaceAtArgs;
use crate::block::GetStateForNeighborUpdateArgs;
use crate::block::NormalUseArgs;
use crate::block::OnNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use crate::block::OnStateReplacedArgs;
use crate::block::PlacedArgs;
use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use pumpkin_protocol::java::server::play::SUseItemOn;

use crate::world::World;
use pumpkin_util::GameMode;

type DoorProperties = pumpkin_data::block_properties::OakDoorLikeProperties;

fn toggle_door(player: &Player, world: &Arc<World>, block_pos: &BlockPos) {
    let (block, block_state) = world.get_block_and_state_id(block_pos);
    let mut door_props = DoorProperties::from_state_id(block_state);
    door_props.open = !door_props.open;

    let other_half = match door_props.half {
        DoubleBlockHalf::Upper => BlockDirection::Down,
        DoubleBlockHalf::Lower => BlockDirection::Up,
    };
    let other_pos = block_pos.offset(other_half.to_offset());

    let (other_block, other_state_id) = world.get_block_and_state_id(&other_pos);
    let mut other_door_props = DoorProperties::from_state_id(other_state_id);
    other_door_props.open = door_props.open;

    world.play_block_sound_expect(
        player,
        get_sound(block, door_props.open),
        SoundCategory::Blocks,
        *block_pos,
    );

    world.set_block_state(
        block_pos,
        door_props.to_state_id(block),
        BlockFlags::NOTIFY_LISTENERS,
    );
    world.set_block_state(
        &other_pos,
        other_door_props.to_state_id(other_block),
        BlockFlags::NOTIFY_LISTENERS,
    );
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

#[expect(clippy::pedantic)]
#[inline]
fn get_hinge(
    world: &World,
    pos: &BlockPos,
    use_item: &SUseItemOn,
    facing: HorizontalFacing,
) -> DoorHinge {
    let top_pos = pos.up();
    let left_dir = facing.rotate_counter_clockwise();
    let left_pos = pos.offset(left_dir.to_offset());
    let (_left_block, left_state) = world.get_block_and_state(&left_pos);
    let top_facing = top_pos.offset(facing.to_offset());
    let top_state = world.get_block_state(&top_facing);
    let right_dir = facing.rotate_clockwise();
    let right_pos = pos.offset(right_dir.to_offset());
    let (_right_block, right_state) = world.get_block_and_state(&right_pos);
    let top_right = top_pos.offset(facing.to_offset());
    let top_right_state = world.get_block_state(&top_right);

    let has_left_door = world
        .get_block(&left_pos)
        .has_tag(&tag::Block::MINECRAFT_DOORS)
        && DoorProperties::from_state_id(left_state.id).half == DoubleBlockHalf::Lower;

    let has_right_door = world
        .get_block(&right_pos)
        .has_tag(&tag::Block::MINECRAFT_DOORS)
        && DoorProperties::from_state_id(right_state.id).half == DoubleBlockHalf::Lower;

    let score = -(left_state.is_full_cube() as i32) - (top_state.is_full_cube() as i32)
        + right_state.is_full_cube() as i32
        + top_right_state.is_full_cube() as i32;

    if (!has_left_door || has_right_door) && score <= 0 {
        if (!has_right_door || has_left_door) && score >= 0 {
            let offset = facing.to_offset();
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

impl DoorBlock {
    #[must_use]
    pub fn is_wooden_door(world: &World, block_pos: &BlockPos) -> bool {
        let block = world.get_block(block_pos);
        block.has_tag(&tag::Block::MINECRAFT_WOODEN_DOORS)
    }

    #[must_use]
    pub fn is_open(world: &World, block_pos: &BlockPos) -> bool {
        let (block, block_state) = world.get_block_and_state_id(block_pos);
        if !block.has_tag(&tag::Block::MINECRAFT_DOORS) {
            return false;
        }
        let door_props = DoorProperties::from_state_id(block_state);
        door_props.open
    }

    pub fn set_open(world: &Arc<World>, block_pos: &BlockPos, open: bool) {
        let (block, block_state) = world.get_block_and_state_id(block_pos);
        if !block.has_tag(&tag::Block::MINECRAFT_DOORS) {
            return;
        }
        let mut door_props = DoorProperties::from_state_id(block_state);
        if door_props.open == open {
            return;
        }
        door_props.open = open;

        let other_half = match door_props.half {
            DoubleBlockHalf::Upper => BlockDirection::Down,
            DoubleBlockHalf::Lower => BlockDirection::Up,
        };
        let other_pos = block_pos.offset(other_half.to_offset());

        let (other_block, other_state_id) = world.get_block_and_state_id(&other_pos);

        world.play_block_sound(get_sound(block, open), SoundCategory::Blocks, *block_pos);

        world.set_block_state(
            block_pos,
            door_props.to_state_id(block),
            BlockFlags::NOTIFY_LISTENERS,
        );

        if other_block.id == block.id {
            let mut other_door_props = DoorProperties::from_state_id(other_state_id);
            other_door_props.open = open;
            world.set_block_state(
                &other_pos,
                other_door_props.to_state_id(other_block),
                BlockFlags::NOTIFY_LISTENERS,
            );
        }
    }
}

impl BlockBehaviour for DoorBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut door_props = DoorProperties::default(args.block);
        let facing = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();
        door_props.facing = facing;
        door_props.half = DoubleBlockHalf::Lower;
        door_props.hinge = get_hinge(args.world, args.position, args.use_item_on, facing);
        door_props.open = false;
        door_props.powered = false;
        door_props.to_state_id(args.block)
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        has_support(args.block_accessor, args.position)
            && args
                .block_accessor
                .get_block_state(&args.position.up())
                .replaceable()
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        {
            let mut door_props = DoorProperties::from_state_id(args.state_id);
            door_props.half = DoubleBlockHalf::Upper;

            args.world.set_block_state(
                &args.position.offset(BlockDirection::Up.to_offset()),
                door_props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK,
            );
        }
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        {
            if !can_open_door(args.block) {
                return BlockActionResult::Pass;
            }

            toggle_door(args.player, args.world, args.position);

            BlockActionResult::Success
        }
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        let door_props = DoorProperties::from_state_id(args.state.id);
        let other_half_pos = match door_props.half {
            DoubleBlockHalf::Upper => args.position.down(),
            DoubleBlockHalf::Lower => args.position.up(),
        };

        let (other_block, other_state) = args.world.get_block_and_state(&other_half_pos);
        if other_block.id != args.block.id {
            args.world.update_neighbors(&other_half_pos, None);
            return; // Neighbor is already gone or is a different block
        }

        let other_props = DoorProperties::from_state_id(other_state.id);
        if other_props.half == door_props.half {
            return;
        }

        let is_creative = args.player.gamemode.load() == GameMode::Creative;
        let flags = if door_props.half == DoubleBlockHalf::Upper && !is_creative {
            BlockFlags::NOTIFY_ALL
        } else {
            BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL
        };

        args.world
            .break_block(&other_half_pos, Some(args.player), flags);
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let block_state = args.world.get_block_state(args.position);
        let mut door_props = DoorProperties::from_state_id(block_state.id);
        let other_half = match door_props.half {
            DoubleBlockHalf::Upper => BlockDirection::Down,
            DoubleBlockHalf::Lower => BlockDirection::Up,
        };
        let other_pos = args.position.offset(other_half.to_offset());
        let (other_block, other_state_id) = args.world.get_block_and_state_id(&other_pos);

        if other_block.id != args.block.id {
            return;
        }

        let powered = block_receives_redstone_power(args.world, args.position)
            || block_receives_redstone_power(args.world, &other_pos);

        if door_props.powered != powered {
            let sound_half = if door_props.open {
                DoubleBlockHalf::Lower
            } else {
                DoubleBlockHalf::Upper
            };

            let mut other_door_props = DoorProperties::from_state_id(other_state_id);

            door_props.powered = powered;
            other_door_props.powered = powered;

            if door_props.open != powered {
                door_props.open = powered;
                other_door_props.open = powered;
            }

            if door_props.half == sound_half {
                args.world.play_block_sound(
                    get_sound(args.block, powered),
                    SoundCategory::Blocks,
                    *args.position,
                );
            }

            args.world.set_block_state(
                args.position,
                door_props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
            args.world.set_block_state(
                &other_pos,
                other_door_props.to_state_id(other_block),
                BlockFlags::NOTIFY_ALL,
            );
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let lv = DoorProperties::from_state_id(args.state_id).half;
        if args.direction.to_axis() != Axis::Y
            || (lv == DoubleBlockHalf::Lower) != (args.direction == BlockDirection::Up)
        {
            if lv == DoubleBlockHalf::Lower
                && args.direction == BlockDirection::Down
                && !has_support(args.world, args.position)
            {
                return BlockStateId::AIR;
            }
        } else if Block::from_state_id(args.neighbor_state_id).id == args.block.id
            && DoorProperties::from_state_id(args.neighbor_state_id).half != lv
        {
            let mut new_state = DoorProperties::from_state_id(args.neighbor_state_id);
            new_state.half = lv;
            return new_state.to_state_id(args.block);
        } else {
            return BlockStateId::AIR;
        }
        args.state_id
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if args.moved {
            return;
        }

        let door_props = DoorProperties::from_state_id(args.old_state_id);
        let other_half_pos = match door_props.half {
            DoubleBlockHalf::Upper => args.position.down(),
            DoubleBlockHalf::Lower => args.position.up(),
        };

        let (other_block, other_state) = args.world.get_block_and_state(&other_half_pos);
        if other_block.id == args.block.id {
            let other_props = DoorProperties::from_state_id(other_state.id);
            if other_props.half != door_props.half {
                args.world.break_block(
                    &other_half_pos,
                    None,
                    BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }
}

fn has_support(world: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    world
        .get_block_state(&block_pos.down())
        .is_side_solid(BlockDirection::Up)
}
