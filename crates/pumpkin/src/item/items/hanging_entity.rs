use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::entity::Entity;
use crate::entity::decoration::item_frame::ItemFrameEntity;
use crate::entity::decoration::painting::PaintingEntity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub struct HangingEntityItem;

impl ItemMetadata for HangingEntityItem {
    fn ids() -> Box<[u16]> {
        [
            Item::PAINTING.id,
            Item::ITEM_FRAME.id,
            Item::GLOW_ITEM_FRAME.id,
        ]
        .into()
    }
}

impl ItemBehaviour for HangingEntityItem {
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
        let world = player.world();
        let target_pos = location.offset(face.to_offset());
        let pos = Vector3::new(
            f64::from(target_pos.0.x) + 0.5,
            f64::from(target_pos.0.y) + 0.5,
            f64::from(target_pos.0.z) + 0.5,
        );

        if item.item.id == Item::PAINTING.id {
            if face == BlockDirection::Up || face == BlockDirection::Down {
                return;
            }

            let entity = Entity::new(world.clone(), pos, &EntityType::PAINTING);
            entity
                .data
                .store(i32::from(face.to_index()), Ordering::Relaxed);
            let painting = Arc::new(PaintingEntity::new(entity));
            world.play_sound(Sound::EntityPaintingPlace, SoundCategory::Blocks, &pos);
            world.spawn_entity(painting);
        } else {
            let entity_type = if item.item.id == Item::GLOW_ITEM_FRAME.id {
                &EntityType::GLOW_ITEM_FRAME
            } else {
                &EntityType::ITEM_FRAME
            };

            let entity = Entity::new(world.clone(), pos, entity_type);
            let frame = ItemFrameEntity::new(entity);
            frame.set_facing(face);
            let sound = frame.get_place_sound();
            let frame_arc = Arc::new(frame);
            world.play_sound(sound, SoundCategory::Blocks, &pos);
            world.spawn_entity(frame_arc);
        }
        item.decrement_unless_creative(player.gamemode.load(), 1);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
