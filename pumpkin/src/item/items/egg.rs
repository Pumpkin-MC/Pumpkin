use std::sync::Arc;

use crate::entity::player::Player;
use crate::entity::projectile::ThrownItemEntity;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use async_trait::async_trait;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;

pub struct EggItem;

impl ItemMetadata for EggItem {
    const IDS: &'static [u16] = &[Item::EGG.id];
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
        let entity = world.create_entity(position, EntityType::EGG);
        let snowball = ThrownItemEntity::new(entity, &player.living_entity.entity);
        let yaw = player.living_entity.entity.yaw.load();
        let pitch = player.living_entity.entity.pitch.load();
        snowball.set_velocity_from(&player.living_entity.entity, pitch, yaw, 0.0, POWER, 1.0);
        world.spawn_entity(Arc::new(snowball)).await;
    }
}
