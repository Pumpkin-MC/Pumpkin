//! Shape occlusion. Vanilla `LightEngine.shapeOccludes`.
//!
//! Opacity stops most blocks. Stairs/slabs/farmland/… stop light on a covered face even at opacity 0.

use pumpkin_data::{Block, BlockDirection, BlockId, BlockState, BlockStateId, tag};
use std::sync::LazyLock;

/// Vanilla classes with no covering tag.
const SINGLETONS: [BlockId; 12] = [
    Block::FARMLAND.id,
    Block::DIRT_PATH.id,
    Block::SNOW.id,
    Block::LECTERN.id,
    Block::DAYLIGHT_DETECTOR.id,
    Block::STONECUTTER.id,
    Block::ENCHANTING_TABLE.id,
    Block::END_PORTAL_FRAME.id,
    Block::SCULK_SENSOR.id,
    Block::CALIBRATED_SCULK_SENSOR.id,
    Block::SCULK_SHRIEKER.id,
    Block::PISTON_HEAD.id,
];

/// `canOcclude` + `useShapeForLightOcclusion`.
pub fn uses_shape_for_light_occlusion(state: &BlockState) -> bool {
    if !state.can_occlude() {
        return false;
    }

    let block = Block::from_state_id(state.id);
    if block.id.has_tag(tag::Block::MINECRAFT_STAIRS)
        || block.id.has_tag(tag::Block::MINECRAFT_SLABS)
        || block.id.has_tag(tag::Block::MINECRAFT_WOODEN_SHELVES)
    {
        return true;
    }

    SINGLETONS.contains(&block.id)
}

/// Bit per face, per state.
static OCCLUDING_FACES: LazyLock<Box<[u8]>> = LazyLock::new(|| {
    let mut faces = vec![0u8; BlockStateId::COUNT as usize].into_boxed_slice();
    for (raw, mask) in faces.iter_mut().enumerate() {
        let Some(state_id) = BlockStateId::new(raw as u16) else {
            continue;
        };
        let state = BlockState::from_id(state_id);
        if !uses_shape_for_light_occlusion(state) {
            continue;
        }
        for face in BlockDirection::all() {
            if state.is_side_solid(face) {
                *mask |= 1 << face as u8;
            }
        }
    }
    faces
});

#[inline]
#[must_use]
pub fn face_occludes(state_id: BlockStateId, face: BlockDirection) -> bool {
    OCCLUDING_FACES
        .get(state_id.as_u16() as usize)
        .is_some_and(|mask| mask & (1 << face as u8) != 0)
}

/// Vanilla `shapeOccludes`: union of leaving face and entered opposite face.
#[inline]
#[must_use]
pub fn shape_occludes(from: BlockStateId, to: BlockStateId, dir: BlockDirection) -> bool {
    face_occludes(from, dir) || face_occludes(to, dir.opposite())
}
