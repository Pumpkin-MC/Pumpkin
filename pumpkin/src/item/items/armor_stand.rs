use std::sync::Arc;

use crate::entity::decoration::armor_stand::ArmorStandEntity;
use crate::entity::Entity;
use crate::entity::player::Player;
use crate::server::Server;
use crate::item::{ItemBehaviour, ItemMetadata};
use async_trait::async_trait;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use pumpkin_world::item::ItemStack;
use uuid::Uuid;

pub struct ArmorStandItem;

impl ArmorStandItem {
    fn calculate_placement_position(location: &BlockPos, face: &BlockDirection) -> BlockPos {
        match face {
            BlockDirection::Up => location.offset(Vector3::new(0, 1, 0)),
            BlockDirection::Down => location.offset(Vector3::new(0, -1, 0)),
            BlockDirection::North => location.offset(Vector3::new(0, 0, -1)),
            BlockDirection::South => location.offset(Vector3::new(0, 0, 1)),
            BlockDirection::West => location.offset(Vector3::new(-1, 0, 0)),
            BlockDirection::East => location.offset(Vector3::new(1, 0, 0)),
        }
    }
}

impl ItemMetadata for ArmorStandItem {
    fn ids() -> Box<[u16]> {
        [Item::ARMOR_STAND.id].into()
    }
}

#[async_trait]
impl ItemBehaviour for ArmorStandItem {
    async fn use_on_block(
        &self,
        _item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        _block: &Block,
        _server: &Server
    ) {
        let world = player.world();
        let position = Self::calculate_placement_position(&location, &face).to_f64();

        let (player_yaw, _) = player.rotation();
        let rotation = ((wrap_degrees(player_yaw - 180.0) + 22.5) / 45.0).floor() * 45.0;

        let entity = Entity::new(
            Uuid::new_v4(),
            world.clone(),
            position,
            &EntityType::ARMOR_STAND,
            false
        );

        // TODO: There's a problem with the entity's rotation at East -90. Other people can also replicate it
        entity.set_rotation(rotation, 0.0);

        world.play_sound(Sound::EntityArmorStandPlace, SoundCategory::Blocks, &entity.pos.load()).await;

        let armor_stand = ArmorStandEntity::new(entity);

        world.spawn_entity(Arc::new(armor_stand)).await;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
