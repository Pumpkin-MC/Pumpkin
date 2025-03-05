use std::sync::Arc;

use crate::entity::player::Player;
use crate::entity::projectile::ThrownItemEntity;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use async_trait::async_trait;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;

pub struct SnowBallItem;

impl ItemMetadata for SnowBallItem {
    const IDS: &'static [u16] = &[Item::SNOWBALL.id];
}

const POWER: f32 = 1.5;

#[async_trait]
impl PumpkinItem for SnowBallItem {
    async fn normal_use(&self, _block: &Item, player: &Player) {
        let position = player.position();
        let world = player.world().await;
        world
            .play_sound(
                Sound::EntitySnowballThrow,
                pumpkin_data::sound::SoundCategory::Neutral,
                &position,
            )
            .await;
        let entity = world.create_entity(position, EntityType::SNOWBALL);
        let snowball = ThrownItemEntity::new(entity, &player.living_entity.entity);
        let yaw = player.living_entity.entity.yaw.load();
        let pitch = player.living_entity.entity.pitch.load();
        snowball.set_velocity_from(&player.living_entity.entity, pitch, yaw, 0.0, POWER, 1.0);
        world.spawn_entity(Arc::new(snowball)).await;
    }
}
