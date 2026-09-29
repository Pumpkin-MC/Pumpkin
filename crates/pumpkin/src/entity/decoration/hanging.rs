use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::decoration::item_frame::ItemFrameEntity;
use crate::entity::decoration::painting::PaintingEntity;
use crate::world::World;

/// Distance from a block's center to the wall plane a hanging entity sits on.
pub const WALL_OFFSET: f64 = 0.46875;

/// Thickness of a hanging entity along its facing axis.
pub const DEPTH: f64 = 0.0625;

/// Vanilla `AABB.ofSize`.
#[must_use]
pub const fn box_of_size(center: Vector3<f64>, size: Vector3<f64>) -> BoundingBox {
    let half = Vector3::new(size.x / 2.0, size.y / 2.0, size.z / 2.0);
    BoundingBox::new(
        Vector3::new(center.x - half.x, center.y - half.y, center.z - half.z),
        Vector3::new(center.x + half.x, center.y + half.y, center.z + half.z),
    )
}

/// Whether another hanging entity (frame or painting) occupies `bounding_box`.
#[must_use]
pub fn overlaps_hanging_entity(world: &World, bounding_box: &BoundingBox, own_id: i32) -> bool {
    world.entities.load().iter().any(|other| {
        other.get_entity().entity_id != own_id
            && other
                .cast_any()
                .downcast_ref::<PaintingEntity>()
                .map_or_else(
                    || {
                        other.cast_any().is::<ItemFrameEntity>()
                            && other
                                .get_entity()
                                .bounding_box
                                .load()
                                .intersects(bounding_box)
                    },
                    |painting| painting.hanging_bounding_box().intersects(bounding_box),
                )
    })
}
