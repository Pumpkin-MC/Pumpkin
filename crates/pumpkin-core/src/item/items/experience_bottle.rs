use std::any::Any;
use std::sync::Arc;

use crate::entity::{Entity, EntityBase};
use crate::entity::player::Player;
use crate::entity::projectile::experience_bottle::ExperienceBottleEntity;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::Hand;

pub struct ExperienceBottleItem;

impl ItemMetadata for ExperienceBottleItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::EXPERIENCE_BOTTLE.id])
    }
}

impl ItemBehaviour for ExperienceBottleItem {
    fn normal_use(&self, _item: &Item, player: &Player) {
        let position = player.position();
        let world = player.world();

        let rand_val = rand::random::<f32>() * 0.4 + 0.8;
        let pitch = 0.4 / rand_val;
        world.play_sound_fine(
            Sound::EntityExperienceBottleThrow,
            SoundCategory::Players,
            &position,
            0.5,
            pitch,
        );

        let entity = Entity::new(world.clone(), position, &EntityType::EXPERIENCE_BOTTLE);
        let bottle = ExperienceBottleEntity::new_shot(entity, player.get_entity());

        let (yaw, player_pitch) = player.rotation();
        // Vanilla: spawnProjectileFromRotation(..., roll: -20.0F, speed: 0.7F, uncertainty: 1.0F)
        bottle
            .thrown
            .set_velocity_from(player_pitch, yaw, -20.0, 0.7, 1.0);
        world.spawn_entity(Arc::new(bottle));

        let mut main_hand = player.inventory.held_item();
        let consumed = if !main_hand.is_empty() && Self::ids().contains(&main_hand.item.id) {
            main_hand.decrement_unless_creative(player.gamemode.load(), 1);
            player.inventory.set_held_item(main_hand);
            true
        } else {
            false
        };

        if !consumed {
            let mut off_hand = player.inventory.off_hand_item();
            if !off_hand.is_empty() && Self::ids().contains(&off_hand.item.id) {
                off_hand.decrement_unless_creative(player.gamemode.load(), 1);
                player.inventory.set_stack_in_hand(Hand::Left, off_hand);
            }
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
