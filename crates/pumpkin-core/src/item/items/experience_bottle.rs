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
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_util::Hand;

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
        let (yaw, pitch) = player.rotation();
        self.normal_use_with_hand(item, player, yaw, pitch, Hand::Right);
    }

    fn normal_use_with_hand(&self, item: &Item, player: &Player, yaw: f32, pitch: f32, hand: Hand) {
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
        let mut stack = inventory.get_stack_in_hand(hand);
        let entity = Entity::new(world.clone(), pos, &EntityType::EXPERIENCE_BOTTLE);
        let bottle = ExperienceBottleEntity::new_shot(entity, player.get_entity());
        bottle.set_item_stack(stack.copy_with_count(1));

        bottle
            .thrown
            .set_velocity_from(pitch, yaw, ROLL, POWER, DIVERGENCE);
        world.spawn_entity(Arc::new(bottle));

        stack.decrement_unless_creative(player.gamemode.load(), 1);
        inventory.set_stack_in_hand(hand, stack);
        player.increment_stat(StatisticCategory::Used, i32::from(item.id), 1);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
