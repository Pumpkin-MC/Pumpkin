use rand::{RngExt, rng};
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::EntityBase;
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
    fn normal_use(&self, _block: &Item, player: &Player) {
        let position = player.position();
        let world = player.world();
        world.play_sound_fine(
            Sound::EntitySnowballThrow,
            pumpkin_data::sound::SoundCategory::Neutral,
            &position,
            0.5,
            0.4 / (rng().random::<f32>() * 0.4 + 0.8),
        );
        let entity = Entity::new(world.clone(), position, &EntityType::SNOWBALL);
        let snowball = SnowballEntity::new_shot(entity, player.get_entity());
        let (yaw, pitch) = player.rotation();
        snowball
            .thrown
            .set_velocity_from(pitch, yaw, 0.0, POWER, 1.0);
        world.spawn_entity(Arc::new(snowball));

        // Consume item
        let mut main_hand = player.inventory.held_item();
        let consumed = if !main_hand.is_empty() && main_hand.item.id == Item::SNOWBALL.id {
            main_hand.decrement_unless_creative(player.gamemode.load(), 1);
            player.inventory.set_held_item(main_hand);
            true
        } else {
            false
        };

        if !consumed {
            let mut off_hand = player.inventory.off_hand_item();
            if !off_hand.is_empty() && off_hand.item.id == Item::SNOWBALL.id {
                off_hand.decrement_unless_creative(player.gamemode.load(), 1);
                player
                    .inventory
                    .set_stack_in_hand(pumpkin_util::Hand::Left, off_hand);
            }
        }

        player.increment_stat(
            pumpkin_data::statistic::StatisticCategory::Used,
            Item::SNOWBALL.id as i32,
            1,
        );
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
