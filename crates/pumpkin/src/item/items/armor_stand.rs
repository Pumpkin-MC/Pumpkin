use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::decoration::armor_stand::ArmorStandEntity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;

pub struct ArmorStandItem;

impl ItemMetadata for ArmorStandItem {
    fn ids() -> Box<[u16]> {
        [Item::ARMOR_STAND.id].into()
    }
}

impl ItemBehaviour for ArmorStandItem {
    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        _block: &Block,
        _server: &Server,
    ) {
        if face == BlockDirection::Down {
            return;
        }

        let world = player.world();
        let target_pos = location.offset(face.to_offset());
        let bottom_center = Vector3::new(
            f64::from(target_pos.0.x) + 0.5,
            f64::from(target_pos.0.y),
            f64::from(target_pos.0.z) + 0.5,
        );

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

        if world.is_space_empty(bounding_box) && world.get_entities_at_box(&bounding_box).is_empty()
        {
            let (player_yaw, _) = player.rotation();
            let rotation = ((wrap_degrees(player_yaw - 180.0) + 22.5) / 45.0).floor() * 45.0;

            let entity = Entity::new(world.clone(), bottom_center, &EntityType::ARMOR_STAND);

            entity.set_rotation(rotation, 0.0);

            world.play_sound(
                Sound::EntityArmorStandPlace,
                SoundCategory::Blocks,
                &entity.pos.load(),
            );

            let armor_stand = ArmorStandEntity::new(entity);

            world.spawn_entity(Arc::new(armor_stand));
            item.decrement_unless_creative(player.gamemode.load(), 1);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
