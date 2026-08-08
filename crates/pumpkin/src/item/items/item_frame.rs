use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::decoration::item_frame::ItemFrameEntity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::game_event::{GameEventContext, emit_game_event};
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

/// The shared placement item for item frames and glow item frames.
///
/// Paintings share the same vanilla class (`HangingEntityItem`) but need painting-variant
/// selection (`Painting.create`, area-sorted `hasSpace`) that has no placement item
/// counterpart in Pumpkin yet, so they are out of scope here.
pub struct ItemFrameItem;

impl ItemFrameItem {
    const fn entity_type(item: &Item) -> &'static EntityType {
        if item.id == Item::GLOW_ITEM_FRAME.id {
            &EntityType::GLOW_ITEM_FRAME
        } else {
            &EntityType::ITEM_FRAME
        }
    }
}

impl ItemMetadata for ItemFrameItem {
    fn ids() -> Box<[u16]> {
        [Item::ITEM_FRAME.id, Item::GLOW_ITEM_FRAME.id].into()
    }
}

impl ItemBehaviour for ItemFrameItem {
    /// `HangingEntityItem.useOn` (`HangingEntityItem.java:34-77`).
    fn use_on_block<'a>(
        &'a self,
        item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        _block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let target = BlockPos(location.0 + face.to_offset());
            // `ItemFrameItem.mayPlace` overrides `HangingEntityItem.mayPlace` and checks
            // `isInsideBuildHeight(blockPos)` without rejecting vertical directions. The
            // player protection hook (`mayUseItemAt`) is not modelled by this placement API.
            let world = player.world();
            if !world.is_in_height_limit(target.0.y) {
                return;
            }

            // `Vec3.atCenterOf(blockPos).relative(direction, -0.46875)` (line 116).
            let offset = face.to_offset();
            let position = Vector3::new(
                f64::from(target.0.x) + 0.5 - f64::from(offset.x) * 0.46875,
                f64::from(target.0.y) + 0.5 - f64::from(offset.y) * 0.46875,
                f64::from(target.0.z) + 0.5 - f64::from(offset.z) * 0.46875,
            );

            let pop_box = ItemFrameEntity::pop_box(position, face);
            // `HangingEntity.hasLevelCollision` (`HangingEntity.java:107-110`).
            let inside_border = {
                let border = world.worldborder.lock().await;
                border.contains(pop_box.min.x, pop_box.min.z)
                    && border.contains(pop_box.max.x - 1.0e-5, pop_box.max.z - 1.0e-5)
            };
            if !inside_border {
                return;
            }
            if !world.is_space_empty(pop_box) {
                return;
            }
            // `HangingEntity.canCoexist` (`HangingEntity.java:98-105`): only other hanging
            // entities (frames, glow frames, paintings) can block a placement -- unlike
            // `is_space_empty`, this must not reject the placing player standing in the box.
            let blocked_by_hanging_entity = world.get_entities_at_box(&pop_box).iter().any(|e| {
                let entity = e.get_entity();
                let is_hanging_entity = entity.entity_type == &EntityType::ITEM_FRAME
                    || entity.entity_type == &EntityType::GLOW_ITEM_FRAME
                    || entity.entity_type == &EntityType::PAINTING;
                is_hanging_entity
                    && entity.data.load(Ordering::Relaxed) == i32::from(face.to_index())
            });
            if blocked_by_hanging_entity {
                return;
            }

            let entity_type = Self::entity_type(item.item);
            let entity = Entity::new(world.clone(), position, entity_type);
            let frame = ItemFrameEntity::new(entity);
            frame.set_facing(face);

            // `HangingEntity.survives` (`HangingEntity.java:82-92`): support-block check.
            if !frame.survives() {
                return;
            }

            let is_glow = entity_type == &EntityType::GLOW_ITEM_FRAME;
            world.play_sound(
                if is_glow {
                    Sound::EntityGlowItemFramePlace
                } else {
                    Sound::EntityItemFramePlace
                },
                SoundCategory::Blocks,
                &position,
            );

            if let Some(player_arc) = world.get_player_by_id(player.get_entity().entity_id) {
                emit_game_event(
                    &world,
                    GameEvent::EntityPlace,
                    position,
                    GameEventContext::of_entity(player_arc),
                )
                .await;
            }

            world.spawn_entity(Arc::new(frame)).await;
            item.decrement_unless_creative(player.gamemode.load(), 1);
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement_box_is_thin_along_the_facing_axis() {
        let position = Vector3::new(10.0, 20.0, 30.0);

        let north = ItemFrameEntity::pop_box(position, BlockDirection::North);
        assert_eq!(north.max.x - north.min.x, 0.75);
        assert_eq!(north.max.y - north.min.y, 0.75);
        assert_eq!(north.max.z - north.min.z, 0.0625);

        let east = ItemFrameEntity::pop_box(position, BlockDirection::East);
        assert_eq!(east.max.x - east.min.x, 0.0625);
        assert_eq!(east.max.y - east.min.y, 0.75);
        assert_eq!(east.max.z - east.min.z, 0.75);

        let up = ItemFrameEntity::pop_box(position, BlockDirection::Up);
        assert_eq!(up.max.x - up.min.x, 0.75);
        assert_eq!(up.max.y - up.min.y, 0.0625);
        assert_eq!(up.max.z - up.min.z, 0.75);
    }

    #[test]
    fn placement_box_is_centered_at_the_hanging_entity_position() {
        let position = Vector3::new(10.0, 20.0, 30.0);
        let bounding_box = ItemFrameEntity::pop_box(position, BlockDirection::South);

        assert_eq!(bounding_box.min.x + bounding_box.max.x, 20.0);
        assert_eq!(bounding_box.min.y + bounding_box.max.y, 40.0);
        assert_eq!(bounding_box.min.z + bounding_box.max.z, 60.0);
    }
}
