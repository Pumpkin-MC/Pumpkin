use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};

use super::*;

#[test]
fn detection_box_is_offset_to_block_position() {
    let bounding_box = detection_box_at(&BlockPos::new(10, 64, -5));

    assert_eq!(bounding_box.min, Vector3::new(10.0625, 64.0, -4.9375));
    assert_eq!(bounding_box.max, Vector3::new(10.9375, 64.25, -4.0625));
}

#[test]
fn entity_at_plate_center_intersects_detection_box() {
    let detection_box = detection_box_at(&BlockPos::new(0, 0, 0));
    let entity_box = BoundingBox::new_array([0.4, 0.0, 0.4], [0.6, 1.8, 0.6]);

    assert!(detection_box.intersects(&entity_box));
}

#[test]
fn entity_outside_horizontal_bounds_does_not_intersect_detection_box() {
    let detection_box = detection_box_at(&BlockPos::new(0, 0, 0));
    let entity_box = BoundingBox::new_array([0.95, 0.0, 0.4], [1.0, 0.2, 0.6]);

    assert!(!detection_box.intersects(&entity_box));
}

#[test]
fn entity_above_detection_height_does_not_intersect_detection_box() {
    let detection_box = detection_box_at(&BlockPos::new(0, 0, 0));
    let entity_box = BoundingBox::new_array([0.4, 0.3, 0.4], [0.6, 0.6, 0.6]);

    assert!(!detection_box.intersects(&entity_box));
}

#[test]
fn entity_partially_overlapping_detection_box_intersects() {
    let detection_box = detection_box_at(&BlockPos::new(0, 0, 0));
    let entity_box = BoundingBox::new_array([0.9, 0.2, 0.4], [0.95, 0.3, 0.6]);

    assert!(detection_box.intersects(&entity_box));
}

#[test]
fn entity_touching_detection_box_boundary_does_not_intersect() {
    let detection_box = detection_box_at(&BlockPos::new(0, 0, 0));
    let entity_box = BoundingBox::new_array([0.9375, 0.0, 0.4], [1.0, 0.2, 0.6]);

    assert!(!detection_box.intersects(&entity_box));
}
