use std::sync::Arc;

use crate::{
    block::{
        BlockBehaviour, CanPlaceAtArgs, GetComparatorOutputArgs, GetStateForNeighborUpdateArgs,
        NormalUseArgs, OnPlaceArgs, OnScheduledTickArgs, UseWithItemArgs,
        blocks::candle_cakes::cake_from_candle, registry::BlockActionResult,
    },
    entity::player::Player,
    world::World,
};
use pumpkin_data::item::Item;
use pumpkin_data::{
    Block, BlockStateId,
    block_properties::CakeLikeProperties,
    sound::{Sound, SoundCategory},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::{GameMode, math::position::BlockPos};
use pumpkin_world::{
    tick::TickPriority,
    world::{BlockAccessor, BlockFlags},
};
#[pumpkin_block("minecraft:cake")]
pub struct CakeBlock;

impl CakeBlock {
    pub fn consume_if_hungry(
        world: &Arc<World>,
        player: &Player,
        block: &Block,
        location: &BlockPos,
        state_id: BlockStateId,
    ) -> BlockActionResult {
        match player.gamemode.load() {
            GameMode::Survival | GameMode::Adventure => {
                let hunger_level = player.hunger_manager.level.load();
                if hunger_level >= 20 {
                    return BlockActionResult::Pass;
                }
                player.hunger_manager.level.store(20.min(hunger_level + 2));
                player
                    .hunger_manager
                    .saturation
                    .store(player.hunger_manager.saturation.load() + 0.4);
                player.send_health();
            }
            GameMode::Creative | GameMode::Spectator => {}
        }

        let mut properties = CakeLikeProperties::from_state_id(state_id);
        match properties.bites {
            0..=5 => {
                player.increment_stat(
                    pumpkin_data::statistic::StatisticCategory::Custom,
                    pumpkin_data::statistic::CustomStatistic::EatCakeSlice as i32,
                    1,
                );
                properties.bites += 1;
                world.set_block_state(
                    location,
                    properties.to_state_id(block),
                    BlockFlags::NOTIFY_ALL,
                );
                BlockActionResult::Consume
            }
            6 => {
                player.increment_stat(
                    pumpkin_data::statistic::StatisticCategory::Custom,
                    pumpkin_data::statistic::CustomStatistic::EatCakeSlice as i32,
                    1,
                );
                world.set_block_state(
                    location,
                    Block::AIR.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                );
                BlockActionResult::Consume
            }
            _ => BlockActionResult::Pass,
        }
    }
}

impl BlockBehaviour for CakeBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if !can_place_at(args.world, args.position) {
            return Block::AIR.default_state.id;
        }
        Block::CAKE.default_state.id
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.position)
    }

    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        let properties = CakeLikeProperties::from_state_id(state_id);
        let item = args.item_stack.item;
        match item.id {
            id if (Item::CANDLE.id..=Item::BLACK_CANDLE.id).contains(&id) => {
                if properties.bites != 0 {
                    return Self::consume_if_hungry(
                        args.world,
                        args.player,
                        args.block,
                        args.position,
                        state_id,
                    );
                }

                if args.player.gamemode.load() != GameMode::Creative {
                    args.item_stack.decrement(1);
                }
                args.world.set_block_state(
                    args.position,
                    cake_from_candle(item).default_state.id,
                    BlockFlags::NOTIFY_ALL,
                );
                args.world.play_sound(
                    Sound::BlockCakeAddCandle,
                    SoundCategory::Blocks,
                    &args.position.to_f64(),
                );
                BlockActionResult::Consume
            }
            _ => Self::consume_if_hungry(
                args.world,
                args.player,
                args.block,
                args.position,
                state_id,
            ),
        }
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position);
        Self::consume_if_hungry(args.world, args.player, args.block, args.position, state_id)
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !can_place_at(args.world.as_ref(), args.position) {
            args.world
                .break_block(args.position, None, BlockFlags::empty());
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !can_place_at(args.world, args.position) {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
        }
        args.state_id
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        {
            let state_id = args.world.get_block_state_id(args.position);
            let properties = CakeLikeProperties::from_state_id(state_id);
            if properties.bites <= 6 {
                Some((7 - properties.bites) * 2)
            } else {
                Some(0)
            }
        }
    }
}

fn can_place_at(world: &dyn BlockAccessor, position: &BlockPos) -> bool {
    let state = world.get_block_state(&position.down());
    state.is_solid()
}
