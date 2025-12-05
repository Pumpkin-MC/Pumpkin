use std::pin::Pin;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::decoration::armor_stand::ArmorStandEntity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use pumpkin_world::item::ItemStack;
use uuid::Uuid;

pub struct ArmorStandItem;

impl ArmorStandItem {
    fn calculate_placement_position(location: &BlockPos, face: BlockDirection) -> BlockPos {
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

impl ItemBehaviour for ArmorStandItem {
    fn use_on_block<'a>(
        &'a self,
        _item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        face: BlockDirection,
        _block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let world = player.world();
            let position = Self::calculate_placement_position(&location, face).to_f64();

            let bottom_center = Vector3::new(position.x, position.y, position.z);

            let armor_stand_dimensions = EntityType::ARMOR_STAND.dimension;
            let width = f64::from(armor_stand_dimensions[0]);
            let height = f64::from(armor_stand_dimensions[1]);

            let bounding_box = BoundingBox::new(
                Vector3::new(
                    bottom_center.x - width / 2.0,
                    bottom_center.y,
                    bottom_center.z - width / 2.0,
                ),
                Vector3::new(
                    bottom_center.x + width / 2.0,
                    bottom_center.y + height,
                    bottom_center.z + width / 2.0,
                ),
            );

            if world.is_space_empty(bounding_box).await
                && world.get_entities_at_box(&bounding_box).await.is_empty()
            {
                let (player_yaw, _) = player.rotation();
                let rotation = ((wrap_degrees(player_yaw - 180.0) + 22.5) / 45.0).floor() * 45.0;

                let entity = Entity::new(
                    Uuid::new_v4(),
                    world.clone(),
                    position,
                    &EntityType::ARMOR_STAND,
                    false,
                );

                entity.set_rotation(rotation, 0.0);

                world
                    .play_sound(
                        Sound::EntityArmorStandPlace,
                        SoundCategory::Blocks,
                        &entity.pos.load(),
                    )
                    .await;

                let armor_stand = ArmorStandEntity::new(entity);

                world.spawn_entity(Arc::new(armor_stand)).await;
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
