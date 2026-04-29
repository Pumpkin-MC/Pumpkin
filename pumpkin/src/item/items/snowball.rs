use std::pin::Pin;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::player::Player;
use crate::entity::projectile::snowball::SnowballEntity;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;

pub struct SnowBallItem;

impl ItemMetadata for SnowBallItem {
    fn ids() -> Box<[u16]> {
        [Item::SNOWBALL.id].into()
    }
}

const POWER: f32 = 1.5;

impl ItemBehaviour for SnowBallItem {
    fn normal_use<'a>(
        &'a self,
        _block: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let position = player.position();
            let world = player.world();
            world
                .play_sound(
                    Sound::EntitySnowballThrow,
                    pumpkin_data::sound::SoundCategory::Neutral,
                    &position,
                )
                .await;
            let entity = Entity::new(world.clone(), position, &EntityType::SNOWBALL);
            let snowball = SnowballEntity::new_shot(entity, &player.living_entity.entity).await;
            let yaw = player.living_entity.entity.yaw.load();
            let pitch = player.living_entity.entity.pitch.load();
            snowball.thrown.set_velocity_from(
                &player.living_entity.entity,
                pitch,
                yaw,
                0.0,
                POWER,
                1.0,
            );
            world.spawn_entity(Arc::new(snowball)).await;

            // Consume item
            let held_item = player.inventory.held_item();
            let consumed = {
                let mut main_hand = held_item.lock().await;
                if !main_hand.is_empty() && main_hand.item.id == Item::SNOWBALL.id {
                    main_hand.decrement_unless_creative(player.gamemode.load(), 1);
                    true
                } else {
                    false
                }
            };

            if !consumed {
                let off_hand_item = player.inventory.off_hand_item().await;
                let mut off_hand = off_hand_item.lock().await;
                if !off_hand.is_empty() && off_hand.item.id == Item::SNOWBALL.id {
                    off_hand.decrement_unless_creative(player.gamemode.load(), 1);
                }
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
