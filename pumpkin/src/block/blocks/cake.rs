use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, CakeLikeProperties, EnumVariants, Integer0To6},
    item::Item,
    sound::{Sound, SoundCategory},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::{GameMode, math::position::BlockPos};
use pumpkin_world::world::BlockFlags;
use rand::{Rng, rng};

use crate::{
    block::{
        blocks::candle_cakes::cake_from_candle,
        pumpkin_block::{NormalUseArgs, PumpkinBlock, UseWithItemArgs},
        registry::BlockActionResult,
    },
    entity::player::Player,
    world::World,
};

#[pumpkin_block("minecraft:cake")]
pub struct CakeBlock;

impl CakeBlock {
    pub async fn consume_if_hungry(
        world: &Arc<World>,
        player: &Player,
        block: &Block,
        location: &BlockPos,
        state_id: u16,
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
                player.send_health().await;
            }
            GameMode::Creative | GameMode::Spectator => {}
        }

        let mut properties = CakeLikeProperties::from_state_id(state_id, block);
        match properties.bites.to_index() {
            0..=5 => {
                properties.bites = Integer0To6::from_index(properties.bites.to_index() + 1);
                world
                    .set_block_state(
                        location,
                        properties.to_state_id(block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                BlockActionResult::Consume
            }
            6 => {
                world
                    .set_block_state(
                        location,
                        Block::AIR.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                BlockActionResult::Consume
            }
            _ => {
                panic!("invalid bite index");
            }
        }
    }
}

#[async_trait]
impl PumpkinBlock for CakeBlock {
    async fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position).await;
        let properties = CakeLikeProperties::from_state_id(state_id, args.block);
        let item_lock = args.item_stack.lock().await;
        let item = item_lock.item;
        drop(item_lock);
        match item.id {
            id if (Item::CANDLE.id..=Item::BLACK_CANDLE.id).contains(&id) => {
                if properties.bites.to_index() != 0 {
                    return Self::consume_if_hungry(
                        args.world,
                        args.player,
                        args.block,
                        args.position,
                        state_id,
                    )
                    .await;
                }

                if args.player.gamemode.load() != GameMode::Creative {
                    let held_item = args.player.inventory.held_item();
                    let mut held_item_guard = held_item.lock().await;
                    held_item_guard.decrement(1);
                }
                args.world
                    .set_block_state(
                        args.position,
                        cake_from_candle(item).default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                let seed: f64 = rng().random();
                args.player
                    .play_sound(
                        Sound::BlockCakeAddCandle as u16,
                        SoundCategory::Blocks,
                        &args.position.to_f64(),
                        1.0,
                        1.0,
                        seed,
                    )
                    .await;
                return BlockActionResult::Consume;
            }
            _ => {
                return Self::consume_if_hungry(
                    args.world,
                    args.player,
                    args.block,
                    args.position,
                    state_id,
                )
                .await;
            }
        }
    }

    async fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state_id = args.world.get_block_state_id(args.position).await;
        Self::consume_if_hungry(args.world, args.player, args.block, args.position, state_id).await
    }
}
