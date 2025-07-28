use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::player::Player;
use crate::entity::projectile::ThrownItemEntity;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use async_trait::async_trait;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;
use uuid::Uuid;

pub struct EggItem;

impl ItemMetadata for EggItem {
    fn ids() -> Box<[u16]> {
        [Item::EGG.id].into()
    }
}

const POWER: f32 = 1.5;

#[async_trait]
impl PumpkinItem for EggItem {
    async fn normal_use(&self, _block: &Item, player: &Player) {
        let position = player.position();
        let world = player.world().await;
        world
            .play_sound(
                Sound::EntityEggThrow,
                pumpkin_data::sound::SoundCategory::Players,
                &position,
            )
            .await;
        // TODO: Implement eggs the right way, so there is a chance of spawning chickens
        let entity = Entity::new(
            Uuid::new_v4(),
            world.clone(),
            position,
            EntityType::EGG,
            false,
        );
        let egg = ThrownItemEntity::new(entity, &player.living_entity.entity);
        let yaw = player.living_entity.entity.yaw.load();
        let pitch = player.living_entity.entity.pitch.load();
        egg.set_velocity_from(&player.living_entity.entity, pitch, yaw, 0.0, POWER, 1.0);
        world.spawn_entity(Arc::new(egg)).await;
    }
}
