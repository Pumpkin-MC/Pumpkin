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
        blocks::candle_cakes::cake_from_candle, pumpkin_block::PumpkinBlock,
        registry::BlockActionResult,
    },
    entity::player::Player,
    server::Server,
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
    ) {
        match player.gamemode.load() {
            GameMode::Survival | GameMode::Adventure => {
                let hunger_level = player.hunger_manager.level.load();
                if hunger_level >= 20 {
                    return;
                }
                player.hunger_manager.level.store(20.min(hunger_level + 2));
                player
                    .hunger_manager
                    .saturation
                    .store(player.hunger_manager.saturation.load() + 0.4);
                player.send_health().await;
            }
            GameMode::Creative => {}
            GameMode::Spectator => return,
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
            }
            6 => {
                world
                    .set_block_state(
                        location,
                        Block::AIR.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
            _ => {
                panic!("invalid bite index");
            }
        }
    }
}

#[async_trait]
impl PumpkinBlock for CakeBlock {
    async fn use_with_item(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        item: &Item,
        _server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        let state_id = world.get_block_state_id(&location).await;
        let properties = CakeLikeProperties::from_state_id(state_id, block);
        match item.id {
            id if (Item::CANDLE.id..=Item::BLACK_CANDLE.id).contains(&id) => {
                if properties.bites.to_index() == 0 {
                    world
                        .set_block_state(
                            &location,
                            cake_from_candle(item).default_state.id,
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                    let seed: f64 = rng().random();
                    player
                        .play_sound(
                            Sound::BlockCakeAddCandle as u16,
                            SoundCategory::Ambient,
                            &location.to_f64(),
                            1.0,
                            1.0,
                            seed,
                        )
                        .await;
                    let held_item = &player.inventory.held_item();
                    let mut held_item_guard = held_item.lock().await;
                    held_item_guard.decrement(1);
                } else {
                    Self::consume_if_hungry(world, player, block, &location, state_id).await;
                }
            }
            _ => {
                Self::consume_if_hungry(world, player, block, &location, state_id).await;
            }
        }
        BlockActionResult::Consume
    }

    async fn normal_use(
        &self,
        block: &Block,
        player: &Player,
        location: BlockPos,
        _server: &Server,
        world: &Arc<World>,
    ) {
        let state_id = world.get_block_state_id(&location).await;
        Self::consume_if_hungry(world, player, block, &location, state_id).await;
    }
}
