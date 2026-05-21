use rand::{RngExt, rng};

use std::pin::Pin;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::player::Player;
use crate::entity::projectile::ender_pearl::EnderPearlEntity;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;

pub struct EnderPearlItem;

impl ItemMetadata for EnderPearlItem {
    fn ids() -> Box<[u16]> {
        [Item::ENDER_PEARL.id].into()
    }
}

const ROLL: f32 = 0.0;
const POWER: f32 = 1.5;
const DIVERGENCE: f32 = 1.0;
const THROW_SOUND_VOLUME: f32 = 0.5;

impl ItemBehaviour for EnderPearlItem {
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let position = player.position();
            let world = player.world();
            world.play_sound_fine(
                Sound::EntityEnderPearlThrow,
                pumpkin_data::sound::SoundCategory::Neutral,
                &position,
                THROW_SOUND_VOLUME,
                0.4 / (rng().random::<f32>() * 0.4 + 0.8),
            );

            let entity = Entity::new(world.clone(), position, &EntityType::ENDER_PEARL);
            let pearl = EnderPearlEntity::new_shot(entity, &player.living_entity.entity);
            let yaw = player.living_entity.entity.yaw.load();
            let pitch = player.living_entity.entity.pitch.load();
            pearl.thrown.set_velocity_from(
                &player.living_entity.entity,
                pitch,
                yaw,
                ROLL,
                POWER,
                DIVERGENCE,
            );
            world.spawn_entity(Arc::new(pearl)).await;

            // Consume item
            let held_item = player.inventory.held_item();
            let consumed = {
                let mut main_hand = held_item.lock().await;
                if !main_hand.is_empty() && main_hand.item.id == Item::ENDER_PEARL.id {
                    main_hand.decrement_unless_creative(player.gamemode.load(), 1);
                    true
                } else {
                    false
                }
            };

            if !consumed {
                let off_hand_item = player.inventory.off_hand_item().await;
                let mut off_hand = off_hand_item.lock().await;
                if !off_hand.is_empty() && off_hand.item.id == Item::ENDER_PEARL.id {
                    off_hand.decrement_unless_creative(player.gamemode.load(), 1);
                }
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
