//! Shape based light occlusion.
//!
//! Most blocks stop light by their flat `opacity`. A few stop it by shape: vanilla
//! `LightEngine.shapeOccludes` treats a fully covered face as blocking outright, whatever the
//! opacity says, which is why e.g. a stair with opacity 0 still casts a hard shadow downward.
//!
//! Not covered by [`LightEngine::get_light_dampening_into`], which only reaches full solid
//! render blocks, those already stop light through their opacity of 15.

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
/// Vanilla asks two things (`LightEngine.isEmptyShape`): `canOcclude`, which the block data now
/// carries, and `useShapeForLightOcclusion`, which it does not. `sided_transparency` is the
/// closest exported flag and a superset, it also covers carpets, comparators and composters,
/// which vanilla lets light through.
fn uses_shape_for_light_occlusion(state: &BlockState) -> bool {
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

/// Whether light moving in `dir` is stopped by the shapes of the block it leaves and the block
/// it enters. Vanilla `LightEngine.shapeOccludes`.
///
/// Vanilla merges the two facing shapes and asks whether the union covers the face
#[inline]
#[must_use]
pub fn shape_occludes(from: BlockStateId, to: BlockStateId, dir: BlockDirection) -> bool {
    face_occludes(from, dir) || face_occludes(to, dir.opposite())
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
                uses_shape_for_light_occlusion(block.default_state),
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
                !uses_shape_for_light_occlusion(block.default_state),
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

    /// `can_occlude` is what keeps glass out; the class list alone would not, since glass has
    /// six solid faces. Pins the gate rather than the outcome.
    #[test]
    fn can_occlude_is_what_excludes_glass() {
        assert!(
            !Block::GLASS.default_state.can_occlude(),
            "glass is noOcclusion in vanilla"
        );
        assert!(
            Block::OAK_STAIRS.default_state.can_occlude(),
            "a stair does occlude, and is only shaped differently"
        );
    }

    /// The waterlogged stair that the parity harness caught: light coming straight down is
    /// stopped outright, where its opacity of 1 alone would have let a level through.
    #[test]
    fn a_covered_face_stops_light_the_opacity_would_have_passed() {
        let water = Block::WATER.default_state.id;
        let stair = Block::JUNGLE_STAIRS
            .states
            .iter()
            .map(|s| s.id)
            .find(|id| face_occludes(*id, BlockDirection::Up))
            .expect("some stair state is covered on top");

        assert!(
            shape_occludes(water, stair, BlockDirection::Down),
            "a full upward face blocks light arriving from above"
        );
        assert!(
            !shape_occludes(water, Block::WATER.default_state.id, BlockDirection::Down),
            "water dampens light but never blocks it by shape"
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
