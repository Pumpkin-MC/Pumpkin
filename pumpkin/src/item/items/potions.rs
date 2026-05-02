use std::pin::Pin;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::player::Player;
use crate::entity::projectile::{
    lingering_potion::LingeringPotionEntity, splash_potion::SplashPotionEntity,
};
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;

pub struct PotionItem;
pub struct SplashPotionItem;
pub struct LingeringPotionItem;

impl ItemMetadata for PotionItem {
    fn ids() -> Box<[u16]> {
        [Item::POTION.id].into()
    }
}

impl ItemMetadata for SplashPotionItem {
    fn ids() -> Box<[u16]> {
        [Item::SPLASH_POTION.id].into()
    }
}

impl ItemMetadata for LingeringPotionItem {
    fn ids() -> Box<[u16]> {
        [Item::LINGERING_POTION.id].into()
    }
}

const POWER: f32 = 0.5;

impl ItemBehaviour for PotionItem {
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        _player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        // Drinking is handled by the consumable flow in the server (active hand + consumption tick).
        Box::pin(async move {})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ItemBehaviour for SplashPotionItem {
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let position = player.position();
            let world = player.world();
            world
                .play_sound(
                    Sound::EntityWitchThrow,
                    pumpkin_data::sound::SoundCategory::Neutral,
                    &position,
                )
                .await;
            let entity = Entity::new(world.clone(), position, &EntityType::SPLASH_POTION);
            let splash = SplashPotionEntity::new_shot(entity, &player.living_entity.entity).await;

            // Copy the held item stack data into the projectile
            let main = player.inventory.held_item();
            let mut used_main = true;
            let mut stack = {
                let s = main.lock().await.clone();
                (!s.is_empty() && s.item.id == pumpkin_data::item::Item::SPLASH_POTION.id)
                    .then_some(s)
            };
            if stack.is_none() {
                let off = player.inventory.off_hand_item().await;
                let s = off.lock().await.clone();
                if !s.is_empty() && s.item.id == pumpkin_data::item::Item::SPLASH_POTION.id {
                    stack = Some(s);
                    used_main = false;
                }
            }
            let stack = stack.unwrap_or_else(|| ItemStack::EMPTY.clone());
            splash.set_item_stack(stack).await;

            let yaw = player.living_entity.entity.yaw.load();
            let pitch = player.living_entity.entity.pitch.load();
            splash.thrown.set_velocity_from(
                &player.living_entity.entity,
                pitch,
                yaw,
                0.0,
                POWER,
                1.0,
            );

            world.spawn_entity(Arc::new(splash)).await;

            // Decrement the used stack (clear)
            if used_main {
                player
                    .inventory
                    .held_item()
                    .lock()
                    .await
                    .decrement_unless_creative(player.gamemode.load(), 1);
            } else {
                player
                    .inventory
                    .off_hand_item()
                    .await
                    .lock()
                    .await
                    .decrement_unless_creative(player.gamemode.load(), 1);
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ItemBehaviour for LingeringPotionItem {
    fn normal_use<'a>(
        &'a self,
        _item: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let position = player.position();
            let world = player.world();
            world
                .play_sound(
                    Sound::EntityWitchThrow,
                    pumpkin_data::sound::SoundCategory::Neutral,
                    &position,
                )
                .await;
            let entity = Entity::new(world.clone(), position, &EntityType::LINGERING_POTION);
            let ling = LingeringPotionEntity::new_shot(entity, &player.living_entity.entity).await;

            // Copy the held item stack data into the projectile
            let main = player.inventory.held_item();
            let mut used_main = true;
            let mut stack = {
                let s = main.lock().await.clone();
                (!s.is_empty() && s.item.id == pumpkin_data::item::Item::LINGERING_POTION.id)
                    .then_some(s)
            };
            if stack.is_none() {
                let off = player.inventory.off_hand_item().await;
                let s = off.lock().await.clone();
                if !s.is_empty() && s.item.id == pumpkin_data::item::Item::LINGERING_POTION.id {
                    stack = Some(s);
                    used_main = false;
                }
            }
            let stack = stack.unwrap_or_else(|| ItemStack::EMPTY.clone());
            ling.set_item_stack(stack).await;

            let yaw = player.living_entity.entity.yaw.load();
            let pitch = player.living_entity.entity.pitch.load();
            ling.thrown.set_velocity_from(
                &player.living_entity.entity,
                pitch,
                yaw,
                0.0,
                POWER,
                1.0,
            );

            world.spawn_entity(Arc::new(ling)).await;

            // Decrement the used stack (clear)
            if used_main {
                player
                    .inventory
                    .held_item()
                    .lock()
                    .await
                    .decrement_unless_creative(player.gamemode.load(), 1);
            } else {
                player
                    .inventory
                    .off_hand_item()
                    .await
                    .lock()
                    .await
                    .decrement_unless_creative(player.gamemode.load(), 1);
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
