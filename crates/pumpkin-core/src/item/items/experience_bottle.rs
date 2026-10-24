use std::any::Any;
use std::sync::Arc;

use crate::entity::player::Player;
use crate::entity::projectile::experience_bottle::ExperienceBottleEntity;
use crate::entity::{Entity, EntityBase};
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::Hand;
use rand::RngExt;

const ROLL: f32 = -20.0;
const POWER: f32 = 0.7;
const DIVERGENCE: f32 = 1.0;

pub struct ExperienceBottleItem;

impl ItemMetadata for ExperienceBottleItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::EXPERIENCE_BOTTLE.id])
    }
}

impl ItemBehaviour for ExperienceBottleItem {
    fn normal_use(&self, item: &Item, player: &Player) {
        self.normal_use_with_hand(item, player, 0.0, 0.0, Hand::Right);
    }

    fn normal_use_with_hand(
        &self,
        _item: &Item,
        player: &Player,
        _yaw: f32,
        _pitch: f32,
        hand: Hand,
    ) {
        let world = player.world();
        let position = player.position();
        let sound_pitch = 0.4 / rand::rng().random::<f32>().mul_add(0.4, 0.8);
        world.play_sound_fine(
            Sound::EntityExperienceBottleThrow,
            SoundCategory::Neutral,
            &position,
            0.5,
            sound_pitch,
        );

        let mut stack = player.inventory().get_stack_in_hand(hand);
        let entity = Entity::new(world.clone(), position, &EntityType::EXPERIENCE_BOTTLE);
        let bottle = ExperienceBottleEntity::new_shot(entity, player.get_entity());
        bottle.set_item_stack(stack.copy_with_count(1));

        let (yaw, pitch) = player.rotation();
        bottle
            .thrown
            .set_velocity_from(pitch, yaw, ROLL, POWER, DIVERGENCE);
        world.spawn_entity(Arc::new(bottle));

        stack.decrement_unless_creative(player.gamemode.load(), 1);
        player.inventory().set_stack_in_hand(hand, stack);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
