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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn vanilla_shape_class(block: &Block) -> bool {
        block.id.has_tag(tag::Block::MINECRAFT_STAIRS)
            || block.id.has_tag(tag::Block::MINECRAFT_SLABS)
            || block.id.has_tag(tag::Block::MINECRAFT_WOODEN_SHELVES)
            || SINGLETONS.contains(&block.id)
    }

    fn each_state(mut f: impl FnMut(&'static Block, &'static BlockState)) {
        for raw in 0..BlockStateId::COUNT {
            let Some(id) = BlockStateId::new(raw) else {
                continue;
            };
            let state = BlockState::from_id(id);
            f(Block::from_state_id(id), state);
        }
    }

    #[test]
    fn uses_shape_matches_vanilla_class_list() {
        let mut mismatches = Vec::new();
        each_state(|block, state| {
            let expected = state.can_occlude() && vanilla_shape_class(block);
            if uses_shape_for_light_occlusion(state) != expected {
                mismatches.push(block.name);
            }
        });
        mismatches.sort_unstable();
        mismatches.dedup();
        assert!(
            mismatches.is_empty(),
            "uses_shape_for_light_occlusion drifted from tags+singletons: {mismatches:?}"
        );
    }

    #[test]
    fn occluding_faces_match_is_side_solid() {
        each_state(|_block, state| {
            let shaped = uses_shape_for_light_occlusion(state);
            for dir in BlockDirection::all() {
                assert_eq!(
                    face_occludes(state.id, dir),
                    shaped && state.is_side_solid(dir),
                    "{} {dir:?}",
                    state.id.to_block().name
                );
            }
        });
    }

    /// `can_occlude && sided_transparency` is not vanilla `useShapeForLightOcclusion`.
    #[test]
    fn sided_transparency_is_not_the_shape_list() {
        let mut extra = BTreeSet::new();
        let mut missing = BTreeSet::new();
        each_state(|block, state| {
            let flags = state.can_occlude() && state.sided_transparency();
            let shaped = uses_shape_for_light_occlusion(state);
            if flags && !shaped {
                extra.insert(block.name);
            }
            if shaped && !flags {
                missing.insert(block.name);
            }
        });
        assert!(
            missing.contains("oak_stairs")
                && missing.contains("oak_slab")
                && missing.contains("oak_shelf")
                && missing.contains("snow"),
            "shape-list blocks that lack sided_transparency: {missing:?}"
        );
        assert!(
            extra.contains("oak_fence") && extra.contains("white_carpet"),
            "sided_transparency extras: {extra:?}"
        );
    }

    #[test]
    fn shape_list_matches_vanilla_classes() {
        for block in [
            Block::OAK_STAIRS,
            Block::OAK_SLAB,
            Block::FARMLAND,
            Block::DIRT_PATH,
            Block::SNOW,
        ] {
            assert!(
                uses_shape_for_light_occlusion(block.default_state),
                "{}",
                block.name
            );
        }
        for block in [
            Block::GLASS,
            Block::OAK_DOOR,
            Block::OAK_TRAPDOOR,
            Block::WHITE_CARPET,
            Block::IRON_BARS,
            Block::OAK_FENCE,
        ] {
            assert!(
                !uses_shape_for_light_occlusion(block.default_state),
                "{}",
                block.name
            );
        }
    }

    #[test]
    fn stair_covers_some_but_not_all_faces() {
        let stair = Block::OAK_STAIRS.default_state.id;
        let solid = BlockDirection::all()
            .into_iter()
            .filter(|d| face_occludes(stair, *d))
            .count();
        assert!((1..6).contains(&solid));
    }

    #[test]
    fn covered_face_stops_light_opacity_would_pass() {
        let water = Block::WATER.default_state.id;
        let stair = Block::JUNGLE_STAIRS
            .states
            .iter()
            .map(|s| s.id)
            .find(|id| face_occludes(*id, BlockDirection::Up))
            .expect("some stair state is covered on top");

        assert!(shape_occludes(water, stair, BlockDirection::Down));
        assert!(!shape_occludes(water, water, BlockDirection::Down));
    }

    #[test]
    fn glass_never_occludes_by_shape() {
        let glass = Block::GLASS.default_state.id;
        for dir in BlockDirection::all() {
            assert!(!face_occludes(glass, dir));
        }
    }
}
