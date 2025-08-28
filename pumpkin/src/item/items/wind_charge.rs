use std::sync::Arc;

use crate::entity::player::Player;
use async_trait::async_trait;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;
use uuid::Uuid;

use crate::entity::Entity;
use crate::entity::projectile::ThrownItemEntity;
use crate::entity::projectile::wind_charge::WindChargeEntity;
use crate::item::{ItemBehaviour, ItemMetadata};

pub struct WindChargeItem;

impl ItemMetadata for WindChargeItem {
    fn ids() -> Box<[u16]> {
        [Item::WIND_CHARGE.id].into()
    }
}

const POWER: f32 = 1.5;

#[async_trait]
impl ItemBehaviour for WindChargeItem {
    async fn normal_use(&self, _block: &Item, player: &Player) {
        let world = player.world();
        let position = player.position();

        // TODO: Implement Cooldown to throw the item

        world
            .play_sound(
                Sound::EntityWindChargeThrow,
                pumpkin_data::sound::SoundCategory::Neutral,
                &position,
            )
            .await;

        let entity = Entity::new(
            Uuid::new_v4(),
            world.clone(),
            position,
            &EntityType::WIND_CHARGE,
            false,
        );

        let wind_charge = ThrownItemEntity::new(entity, &player.living_entity.entity);
        let yaw = player.living_entity.entity.yaw.load();
        let pitch = player.living_entity.entity.pitch.load();

        wind_charge.set_velocity_from(&player.living_entity.entity, pitch, yaw, 0.0, POWER, 1.0);
        // TODO: player.incrementStat(Stats.USED)

        // TODO: Implement that the projectile will explode on impact on ground
        world
            .spawn_entity(Arc::new(WindChargeEntity::new(wind_charge)))
            .await;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
