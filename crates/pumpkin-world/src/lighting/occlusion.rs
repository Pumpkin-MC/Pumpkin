//! Shape based light occlusion.
//!
//! Most blocks stop light by their flat `opacity`. A few stop it by shape: vanilla
//! `LightEngine.shapeOccludes` treats a fully covered face as blocking outright, whatever the
//! opacity says, which is why e.g. a stair with opacity 0 still casts a hard shadow downward.

use pumpkin_data::{Block, BlockDirection, BlockId, BlockState, BlockStateId, tag};
use std::sync::LazyLock;

/// The blocks in vanilla's list that no tag covers.
const SINGLETONS: [BlockId; 11] = [
    Block::FARMLAND.id,
    Block::DIRT_PATH.id,
    Block::SNOW.id,
    Block::LECTERN.id,
    Block::DAYLIGHT_DETECTOR.id,
    Block::STONECUTTER.id,
    Block::ENCHANTING_TABLE.id,
    Block::END_PORTAL_FRAME.id,
    Block::SCULK_SENSOR.id,
    Block::SCULK_SHRIEKER.id,
    Block::PISTON_HEAD.id,
];

/// Whether this block's light occlusion follows its shape instead of its opacity.
///
/// Mirrors the block classes that override `useShapeForLightOcclusion` in vanilla. It is a list
/// rather than a data flag because the extracted block data carries no `canOcclude`, and
/// `BlockState::sided_transparency` is a superset that also covers every `noOcclusion` block like
/// doors, trapdoors, carpets and glass among them, none of which block light in vanilla.
fn uses_shape_for_light_occlusion(block: &Block) -> bool {
    if block.id.has_tag(tag::Block::MINECRAFT_STAIRS)
        || block.id.has_tag(tag::Block::MINECRAFT_SLABS)
        || block.id.has_tag(tag::Block::MINECRAFT_WOODEN_SHELVES)
    {
        return true;
    }

    SINGLETONS.contains(&block.id)
}

/// One bit per face, for every block state: does light entering through that face stop here.
static OCCLUDING_FACES: LazyLock<Box<[u8]>> = LazyLock::new(|| {
    // Sized by probing: `BlockStateId::new` rejects anything past the last state.
    let count = (0..=u16::MAX)
        .take_while(|raw| BlockStateId::new(*raw).is_some())
        .count();
    let mut faces = vec![0u8; count].into_boxed_slice();
    for (raw, mask) in faces.iter_mut().enumerate() {
        let Some(state_id) = BlockStateId::new(raw as u16) else {
            continue;
        };
        if !uses_shape_for_light_occlusion(Block::from_state_id(state_id)) {
            continue;
        }
        let state = BlockState::from_id(state_id);
        for face in BlockDirection::all() {
            if state.is_side_solid(face) {
                *mask |= 1 << face as u8;
            }
        }
    }
    faces
});

/// Whether light entering `state_id` through its `face` is stopped by the block's shape.
///
/// Vanilla merges the two facing shapes and asks whether the union covers the face
/// (`Shapes.faceShapeOccludes`). Only the entered face is tested here
#[inline]
#[must_use]
pub fn face_occludes(state_id: BlockStateId, face: BlockDirection) -> bool {
    OCCLUDING_FACES
        .get(state_id.as_u16() as usize)
        .is_some_and(|mask| mask & (1 << face as u8) != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_vanillas_shape_occluding_blocks_are_listed() {
        for block in [
            Block::OAK_STAIRS,
            Block::OAK_SLAB,
            Block::FARMLAND,
            Block::DIRT_PATH,
            Block::SNOW,
        ] {
            assert!(
                uses_shape_for_light_occlusion(&block),
                "{} overrides useShapeForLightOcclusion in vanilla",
                block.name
            );
        }

        // `noOcclusion` blocks: vanilla gives them an empty occlusion shape, so they never
        // block light by shape however solid their faces look.
        for block in [
            Block::GLASS,
            Block::OAK_DOOR,
            Block::OAK_TRAPDOOR,
            Block::WHITE_CARPET,
            Block::IRON_BARS,
            Block::OAK_FENCE,
        ] {
            assert!(
                !uses_shape_for_light_occlusion(&block),
                "{} does not occlude light by shape in vanilla",
                block.name
            );
        }
    }

    #[test]
    fn a_stair_stops_light_through_its_full_face_but_not_its_open_one() {
        let stair = Block::OAK_STAIRS.default_state.id;
        let solid: Vec<BlockDirection> = BlockDirection::all()
            .into_iter()
            .filter(|d| face_occludes(stair, *d))
            .collect();
        assert!(
            !solid.is_empty(),
            "a stair has at least one fully covered face"
        );
        assert!(
            solid.len() < 6,
            "a stair is not covered on every face, or it would be a full block"
        );
    }

    #[test]
    fn glass_never_occludes_by_shape() {
        let glass = Block::GLASS.default_state.id;
        for dir in BlockDirection::all() {
            assert!(
                !face_occludes(glass, dir),
                "glass has solid faces but must stay transparent to light"
            );
        }
    }
}
