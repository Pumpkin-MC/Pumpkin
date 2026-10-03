use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::block::registry::BlockActionResult;
use crate::entity::Entity;
use crate::entity::decoration::item_frame::ItemFrameEntity;
use crate::entity::decoration::painting::PaintingEntity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::data_component_impl::PaintingVariantImpl;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::painting_variant::PaintingVariant;
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
    ) -> BlockActionResult {
        let world = player.world();
        let target_pos = location.offset(face.to_offset());
        let pos = Vector3::new(
            f64::from(target_pos.0.x) + 0.5,
            f64::from(target_pos.0.y) + 0.5,
            f64::from(target_pos.0.z) + 0.5,
        );

        if item.item.id == Item::PAINTING.id {
            if face == BlockDirection::Up || face == BlockDirection::Down {
                return BlockActionResult::Fail;
            }

            let variant = if let Some(comp) = item.get_data_component::<PaintingVariantImpl>() {
                if let Some(v) = PaintingVariant::from_name(&comp.value) {
                    if !PaintingEntity::painting_fits(&world, location, face, v) {
                        return BlockActionResult::Fail;
                    }
                    v
                } else {
                    return BlockActionResult::Fail;
                }
            } else {
                let Some(v) = PaintingEntity::choose_variant(&world, location, face) else {
                    return BlockActionResult::Fail;
                };
                v
            };

            let spawn_pos = PaintingEntity::calculate_center_pos(
                location,
                face,
                variant.width(),
                variant.height(),
            );
            let entity = Entity::new(world.clone(), spawn_pos, &EntityType::PAINTING);
            entity
                .data
                .store(i32::from(face.to_index()), Ordering::Relaxed);
            let painting = Arc::new(PaintingEntity::new_with_variant(entity, variant));
            world.play_sound(
                Sound::EntityPaintingPlace,
                SoundCategory::Blocks,
                &spawn_pos,
            );
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
        BlockActionResult::Success
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
