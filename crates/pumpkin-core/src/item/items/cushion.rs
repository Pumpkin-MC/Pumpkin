use crate::block::registry::BlockActionResult;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::decoration::cushion::CushionEntity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;

pub struct CushionItem;

impl ItemMetadata for CushionItem {
    fn ids() -> Box<[u16]> {
        [
            Item::WHITE_CUSHION.id,
            Item::ORANGE_CUSHION.id,
            Item::MAGENTA_CUSHION.id,
            Item::LIGHT_BLUE_CUSHION.id,
            Item::YELLOW_CUSHION.id,
            Item::LIME_CUSHION.id,
            Item::PINK_CUSHION.id,
            Item::GRAY_CUSHION.id,
            Item::LIGHT_GRAY_CUSHION.id,
            Item::CYAN_CUSHION.id,
            Item::PURPLE_CUSHION.id,
            Item::BLUE_CUSHION.id,
            Item::BROWN_CUSHION.id,
            Item::GREEN_CUSHION.id,
            Item::RED_CUSHION.id,
            Item::BLACK_CUSHION.id,
        ]
        .into()
    }
}

impl ItemBehaviour for CushionItem {
    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        cursor_pos: Vector3<f32>,
        _block: &Block,
        _server: &Server,
    ) -> BlockActionResult {
        if face != BlockDirection::Up {
            return BlockActionResult::Fail;
        }

        let world = player.world();
        let target_pos = if world.get_block_state(&location).replaceable() {
            location
        } else {
            location.offset(face.to_offset())
        };
        let bottom_center = Vector3::new(
            f64::from(target_pos.0.x) + 0.5,
            f64::from(location.0.y) + f64::from(cursor_pos.y),
            f64::from(target_pos.0.z) + 0.5,
        );

        let dimensions = EntityType::CUSHION.dimension;
        let width = f64::from(dimensions[0]);
        let height = f64::from(dimensions[1]);
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

        if !world.is_space_empty(bounding_box)
            || world
                .get_entities_at_box(&bounding_box)
                .iter()
                .any(|e| e.get_entity().entity_type == &EntityType::CUSHION)
        {
            return BlockActionResult::Fail;
        }

        let (player_yaw, _) = player.rotation();
        let rotation = ((wrap_degrees(player_yaw) + 45.0) / 90.0).floor() * 90.0;
        let entity = Entity::new(world.clone(), bottom_center, &EntityType::CUSHION);
        entity.set_rotation(rotation, 0.0);

        let pos = entity.pos.load();
        world.play_sound_fine(
            Sound::EntityCushionPlace,
            SoundCategory::Blocks,
            &pos,
            0.75,
            0.8,
        );

        let cushion = CushionEntity::new(entity, CushionEntity::color_from_stack(item));
        world.spawn_entity(Arc::new(cushion));
        world.emit_game_event(GameEvent::EntityPlace.name(), pos);
        item.decrement_unless_creative(player.gamemode.load(), 1);
        BlockActionResult::Success
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
