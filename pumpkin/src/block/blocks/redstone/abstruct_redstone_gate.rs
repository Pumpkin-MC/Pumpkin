use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection, BlockState, HorizontalFacingExt,
    block_properties::{
        BlockProperties, ComparatorLikeProperties, EnumVariants, HorizontalFacing,
        RedstoneWireLikeProperties, RepeaterLikeProperties, get_state_by_state_id,
    },
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    chunk::TickPriority,
    world::{BlockAccessor, BlockFlags},
};

use crate::{
    block::pumpkin_block::{
        GetRedstonePowerArgs, OnNeighborUpdateArgs, OnStateReplacedArgs, PlayerPlacedArgs,
    },
    entity::player::Player,
    world::World,
};

use super::{get_redstone_power, is_diode};

pub trait RedstoneGateBlockProperties {
    fn is_powered(&self) -> bool;
    fn get_facing(&self) -> HorizontalFacing;
    fn set_facing(&mut self, facing: HorizontalFacing);
}

#[async_trait]
pub trait RedstoneGateBlock<T: Send + BlockProperties + RedstoneGateBlockProperties> {
    async fn can_place_at(&self, world: &dyn BlockAccessor, pos: BlockPos) -> bool {
        let under_pos = pos.down();
        let under_state = world.get_block_state(&under_pos).await;
        self.can_place_above(world, under_pos, under_state).await
    }

    async fn can_place_above(
        &self,
        _world: &dyn BlockAccessor,
        _pos: BlockPos,
        state: &BlockState,
    ) -> bool {
        state.is_side_solid(BlockDirection::Up)
    }

    async fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        let props = T::from_state_id(args.state.id, args.block);
        if props.is_powered() && props.get_facing().to_block_direction() == args.direction {
            self.get_output_level(args.world, *args.location).await
        } else {
            0
        }
    }

    async fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        self.get_weak_redstone_power(args).await
    }

    async fn get_output_level(&self, world: &World, pos: BlockPos) -> u8;

    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let state = args.world.get_block_state(args.location).await;
        if RedstoneGateBlock::can_place_at(self, args.world.as_ref(), *args.location).await {
            self.update_powered(args.world, *args.location, state, args.block)
                .await;
            return;
        }
        args.world
            .set_block_state(
                args.location,
                Block::AIR.default_state.id,
                BlockFlags::NOTIFY_ALL,
            )
            .await;
        for dir in BlockDirection::all() {
            args.world
                .update_neighbor(&args.location.offset(dir.to_offset()), args.source_block)
                .await;
        }
    }

    async fn update_powered(&self, world: &World, pos: BlockPos, state: &BlockState, block: &Block);

    async fn has_power(
        &self,
        world: &World,
        pos: BlockPos,
        state: &BlockState,
        block: &Block,
    ) -> bool {
        self.get_power(world, pos, state, block).await > 0
    }

    async fn get_power(
        &self,
        world: &World,
        pos: BlockPos,
        state: &BlockState,
        block: &Block,
    ) -> u8 {
        get_power::<T>(world, pos, state.id, block).await
    }

    async fn get_max_input_level_sides(
        &self,
        world: &World,
        pos: BlockPos,
        state_id: BlockStateId,
        block: &Block,
        only_gate: bool,
    ) -> u8 {
        let props = T::from_state_id(state_id, block);
        let facing = props.get_facing();

        let power_left = get_power_on_side(world, &pos, facing.rotate_clockwise(), only_gate).await;
        let power_right =
            get_power_on_side(world, &pos, facing.rotate_counter_clockwise(), only_gate).await;

        std::cmp::max(power_left, power_right)
    }

    async fn update_target(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        state_id: BlockStateId,
        block: &Block,
    ) {
        let props = T::from_state_id(state_id, block);
        let facing = props.get_facing();
        let front_pos = pos.offset(facing.opposite().to_offset());
        world.update_neighbor(&front_pos, block).await;
        world
            .update_neighbors(&front_pos, Some(facing.to_block_direction()))
            .await;
    }

    async fn on_place(&self, player: &Player, block: &Block) -> BlockStateId {
        let mut props = T::default(block);
        let dir = player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();
        props.set_facing(dir);

        props.to_state_id(block)
    }

    async fn player_placed(&self, args: PlayerPlacedArgs<'_>) {
        if let Some(state) = get_state_by_state_id(args.state_id) {
            if RedstoneGateBlock::has_power(self, args.world, *args.location, state, args.block)
                .await
            {
                args.world
                    .schedule_block_tick(args.block, *args.location, 1, TickPriority::Normal)
                    .await;
            }
        }
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if args.moved
            || Block::from_state_id(args.old_state_id)
                .is_some_and(|old_block| old_block == args.block)
        {
            return;
        }
        if let Some(old_state) = get_state_by_state_id(args.old_state_id) {
            RedstoneGateBlock::update_target(
                self,
                args.world,
                *args.location,
                old_state.id,
                args.block,
            )
            .await;
        }
    }

    async fn is_target_not_aligned(
        &self,
        world: &dyn BlockAccessor,
        pos: BlockPos,
        state: &BlockState,
        block: &Block,
    ) -> bool {
        let props = T::from_state_id(state.id, block);
        let facing = props.get_facing().opposite();
        let (target_block, target_state) = world
            .get_block_and_block_state(&pos.offset(facing.to_offset()))
            .await;
        if target_block == &Block::COMPARATOR {
            let props = ComparatorLikeProperties::from_state_id(target_state.id, target_block);
            props.facing != facing
        } else if target_block == &Block::REPEATER {
            let props = RepeaterLikeProperties::from_state_id(target_state.id, target_block);
            props.facing != facing
        } else {
            false
        }
    }

    fn get_update_delay_internal(&self, state_id: BlockStateId, block: &Block) -> u16;
}

pub async fn get_power<T: BlockProperties + RedstoneGateBlockProperties + Send>(
    world: &World,
    pos: BlockPos,
    state_id: BlockStateId,
    block: &Block,
) -> u8 {
    let props = T::from_state_id(state_id, block);
    let facing = props.get_facing();
    let source_pos = pos.offset(facing.to_offset());
    let (source_block, source_state) = world.get_block_and_block_state(&source_pos).await;
    let source_level = get_redstone_power(
        source_block,
        source_state,
        world,
        &source_pos,
        facing.to_block_direction(),
    )
    .await;
    if source_level >= 15 {
        source_level
    } else {
        source_level.max(if source_block == &Block::REDSTONE_WIRE {
            let props = RedstoneWireLikeProperties::from_state_id(source_state.id, source_block);
            props.power.to_index() as u8
        } else {
            0
        })
    }
}

async fn get_power_on_side(
    world: &World,
    pos: &BlockPos,
    side: HorizontalFacing,
    only_gate: bool,
) -> u8 {
    let side_pos = pos.offset(side.to_block_direction().to_offset());
    let (side_block, side_state) = world.get_block_and_block_state(&side_pos).await;
    if !only_gate || is_diode(side_block) {
        world
            .block_registry
            .get_weak_redstone_power(
                side_block,
                world,
                &side_pos,
                side_state,
                side.to_block_direction(),
            )
            .await
    } else {
        0
    }
}
