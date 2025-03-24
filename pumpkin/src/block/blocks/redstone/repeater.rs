use async_trait::async_trait;
use pumpkin_data::{
    block::{
        Block, BlockProperties, BlockState, Boolean, EnumVariants, HorizontalFacing, Integer1To4,
    },
    item::Item,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    block::{BlockDirection, HorizontalFacingExt},
    chunk::TickPriority,
};

use crate::{
    block::{pumpkin_block::PumpkinBlock, registry::BlockActionResult},
    entity::player::Player,
    server::Server,
    world::{BlockFlags, World},
};

use super::{diode_get_input_strength, get_weak_power, is_diode};

type RepeaterProperties = pumpkin_data::block::RepeaterLikeProperties;

#[pumpkin_block("minecraft:repeater")]
pub struct RepeaterBlock;

#[async_trait]
impl PumpkinBlock for RepeaterBlock {
    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        player_direction: &HorizontalFacing,
        _other: bool,
    ) -> u16 {
        let mut props = RepeaterProperties::default(block);
        props.facing = player_direction.opposite();
        props.locked =
            Boolean::from_bool(should_be_locked(*player_direction, world, *block_pos).await);
        props.to_state_id(block)
    }

    async fn on_neighbor_update(
        &self,
        world: &World,
        block: &Block,
        block_pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
        let state = world.get_block_state(block_pos).await.unwrap();
        let mut rep = RepeaterProperties::from_state_id(state.id, block);
        let should_be_locked = should_be_locked(rep.facing, world, *block_pos).await;
        if !rep.locked.to_bool() && should_be_locked {
            rep.locked = Boolean::True;
            world
                .set_block_state(block_pos, rep.to_state_id(block), BlockFlags::empty())
                .await;
        } else if rep.locked.to_bool() && !should_be_locked {
            rep.locked = Boolean::False;
            world
                .set_block_state(block_pos, rep.to_state_id(block), BlockFlags::empty())
                .await;
        }

        if !rep.locked.to_bool() && !world.is_block_tick_scheduled(block_pos, block).await {
            let should_be_powered = should_be_powered(rep, world, *block_pos).await;
            if should_be_powered != rep.powered.to_bool() {
                schedule_tick(rep, world, *block_pos, should_be_powered).await;
            }
        }
    }

    async fn on_scheduled_tick(&self, world: &World, block: &Block, block_pos: &BlockPos) {
        let state = world.get_block_state(block_pos).await.unwrap();
        let mut rep = RepeaterProperties::from_state_id(state.id, block);
        if rep.locked.to_bool() {
            return;
        }

        let should_be_powered = should_be_powered(rep, world, *block_pos).await;
        if rep.powered.to_bool() && !should_be_powered {
            rep.powered = Boolean::False;
            world
                .set_block_state(block_pos, rep.to_state_id(block), BlockFlags::empty())
                .await;
            on_state_change(rep, world, *block_pos).await;
        } else if !rep.powered.to_bool() {
            rep.powered = Boolean::True;
            world
                .set_block_state(block_pos, rep.to_state_id(block), BlockFlags::empty())
                .await;
            on_state_change(rep, world, *block_pos).await;
        }
    }

    async fn normal_use(
        &self,
        block: &Block,
        _player: &Player,
        location: BlockPos,
        _server: &Server,
        world: &World,
    ) {
        let state = world.get_block_state(&location).await.unwrap();
        let props = RepeaterProperties::from_state_id(state.id, block);
        on_use(props, world, location, block).await;
    }

    async fn use_with_item(
        &self,
        block: &Block,
        _player: &Player,
        location: BlockPos,
        _item: &Item,
        _server: &Server,
        world: &World,
    ) -> BlockActionResult {
        let state = world.get_block_state(&location).await.unwrap();
        let props = RepeaterProperties::from_state_id(state.id, block);
        on_use(props, world, location, block).await;
        BlockActionResult::Consume
    }

    async fn get_weak_redstone_power(
        &self,
        block: &Block,
        _world: &World,
        _block_pos: &BlockPos,
        state: &BlockState,
        direction: &BlockDirection,
    ) -> u8 {
        let repeater_props = RepeaterProperties::from_state_id(state.id, block);
        if repeater_props.facing.to_block_direction() == *direction
            && repeater_props.powered.to_bool()
        {
            return 15;
        }
        0
    }

    async fn get_strong_redstone_power(
        &self,
        block: &Block,
        _world: &World,
        _block_pos: &BlockPos,
        state: &BlockState,
        direction: &BlockDirection,
    ) -> u8 {
        let repeater_props = RepeaterProperties::from_state_id(state.id, block);
        if repeater_props.facing.to_block_direction() == *direction
            && repeater_props.powered.to_bool()
        {
            return 15;
        }
        0
    }

    async fn emits_redstone_power(
        &self,
        block: &Block,
        state: &BlockState,
        direction: &BlockDirection,
    ) -> bool {
        let repeater_props = RepeaterProperties::from_state_id(state.id, block);
        repeater_props.facing.to_block_direction() == *direction
            || repeater_props.facing.to_block_direction() == direction.opposite()
    }
}

async fn on_use(props: RepeaterProperties, world: &World, block_pos: BlockPos, block: &Block) {
    let mut props = props;
    props.delay = match props.delay {
        Integer1To4::L1 => Integer1To4::L2,
        Integer1To4::L2 => Integer1To4::L3,
        Integer1To4::L3 => Integer1To4::L4,
        Integer1To4::L4 => Integer1To4::L1,
    };
    let state = props.to_state_id(block);
    world
        .set_block_state(&block_pos, state, BlockFlags::empty())
        .await;
}

async fn should_be_locked(facing: HorizontalFacing, world: &World, pos: BlockPos) -> bool {
    let right_side = get_power_on_side(world, pos, facing.rotate()).await;
    let left_side = get_power_on_side(world, pos, facing.rotate_ccw()).await;
    std::cmp::max(right_side, left_side) > 0
}

async fn get_power_on_side(world: &World, pos: BlockPos, side: HorizontalFacing) -> u8 {
    let side_pos = pos.offset(side.to_block_direction().to_offset());
    let (side_block, side_state) = world.get_block_and_block_state(&side_pos).await.unwrap();
    if is_diode(&side_block) {
        get_weak_power(
            &side_block,
            &side_state,
            world,
            side_pos,
            side.to_block_direction(),
            false,
        )
        .await
    } else {
        0
    }
}

async fn on_state_change(rep: RepeaterProperties, world: &World, pos: BlockPos) {
    let front_pos = pos.offset(rep.facing.opposite().to_block_direction().to_offset());
    let front_block = world.get_block(&front_pos).await.unwrap();
    world.update_neighbor(&front_pos, &front_block).await;
    for direction in &BlockDirection::all() {
        let neighbor_pos = front_pos.offset(direction.to_offset());
        let block = world.get_block(&neighbor_pos).await.unwrap();
        world.update_neighbor(&neighbor_pos, &block).await;
    }
}

async fn schedule_tick(
    rep: RepeaterProperties,
    world: &World,
    pos: BlockPos,
    should_be_powered: bool,
) {
    let front_block = world
        .get_block(&pos.offset(rep.facing.opposite().to_block_direction().to_offset()))
        .await
        .unwrap();
    let priority = if is_diode(&front_block) {
        TickPriority::ExtremelyHigh
    } else if !should_be_powered {
        TickPriority::VeryHigh
    } else {
        TickPriority::High
    };
    world
        .schedule_block_tick(
            &Block::REPEATER,
            pos,
            // 1 redstone tick = 2 ticks
            (rep.delay.to_index() + 1) * 2,
            priority,
        )
        .await;
}

async fn should_be_powered(rep: RepeaterProperties, world: &World, pos: BlockPos) -> bool {
    diode_get_input_strength(world, pos, rep.facing.to_block_direction()).await > 0
}
