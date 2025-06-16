use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, CakeLikeProperties, EnumVariants, Integer0To6},
    entity::EntityPose,
    item::Item,
    sound::{Sound, SoundCategory},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::{Rng, rng};

use crate::{
    block::{pumpkin_block::PumpkinBlock, registry::BlockActionResult},
    entity::{EntityBase, player::Player},
    server::Server,
    world::World,
};

#[pumpkin_block("minecraft:cake")]
pub struct CakeBlock;

impl CakeBlock {
    async fn consume_if_hungry(
        &self,
        world: &Arc<World>,
        player: &Player,
        block: &Block,
        location: &BlockPos,
        state_id: u16,
    ) {
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
                        Block::AIR.default_state_id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
            _ => {
                panic!("invalid hunger index");
            }
        }
    }

    fn candle_cake_from_candle(item: &Item) -> Block {
        match item.id {
            1305 => Block::CANDLE_CAKE,            // Item::CANDLE.id
            1306 => Block::WHITE_CANDLE_CAKE,      // Item::WHITE_CANDLE.id
            1307 => Block::ORANGE_CANDLE_CAKE,     // Item::ORANGE_CANDLE.id
            1308 => Block::MAGENTA_CANDLE_CAKE,    // Item::MAGENTA_CANDLE.id
            1309 => Block::LIGHT_BLUE_CANDLE_CAKE, // Item::LIGHT_BLUE_CANDLE.id
            1310 => Block::YELLOW_CANDLE_CAKE,     // Item::YELLOW_CANDLE.id
            1311 => Block::LIME_CANDLE_CAKE,       // Item::LIME_CANDLE.id
            1312 => Block::PINK_CANDLE_CAKE,       // Item::PINK_CANDLE.id
            1313 => Block::GRAY_CANDLE_CAKE,       // Item::GRAY_CANDLE.id
            1314 => Block::LIGHT_GRAY_CANDLE_CAKE, // Item::LIGHT_GRAY_CANDLE.id
            1315 => Block::CYAN_CANDLE_CAKE,       // Item::CYAN_CANDLE.id
            1316 => Block::PURPLE_CANDLE_CAKE,     // Item::PURPLE_CANDLE.id
            1317 => Block::BLUE_CANDLE_CAKE,       // Item::BLUE_CANDLE.id
            1318 => Block::BROWN_CANDLE_CAKE,      // Item::BROWN_CANDLE.id
            1319 => Block::GREEN_CANDLE_CAKE,      // Item::GREEN_CANDLE.id
            1320 => Block::RED_CANDLE_CAKE,        // Item::RED_CANDLE.id
            1321 => Block::BLACK_CANDLE_CAKE,      // Item::BLACK_CANDLE.id
            other => panic!("Expected a candle block, got {other:?}"),
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
        if player.get_entity().pose.load() == EntityPose::Crouching {
            return BlockActionResult::Continue;
        }
        let state_id = world.get_block_state_id(&location).await;
        let properties = CakeLikeProperties::from_state_id(state_id, block);
        match item.id {
            id if (Item::CANDLE.id..=Item::BLACK_CANDLE.id).contains(&id) => {
                if properties.bites.to_index() == 0 {
                    world
                        .set_block_state(
                            &location,
                            Self::candle_cake_from_candle(item).default_state_id,
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
                } else {
                    self.consume_if_hungry(world, player, block, &location, state_id)
                        .await;
                }
            }
            _ => {
                self.consume_if_hungry(world, player, block, &location, state_id)
                    .await;
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
        self.consume_if_hungry(world, player, block, &location, state_id)
            .await;
    }
}
