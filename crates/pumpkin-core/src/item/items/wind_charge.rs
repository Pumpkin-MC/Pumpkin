use rand::{RngExt, rng};
use std::sync::Arc;

use crate::entity::player::Player;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;

use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::projectile::ThrownItemEntity;
use crate::entity::projectile::wind_charge::{WIND_CHARGE_GRAVITY, WindChargeEntity};
use crate::item::{ItemBehaviour, ItemMetadata};

pub struct WindChargeItem;

impl ItemMetadata for WindChargeItem {
    fn ids() -> Box<[u16]> {
        [Item::WIND_CHARGE.id].into()
    }
}

const POWER: f32 = 1.5;
const THROW_SOUND_VOLUME: f32 = 0.5;

impl ItemBehaviour for WindChargeItem {
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
        if stack.is_empty() || stack.item.id != Item::WIND_CHARGE.id {
            return;
        }

        let world = player.world();
        let position = player.position();

        world.play_sound_fine(
            Sound::EntityWindChargeThrow,
            pumpkin_data::sound::SoundCategory::Neutral,
            &position,
            THROW_SOUND_VOLUME,
            0.4 / (rng().random::<f32>() * 0.4 + 0.8),
        );

        let entity = Entity::new(world.clone(), position, &EntityType::WIND_CHARGE);

        let wind_charge = ThrownItemEntity::new(entity, player.get_entity(), WIND_CHARGE_GRAVITY);
        wind_charge.set_velocity_from(pitch, yaw, 0.0, POWER, 1.0);

        world.spawn_entity(Arc::new(WindChargeEntity::new_normal(wind_charge)));

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
