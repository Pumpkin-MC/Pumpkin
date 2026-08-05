use crate::entity::EntityBase;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::Half;
use pumpkin_data::block_properties::HorizontalAxis;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::block_properties::StairsShape;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, BlockState, Mirror, Rotation, tag};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::block::BlockBehaviour;
use crate::block::BlockFuture;
use crate::block::OnNeighborUpdateArgs;
use crate::block::OnPlaceArgs;
use crate::world::World;

type StairsProperties = pumpkin_data::block_properties::OakStairsLikeProperties;

#[pumpkin_block_from_tag("minecraft:stairs")]
pub struct StairBlock;

impl BlockBehaviour for StairBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut stair_props = StairsProperties::default(args.block);
            stair_props.waterlogged = args.replacing.water_source();

            stair_props.facing = args.player.get_entity().get_horizontal_facing();
            stair_props.half = match args.direction {
                BlockDirection::Up => Half::Top,
                BlockDirection::Down => Half::Bottom,
                _ => match args.use_item_on.cursor_pos.y {
                    0.0..0.5 => Half::Bottom,
                    0.5..1.0 => Half::Top,

                    // This cannot happen normally
                    _ => Half::Bottom,
                },
            };

            stair_props.shape = compute_stair_shape(
                args.world,
                args.position,
                stair_props.facing,
                stair_props.half,
            );

            stair_props.to_state_id(args.block)
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state_id = args.world.get_block_state_id(args.position);
            let mut stair_props = StairsProperties::from_state_id(state_id, args.block);

            let new_shape = compute_stair_shape(
                args.world,
                args.position,
                stair_props.facing,
                stair_props.half,
            );

            if stair_props.shape != new_shape {
                stair_props.shape = new_shape;
                args.world
                    .set_block_state(
                        args.position,
                        stair_props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        })
    }

    fn mirror(&self, block: &Block, state_id: BlockStateId, mirror: Mirror) -> &'static BlockState {
        let stair = StairsProperties::from_state_id(state_id, block);
        BlockState::from_id(mirror_stairs(stair, mirror).to_state_id(block))
    }

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        let stair = StairsProperties::from_state_id(state_id, block);
        BlockState::from_id(rotate_stairs(stair, rotation).to_state_id(block))
    }
}

const fn rotate_facing(facing: HorizontalFacing, rotation: Rotation) -> HorizontalFacing {
    match rotation {
        Rotation::None => facing,
        Rotation::Clockwise90 => facing.rotate_clockwise(),
        Rotation::Rotate180 => facing.opposite(),
        Rotation::CounterClockwise90 => facing.rotate_counter_clockwise(),
    }
}

/// Matches vanilla `StairBlock.rotate`: only the facing changes, the shape is untouched.
const fn rotate_stairs(mut stair: StairsProperties, rotation: Rotation) -> StairsProperties {
    stair.facing = rotate_facing(stair.facing, rotation);
    stair
}

/// Matches vanilla `StairBlock.mirror`. A mirror only has an effect when the facing's axis
/// lines up with the mirror axis (`LEFT_RIGHT` mirrors Z-axis facings, `FRONT_BACK` mirrors
/// X-axis facings) - otherwise it falls through to the block-behaviour default (identity).
/// When it does apply, the facing is rotated 180 degrees and the inner/outer shape is
/// swapped left<->right, except that `FRONT_BACK` leaves `INNER_LEFT`/`INNER_RIGHT`/
/// `STRAIGHT` unchanged - this asymmetry with `LEFT_RIGHT` is intentional, straight from
/// vanilla source, not a transcription error.
fn mirror_stairs(mut stair: StairsProperties, mirror: Mirror) -> StairsProperties {
    match mirror {
        Mirror::LeftRight if stair.facing.to_axis() == HorizontalAxis::Z => {
            stair.facing = rotate_facing(stair.facing, Rotation::Rotate180);
            stair.shape = match stair.shape {
                StairsShape::OuterLeft => StairsShape::OuterRight,
                StairsShape::OuterRight => StairsShape::OuterLeft,
                StairsShape::InnerLeft => StairsShape::InnerRight,
                StairsShape::InnerRight => StairsShape::InnerLeft,
                StairsShape::Straight => StairsShape::Straight,
            };
            stair
        }
        Mirror::FrontBack if stair.facing.to_axis() == HorizontalAxis::X => {
            stair.facing = rotate_facing(stair.facing, Rotation::Rotate180);
            stair.shape = match stair.shape {
                StairsShape::OuterLeft => StairsShape::OuterRight,
                StairsShape::OuterRight => StairsShape::OuterLeft,
                other => other,
            };
            stair
        }
        _ => stair,
    }
}

fn compute_stair_shape(
    world: &World,
    block_pos: &BlockPos,
    facing: HorizontalFacing,
    half: Half,
) -> StairsShape {
    let right_locked = get_stair_properties_if_exists(
        world,
        &block_pos.offset(facing.rotate_clockwise().to_offset()),
    )
    .is_some_and(|other_stair_props| {
        other_stair_props.half == half && other_stair_props.facing == facing
    });

    let left_locked = get_stair_properties_if_exists(
        world,
        &block_pos.offset(facing.rotate_counter_clockwise().to_offset()),
    )
    .is_some_and(|other_stair_props| {
        other_stair_props.half == half && other_stair_props.facing == facing
    });

    if left_locked && right_locked {
        return StairsShape::Straight;
    }

    if let Some(other_stair_props) =
        get_stair_properties_if_exists(world, &block_pos.offset(facing.to_offset()))
        && other_stair_props.half == half
    {
        if !left_locked && other_stair_props.facing == facing.rotate_clockwise() {
            return StairsShape::OuterRight;
        } else if !right_locked && other_stair_props.facing == facing.rotate_counter_clockwise() {
            return StairsShape::OuterLeft;
        }
    }

    if let Some(other_stair_props) =
        get_stair_properties_if_exists(world, &block_pos.offset(facing.opposite().to_offset()))
        && other_stair_props.half == half
    {
        if !right_locked && other_stair_props.facing == facing.rotate_clockwise() {
            return StairsShape::InnerRight;
        } else if !left_locked && other_stair_props.facing == facing.rotate_counter_clockwise() {
            return StairsShape::InnerLeft;
        }
    }

    StairsShape::Straight
}

fn get_stair_properties_if_exists(world: &World, block_pos: &BlockPos) -> Option<StairsProperties> {
    let (block, block_state) = world.get_block_and_state_id(block_pos);
    block
        .has_tag(&tag::Block::MINECRAFT_STAIRS)
        .then(|| StairsProperties::from_state_id(block_state, block))
}

#[cfg(test)]
mod rotate_mirror_tests {
    use super::*;

    const fn stair(facing: HorizontalFacing, shape: StairsShape) -> StairsProperties {
        StairsProperties {
            facing,
            half: Half::Bottom,
            shape,
            waterlogged: false,
        }
    }

    #[test]
    fn rotate_none_is_identity() {
        let s = stair(HorizontalFacing::North, StairsShape::OuterLeft);
        assert_eq!(rotate_stairs(s, Rotation::None), s);
    }

    #[test]
    fn rotate_only_changes_facing_never_shape() {
        for shape in [
            StairsShape::Straight,
            StairsShape::InnerLeft,
            StairsShape::InnerRight,
            StairsShape::OuterLeft,
            StairsShape::OuterRight,
        ] {
            let s = stair(HorizontalFacing::North, shape);
            for rotation in [
                Rotation::Clockwise90,
                Rotation::Rotate180,
                Rotation::CounterClockwise90,
            ] {
                assert_eq!(rotate_stairs(s, rotation).shape, shape);
            }
        }
    }

    #[test]
    fn rotate_clockwise_90_cycles_facing() {
        assert_eq!(
            rotate_facing(HorizontalFacing::North, Rotation::Clockwise90),
            HorizontalFacing::East
        );
        assert_eq!(
            rotate_facing(HorizontalFacing::East, Rotation::Clockwise90),
            HorizontalFacing::South
        );
        assert_eq!(
            rotate_facing(HorizontalFacing::South, Rotation::Clockwise90),
            HorizontalFacing::West
        );
        assert_eq!(
            rotate_facing(HorizontalFacing::West, Rotation::Clockwise90),
            HorizontalFacing::North
        );
    }

    #[test]
    fn rotate_180_is_opposite_facing() {
        assert_eq!(
            rotate_facing(HorizontalFacing::North, Rotation::Rotate180),
            HorizontalFacing::South
        );
        assert_eq!(
            rotate_facing(HorizontalFacing::East, Rotation::Rotate180),
            HorizontalFacing::West
        );
    }

    #[test]
    fn mirror_left_right_applies_when_facing_axis_is_z() {
        // facing north/south -> axis Z, LEFT_RIGHT applies: 180 facing + shape swap
        let s = stair(HorizontalFacing::North, StairsShape::OuterLeft);
        let mirrored = mirror_stairs(s, Mirror::LeftRight);
        assert_eq!(mirrored.facing, HorizontalFacing::South);
        assert_eq!(mirrored.shape, StairsShape::OuterRight);

        let s = stair(HorizontalFacing::South, StairsShape::InnerRight);
        let mirrored = mirror_stairs(s, Mirror::LeftRight);
        assert_eq!(mirrored.facing, HorizontalFacing::North);
        assert_eq!(mirrored.shape, StairsShape::InnerLeft);
    }

    #[test]
    fn mirror_left_right_is_noop_when_facing_axis_is_x() {
        // facing east/west -> axis X, LEFT_RIGHT does not apply (falls to identity default)
        let s = stair(HorizontalFacing::East, StairsShape::OuterLeft);
        assert_eq!(mirror_stairs(s, Mirror::LeftRight), s);
    }

    #[test]
    fn mirror_front_back_applies_when_facing_axis_is_x() {
        let s = stair(HorizontalFacing::East, StairsShape::OuterLeft);
        let mirrored = mirror_stairs(s, Mirror::FrontBack);
        assert_eq!(mirrored.facing, HorizontalFacing::West);
        assert_eq!(mirrored.shape, StairsShape::OuterRight);
    }

    #[test]
    fn mirror_front_back_leaves_inner_shapes_and_straight_unchanged() {
        // This is the vanilla asymmetry: FRONT_BACK swaps outer left/right but does NOT
        // swap inner left/right or touch straight, unlike LEFT_RIGHT.
        for shape in [
            StairsShape::Straight,
            StairsShape::InnerLeft,
            StairsShape::InnerRight,
        ] {
            let s = stair(HorizontalFacing::West, shape);
            let mirrored = mirror_stairs(s, Mirror::FrontBack);
            assert_eq!(mirrored.shape, shape);
            assert_eq!(mirrored.facing, HorizontalFacing::East);
        }
    }

    #[test]
    fn mirror_front_back_is_noop_when_facing_axis_is_z() {
        let s = stair(HorizontalFacing::North, StairsShape::InnerLeft);
        assert_eq!(mirror_stairs(s, Mirror::FrontBack), s);
    }

    #[test]
    fn mirror_none_is_identity() {
        let s = stair(HorizontalFacing::North, StairsShape::OuterRight);
        assert_eq!(mirror_stairs(s, Mirror::None), s);
    }
}
