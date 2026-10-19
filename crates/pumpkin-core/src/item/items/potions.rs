use rand::{RngExt, rng};
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::entity::projectile::{
    lingering_potion::LingeringPotionEntity, splash_potion::SplashPotionEntity,
};
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};

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

const ROLL: f32 = -20.0;
const POWER: f32 = 0.5;
const DIVERGENCE: f32 = 1.0;
const THROW_SOUND_VOLUME: f32 = 0.5;

impl ItemBehaviour for PotionItem {
    fn normal_use(&self, _item: &Item, _player: &Player) {
        // Drinking is handled by the consumable flow in the server (active hand + consumption tick).
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ItemBehaviour for SplashPotionItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_hand(item, player, yaw, pitch, pumpkin_util::Hand::Right);
    }

    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        yaw: f32,
        pitch: f32,
        hand: pumpkin_util::Hand,
    ) {
        let mut stack = match hand {
            pumpkin_util::Hand::Right => player.inventory.held_item(),
            pumpkin_util::Hand::Left => player.inventory.off_hand_item(),
        };
        if stack.is_empty() || stack.item.id != Item::SPLASH_POTION.id {
            return;
        }

        let position = player.position();
        let world = player.world();
        world.play_sound_fine(
            Sound::EntitySplashPotionThrow,
            SoundCategory::Players,
            &position,
            THROW_SOUND_VOLUME,
            0.4 / (rng().random::<f32>() * 0.4 + 0.8),
        );
        let entity = Entity::new(world.clone(), position, &EntityType::SPLASH_POTION);
        let splash = SplashPotionEntity::new_shot(entity, player.get_entity());
        splash.set_item_stack(stack.copy_with_count(1));

        splash
            .thrown
            .set_velocity_from(pitch, yaw, ROLL, POWER, DIVERGENCE);

        world.spawn_entity(Arc::new(splash));

        stack.decrement_unless_creative(player.gamemode.load(), 1);
        match hand {
            pumpkin_util::Hand::Right => player.inventory.set_held_item(stack),
            pumpkin_util::Hand::Left => player
                .inventory
                .set_stack_in_hand(pumpkin_util::Hand::Left, stack),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ItemBehaviour for LingeringPotionItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_hand(item, player, yaw, pitch, pumpkin_util::Hand::Right);
    }

    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        yaw: f32,
        pitch: f32,
        hand: pumpkin_util::Hand,
    ) {
        let mut stack = match hand {
            pumpkin_util::Hand::Right => player.inventory.held_item(),
            pumpkin_util::Hand::Left => player.inventory.off_hand_item(),
        };
        if stack.is_empty() || stack.item.id != Item::LINGERING_POTION.id {
            return;
        }

        let position = player.position();
        let world = player.world();
        world.play_sound_fine(
            Sound::EntityLingeringPotionThrow,
            SoundCategory::Neutral,
            &position,
            THROW_SOUND_VOLUME,
            0.4 / (rng().random::<f32>() * 0.4 + 0.8),
        );
        let entity = Entity::new(world.clone(), position, &EntityType::LINGERING_POTION);
        let ling = LingeringPotionEntity::new_shot(entity, player.get_entity());
        ling.set_item_stack(stack.copy_with_count(1));

        ling.thrown
            .set_velocity_from(pitch, yaw, ROLL, POWER, DIVERGENCE);

        world.spawn_entity(Arc::new(ling));

        stack.decrement_unless_creative(player.gamemode.load(), 1);
        match hand {
            pumpkin_util::Hand::Right => player.inventory.set_held_item(stack),
            pumpkin_util::Hand::Left => player
                .inventory
                .set_stack_in_hand(pumpkin_util::Hand::Left, stack),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
