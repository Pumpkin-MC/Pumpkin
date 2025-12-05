use std::pin::Pin;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::player::Player;
use crate::entity::projectile::ThrownItemEntity;
use crate::item::{ItemBehaviour, ItemMetadata};
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

impl ItemBehaviour for EggItem {
    fn normal_use<'a>(
        &'a self,
        _block: &'a Item,
        player: &'a Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let position = player.position();
            let world = player.world();
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
                &EntityType::EGG,
                false,
            );
            let egg = ThrownItemEntity::new(entity, &player.living_entity.entity);
            let yaw = player.living_entity.entity.yaw.load();
            let pitch = player.living_entity.entity.pitch.load();
            egg.set_velocity_from(&player.living_entity.entity, pitch, yaw, 0.0, POWER, 1.0);
            world.spawn_entity(Arc::new(egg)).await;
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
