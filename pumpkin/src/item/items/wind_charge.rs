use std::pin::Pin;
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

impl ItemBehaviour for WindChargeItem {
    fn normal_use<'a>(
        &'a self,
        _block: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();
            let position = player.position();

            let entity = Entity::new(world.clone(), position, &EntityType::WIND_CHARGE);

            let wind_charge =
                ThrownItemEntity::new(entity, player.get_entity(), WIND_CHARGE_GRAVITY);
            let (yaw, pitch) = player.rotation();

            wind_charge.set_velocity_from(player.get_entity(), pitch, yaw, 0.0, POWER, 1.0);
            world
                .spawn_entity(Arc::new(WindChargeEntity::new_normal(wind_charge)))
                .await;

            // Vanilla `WindChargeItem#use` plays WIND_CHARGE_THROW after spawning the projectile,
            // at SoundSource.NEUTRAL, volume 0.5, pitch 0.4F / (random.nextFloat() * 0.4F + 0.8F).
            world.play_sound_fine(
                Sound::EntityWindChargeThrow,
                pumpkin_data::sound::SoundCategory::Neutral,
                &position,
                0.5,
                super::throw_sound_pitch(rand::random()),
            );

            let held_item = player.inventory.held_item();
            let consumed = {
                let mut main_hand = held_item.lock().await;
                if !main_hand.is_empty() && main_hand.item.id == Item::WIND_CHARGE.id {
                    main_hand.decrement_unless_creative(player.gamemode.load(), 1);
                    true
                } else {
                    false
                }
            };

            if !consumed {
                let off_hand_item = player.inventory.off_hand_item().await;
                let mut off_hand = off_hand_item.lock().await;
                if !off_hand.is_empty() && off_hand.item.id == Item::WIND_CHARGE.id {
                    off_hand.decrement_unless_creative(player.gamemode.load(), 1);
                }
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
