use async_trait::async_trait;
use std::time::Duration;

use crate::entity::player::Player;
use crate::item::pumpkin_item::PumpkinItem;
use crate::server::Server;
use pumpkin_data::item::Item;
use pumpkin_data::parse_registry_name;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::registry::Block;

#[async_trait]
pub trait PumpkinFood: PumpkinItem {
    async fn can_consume(&self, item: &Item, player: &Player) -> bool {
        item.components
            .food
            .is_some_and(|food| food.can_always_eat || player.hunger_manager.level.load() < 20)
    }

    fn get_eat_time(&self, item: &Item) -> Duration {
        item.components.consumable.map_or_else(
            || Duration::from_millis(1600),
            |consumable| Duration::from_secs_f32(consumable.consume_seconds),
        )
    }

    async fn begin_eating(&self, item: &Item, player: &Player) {
        // Play eating sound

        let sound = item
            .components
            .consumable
            .and_then(|consumable| Sound::from_name(&parse_registry_name(consumable.sound)));

        player
            .world()
            .await
            .play_sound(
                sound.map_or(Sound::EntityGenericEat, |sound| sound),
                SoundCategory::Players,
                &player.living_entity.entity.pos.load(),
            )
            .await;

        // Set metadata to show eating animation
        player.set_eating(item.id, self.get_eat_time(item)).await;
    }
}

#[async_trait]
impl<T: PumpkinFood + Send + Sync> PumpkinItem for T {
    async fn normal_use(&self, item: &Item, player: &Player, _server: &Server) {
        if self.can_consume(item, player).await {
            // Start eating animation and set player state with timer
            self.begin_eating(item, player).await;
        }
    }

    async fn use_on_block(
        &self,
        item: &Item,
        player: &Player,
        _location: BlockPos,
        _block: &Block,
        server: &Server,
    ) {
        // For most food items, using on a block is the same as normal use
        self.normal_use(item, player, server).await;
    }
}
