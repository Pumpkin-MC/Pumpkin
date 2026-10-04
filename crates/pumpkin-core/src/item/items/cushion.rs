use crate::block::registry::BlockActionResult;
use std::sync::Arc;

use crate::entity::Entity;
use crate::entity::decoration::cushion::CushionEntity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::World;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

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

impl CushionItem {
    const COLLISION_SHAPE_RAYCAST_EPSILON: f64 = 0.001;

    /// Vanilla `recalculateContextForSpecialCollisionShapes`: cauldrons, hoppers and composters
    /// take the face and height from their collision shape, so a cushion lands inside them.
    fn recalculate_for_special_collision_shapes(
        world: &World,
        player: &Player,
        location: BlockPos,
        cursor_pos: Vector3<f32>,
    ) -> Option<(BlockDirection, f64)> {
        let (block, state) = world.get_block_and_state(&location);
        if !block.has_tag(&tag::Block::MINECRAFT_CUSHION_USES_COLLISION_SHAPE) {
            return None;
        }

        let click_location = location.0.to_f64() + cursor_pos.to_f64();
        let ray_from = player.eye_position();
        let ray = click_location - ray_from;
        let ray_to = click_location + ray.normalize() * Self::COLLISION_SHAPE_RAYCAST_EPSILON;

        let direction = ray_to - ray_from;
        state
            .get_block_collision_shapes_at(&location)
            .filter_map(|shape| clip_box(&shape.at_pos(location), ray_from, direction))
            .min_by(|(a, _), (b, _)| a.total_cmp(b))
            .map(|(t, hit_face)| (hit_face, direction.y.mul_add(t, ray_from.y)))
    }
}

/// Where a ray from `from` along `direction` enters `bounding_box`, as the fraction of
/// `direction` travelled and the face it enters through, like vanilla `AABB.clip`.
fn clip_box(
    bounding_box: &BoundingBox,
    from: Vector3<f64>,
    direction: Vector3<f64>,
) -> Option<(f64, BlockDirection)> {
    let mut t_enter = 0.0f64;
    let mut t_exit = 1.0f64;
    let mut entry_face = None;
    for (start, delta, min, max, min_face, max_face) in [
        (
            from.x,
            direction.x,
            bounding_box.min.x,
            bounding_box.max.x,
            BlockDirection::West,
            BlockDirection::East,
        ),
        (
            from.y,
            direction.y,
            bounding_box.min.y,
            bounding_box.max.y,
            BlockDirection::Down,
            BlockDirection::Up,
        ),
        (
            from.z,
            direction.z,
            bounding_box.min.z,
            bounding_box.max.z,
            BlockDirection::North,
            BlockDirection::South,
        ),
    ] {
        if delta == 0.0 {
            if start < min || start > max {
                return None;
            }
            continue;
        }
        let (near, far, face) = if delta > 0.0 {
            ((min - start) / delta, (max - start) / delta, min_face)
        } else {
            ((max - start) / delta, (min - start) / delta, max_face)
        };
        if near > t_enter {
            t_enter = near;
            entry_face = Some(face);
        }
        t_exit = t_exit.min(far);
        if t_enter > t_exit {
            return None;
        }
    }
    entry_face.map(|face| (t_enter, face))
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
        // Vanilla validates the hit relative to the clicked block's center.
        if ![cursor_pos.x, cursor_pos.y, cursor_pos.z]
            .into_iter()
            .all(|coordinate| (f64::from(coordinate) - 0.5).abs() < 1.000_000_1)
        {
            return BlockActionResult::Fail;
        }
        let world = player.world();
        let (face, click_y) =
            Self::recalculate_for_special_collision_shapes(&world, player, location, cursor_pos)
                .unwrap_or((face, f64::from(location.0.y) + f64::from(cursor_pos.y)));
        if face != BlockDirection::Up {
            return BlockActionResult::Fail;
        }

        let clicked_pos = if world.get_block_state(&location).replaceable() {
            location
        } else {
            location.offset(face.to_offset())
        };
        // Vanilla keeps the height of the click, so cushions rest on slabs and other low blocks.
        let entity_pos = Vector3::new(
            f64::from(clicked_pos.0.x) + 0.5,
            click_y,
            f64::from(clicked_pos.0.z) + 0.5,
        );

        let dimensions = EntityType::CUSHION.dimension;
        let width = f64::from(dimensions[0]);
        let height = f64::from(dimensions[1]);
        let bounding_box = BoundingBox::new(
            Vector3::new(
                entity_pos.x - width / 2.0,
                entity_pos.y,
                entity_pos.z - width / 2.0,
            ),
            Vector3::new(
                entity_pos.x + width / 2.0,
                entity_pos.y + height,
                entity_pos.z + width / 2.0,
            ),
        );

        if !CushionEntity::can_be_placed_at(&world, &bounding_box)
            || world
                .get_entities_at_box(&bounding_box)
                .iter()
                .any(|entity| entity.cast_any().is::<CushionEntity>())
        {
            return BlockActionResult::Fail;
        }

        let (player_yaw, _) = player.rotation();
        // Vanilla `Direction.fromYRot(...).toYRot()`.
        let rotation = (((player_yaw / 90.0 + 0.5).floor() as i32) & 3) as f32 * 90.0;
        let entity = Entity::new(world.clone(), entity_pos, &EntityType::CUSHION);
        entity.set_rotation(rotation, 0.0);

        world.play_sound(
            Sound::EntityCushionPlace,
            SoundCategory::Blocks,
            &entity.pos.load(),
        );

        let cushion = CushionEntity::new(entity, CushionEntity::color_from_item(item.get_item()));
        world.spawn_entity(Arc::new(cushion));
        item.decrement_unless_creative(player.gamemode.load(), 1);
        BlockActionResult::Success
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
