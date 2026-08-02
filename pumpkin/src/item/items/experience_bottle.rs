use std::pin::Pin;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::entity::projectile::experience_bottle::ExperienceBottleEntity;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;

pub struct ExperienceBottleItem;

impl ItemMetadata for ExperienceBottleItem {
    fn ids() -> Box<[u16]> {
        [Item::EXPERIENCE_BOTTLE.id].into()
    }
}

// Vanilla values from ExperienceBottleItem.java:
// Projectile.spawnProjectileFromRotation(..., -20.0F, 0.7F, 1.0F);
// The -20.0F is a pitch offset (not used in set_velocity_from directly)
// 0.7F is the power
// 1.0F is the divergence/uncertainty
const POWER: f32 = 0.7;
const DIVERGENCE: f32 = 1.0;

impl ItemBehaviour for ExperienceBottleItem {
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let position = player.position();
            let world = player.world();
            world.play_sound(
                Sound::EntityExperienceBottleThrow,
                pumpkin_data::sound::SoundCategory::Neutral,
                &position,
            );
            let entity = Entity::new(
                world.clone(),
                position,
                &pumpkin_data::entity::EntityType::EXPERIENCE_BOTTLE,
            );
            let bottle = ExperienceBottleEntity::new_shot(entity, player.get_entity());
            let (yaw, pitch) = player.rotation();
            bottle.thrown.set_velocity_from(
                player.get_entity(),
                pitch,
                yaw,
                0.0,
                POWER,
                DIVERGENCE,
            );
            world.spawn_entity(Arc::new(bottle)).await;

            // Consume item
            let held_item = player.inventory.held_item();
            let consumed = {
                let mut main_hand = held_item.lock().await;
                if !main_hand.is_empty() && main_hand.item.id == Item::EXPERIENCE_BOTTLE.id {
                    main_hand.decrement_unless_creative(player.gamemode.load(), 1);
                    true
                } else {
                    false
                }
            };

            if !consumed {
                let off_hand_item = player.inventory.off_hand_item().await;
                let mut off_hand = off_hand_item.lock().await;
                if !off_hand.is_empty() && off_hand.item.id == Item::EXPERIENCE_BOTTLE.id {
                    off_hand.decrement_unless_creative(player.gamemode.load(), 1);
                }
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
