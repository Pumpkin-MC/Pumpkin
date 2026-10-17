use rand::RngExt;
use std::any::Any;
use std::sync::Arc;

use crate::entity::player::Player;
use crate::entity::projectile::experience_bottle::ExperienceBottleEntity;
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};

pub struct ExperienceBottleItem;

impl ItemMetadata for ExperienceBottleItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::EXPERIENCE_BOTTLE.id])
    }
}

impl ItemBehaviour for ExperienceBottleItem {
    fn normal_use(&self, _item: &Item, player: &Player) {
        let world = player.world();
        let pos = player.position();
        world.play_sound_fine(
            Sound::EntityExperienceBottleThrow,
            SoundCategory::Neutral,
            &pos,
            0.5,
            0.4 / (rand::rng().random::<f32>() * 0.4 + 0.8),
        );

        let inventory = player.inventory();
        let mut held = inventory.held_item();
        let hand = if !held.is_empty() && held.item == &Item::EXPERIENCE_BOTTLE {
            pumpkin_util::Hand::Right
        } else {
            held = inventory.off_hand_item();
            pumpkin_util::Hand::Left
        };
        let entity = Entity::new(world.clone(), pos, &EntityType::EXPERIENCE_BOTTLE);
        let bottle = ExperienceBottleEntity::new_shot(entity, player.get_entity());
        bottle.set_item_stack(held.split_unless_creative(player.gamemode.load(), 1));
        let (yaw, pitch) = player.rotation();
        bottle.thrown.set_velocity_from(pitch, yaw, -20.0, 0.7, 1.0);
        world.spawn_entity(Arc::new(bottle));

        inventory.set_stack_in_hand(hand, held);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
