use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::player::Player;
use crate::entity::projectile::ThrownItemEntity;
use crate::item::pumpkin_item::PumpkinItem;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;
use pumpkin_data::world::WorldEvent;
use pumpkin_macros::pumpkin_item;
use pumpkin_protocol::client::play::CLevelEvent;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

#[pumpkin_item("splash_potion")]
pub struct SplashPotionItem;

const POWER: f32 = 1.5;

#[async_trait]
impl PumpkinItem for SplashPotionItem {
    async fn normal_use(&self, _item: &Item, player: &Player, server: &Server) {
        let position = player.position();
        let world = player.world().await;
        world
            .play_sound(
                Sound::EntitySplashPotionThrow,
                pumpkin_data::sound::SoundCategory::Neutral,
                &position,
            )
            .await;
        let entity = server.add_entity_with_owner(
            position,
            EntityType::POTION,
            Some(player.entity_id()),
            &world,
        );
        let snowball = ThrownItemEntity::new(
            entity,
            &player.living_entity.entity,
            20,
            Item::SPLASH_POTION,
        );
        let yaw = player.living_entity.entity.yaw.load();
        let pitch = player.living_entity.entity.pitch.load();
        snowball.set_velocity_from(&player.living_entity.entity, pitch, yaw, 0.0, POWER, 1.0);
        world.spawn_entity(Arc::new(snowball)).await;
    }

    async fn on_entity_destroy(&self, _item: &Item, entity: &Entity, world: &World) {
        // TODO: just use entity.block_pos
        let landed_pos = entity.pos.load();
        let landed_block_position = BlockPos(Vector3 {
            x: landed_pos.x as i32,
            y: landed_pos.y as i32,
            z: landed_pos.z as i32,
        });
        world
            .broadcast_packet_all(&CLevelEvent::new(
                WorldEvent::InstantSplashPotionSplashed as i32,
                landed_block_position,
                // TODO: RGB color as an integer (e.g. 8364543 for #7FA1FF).
                8_364_543,
                false,
            ))
            .await;
    }
}
