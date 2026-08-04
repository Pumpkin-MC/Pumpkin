use pumpkin_data::fluid::Fluid;
use pumpkin_data::{
    Block, BlockDirection, BlockStateId,
    block_properties::{BlockProperties, ScaffoldingLikeProperties},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockAccessor;
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockBehaviour, BlockFuture, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    OnScheduledTickArgs, PlacedArgs,
};
use crate::entity::falling::FallingEntity;

/// `net.minecraft.world.level.block.ScaffoldingBlock`: distance-from-support propagation with a
/// tick-driven collapse into a falling-block entity when unsupported.
#[pumpkin_block("minecraft:scaffolding")]
pub struct ScaffoldingBlock;

const MAX_DISTANCE: u8 = 7;

impl ScaffoldingBlock {
    /// `ScaffoldingBlock.getDistance` (`ScaffoldingBlock.java:158-179`).
    ///
    /// Faithful to a vanilla mutable-cursor quirk: the vertical check reads `pos.below()`, but
    /// the horizontal scan afterward is centered on `pos` itself (the cursor is reused via
    /// `setWithOffset(pos, direction)`, not advanced from `pos.below()`).
    fn get_distance(accessor: &dyn BlockAccessor, pos: &BlockPos) -> u8 {
        let below_pos = pos.down();
        let (below_block, below_state) = accessor.get_block_and_state(&below_pos);

        let mut distance = MAX_DISTANCE;
        if below_block == &Block::SCAFFOLDING {
            distance =
                ScaffoldingLikeProperties::from_state_id(below_state.id, below_block).r#distance;
        } else if below_state.is_side_solid(BlockDirection::Up) {
            return 0;
        }

        for direction in [
            BlockDirection::North,
            BlockDirection::South,
            BlockDirection::West,
            BlockDirection::East,
        ] {
            let neighbor_pos = pos.offset(direction.to_offset());
            let (neighbor_block, neighbor_state) = accessor.get_block_and_state(&neighbor_pos);
            if neighbor_block == &Block::SCAFFOLDING {
                let neighbor_distance =
                    ScaffoldingLikeProperties::from_state_id(neighbor_state.id, neighbor_block)
                        .r#distance;
                distance = distance.min(neighbor_distance.saturating_add(1));
                if distance == 1 {
                    break;
                }
            }
        }

        distance
    }

    /// `ScaffoldingBlock.isBottom` (`ScaffoldingBlock.java:154-156`): true when the scaffold isn't
    /// resting directly on another scaffold, but also isn't fully supported (`distance > 0`).
    fn is_bottom(accessor: &dyn BlockAccessor, pos: &BlockPos, distance: u8) -> bool {
        distance > 0 && accessor.get_block(&pos.down()) != &Block::SCAFFOLDING
    }
}

impl BlockBehaviour for ScaffoldingBlock {
    /// `ScaffoldingBlock.getStateForPlacement`.
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = ScaffoldingLikeProperties::default(args.block);
            props.r#waterlogged = args.replacing.water_source();
            props.r#distance = Self::get_distance(args.world, args.position);
            props.r#bottom = Self::is_bottom(args.world, args.position, props.r#distance);
            props.to_state_id(args.block)
        })
    }

    /// `ScaffoldingBlock.canSurvive`: `getDistance(level, pos) < 7`.
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        Self::get_distance(args.block_accessor, args.position) < MAX_DISTANCE
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
        })
    }

    /// `ScaffoldingBlock.updateShape`: schedules the same tick-delay-1 on every neighbor update,
    /// plus a water tick if waterlogged. Does not itself recompute distance/bottom - that only
    /// happens in the scheduled tick.
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let props = ScaffoldingLikeProperties::from_state_id(args.state_id, args.block);
            if props.r#waterlogged {
                args.world.schedule_fluid_tick(
                    &Fluid::WATER,
                    *args.position,
                    Fluid::WATER.flow_speed as u8,
                    TickPriority::Normal,
                );
            }
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
            args.state_id
        })
    }

    /// `ScaffoldingBlock.tick` (`ScaffoldingBlock.java:116-129`). See the doc's gap review for
    /// why this three-way branch must be transcribed literally rather than re-derived: the
    /// destroy-no-entity and fall-as-entity paths have different drop semantics.
    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let old_state_id = args.world.get_block_state_id(args.position);
            let old_props = ScaffoldingLikeProperties::from_state_id(old_state_id, args.block);

            let new_distance = Self::get_distance(args.world.as_ref(), args.position);
            let mut new_props = old_props;
            new_props.r#distance = new_distance;
            new_props.r#bottom = Self::is_bottom(args.world.as_ref(), args.position, new_distance);
            let new_state_id = new_props.to_state_id(args.block);

            if new_distance == MAX_DISTANCE {
                if old_props.r#distance == MAX_DISTANCE {
                    FallingEntity::replace_spawn(args.world, *args.position, new_state_id).await;
                } else {
                    args.world
                        .break_block(args.position, None, BlockFlags::empty())
                        .await;
                }
            } else if new_state_id != old_state_id {
                args.world
                    .set_block_state(args.position, new_state_id, BlockFlags::NOTIFY_ALL)
                    .await;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::BlockState;
    use pumpkin_data::fluid::Fluid;
    use std::collections::HashMap;

    /// Minimal `BlockAccessor` mock for unit-testing `get_distance`/`is_bottom` without a live
    /// `World`. Unlisted positions default to air.
    #[derive(Default)]
    struct MockAccessor {
        blocks: HashMap<BlockPos, (&'static Block, BlockStateId)>,
    }

    impl MockAccessor {
        fn set(&mut self, pos: BlockPos, block: &'static Block, state_id: BlockStateId) {
            self.blocks.insert(pos, (block, state_id));
        }
    }

    impl BlockAccessor for MockAccessor {
        fn get_block(&self, position: &BlockPos) -> &'static Block {
            self.blocks
                .get(position)
                .map_or(&Block::AIR, |(block, _)| *block)
        }

        fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
            self.blocks
                .get(position)
                .map_or(Block::AIR.default_state, |(_, state_id)| {
                    BlockState::from_id(*state_id)
                })
        }

        fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
            self.blocks
                .get(position)
                .map_or(Block::AIR.default_state.id, |(_, state_id)| *state_id)
        }

        fn get_block_and_state(
            &self,
            position: &BlockPos,
        ) -> (&'static Block, &'static BlockState) {
            (self.get_block(position), self.get_block_state(position))
        }

        fn get_fluid(&self, _position: &BlockPos) -> Fluid {
            Fluid::EMPTY
        }
    }

    fn accessor_with(blocks: &[(BlockPos, &'static Block, BlockStateId)]) -> MockAccessor {
        let mut accessor = MockAccessor::default();
        for (pos, block, state_id) in blocks {
            accessor.set(*pos, block, *state_id);
        }
        accessor
    }

    fn scaffolding_state(distance: u8) -> BlockStateId {
        let mut props = ScaffoldingLikeProperties::default(&Block::SCAFFOLDING);
        props.r#distance = distance;
        props.to_state_id(&Block::SCAFFOLDING)
    }

    #[test]
    fn distance_zero_when_directly_supported() {
        let pos = BlockPos::new(0, 1, 0);
        let accessor = accessor_with(&[(pos.down(), &Block::STONE, Block::STONE.default_state.id)]);
        assert_eq!(ScaffoldingBlock::get_distance(&accessor, &pos), 0);
    }

    #[test]
    fn distance_seven_when_unsupported() {
        let pos = BlockPos::new(0, 1, 0);
        let accessor = accessor_with(&[(pos.down(), &Block::AIR, Block::AIR.default_state.id)]);
        assert_eq!(
            ScaffoldingBlock::get_distance(&accessor, &pos),
            MAX_DISTANCE
        );
    }

    #[test]
    fn distance_propagates_vertically_through_scaffolding() {
        let pos = BlockPos::new(0, 1, 0);
        let accessor = accessor_with(&[(pos.down(), &Block::SCAFFOLDING, scaffolding_state(3))]);
        assert_eq!(ScaffoldingBlock::get_distance(&accessor, &pos), 3);
    }

    #[test]
    fn distance_propagates_horizontally_plus_one() {
        let pos = BlockPos::new(0, 1, 0);
        let accessor = accessor_with(&[
            (pos.down(), &Block::AIR, Block::AIR.default_state.id),
            (
                pos.offset(BlockDirection::North.to_offset()),
                &Block::SCAFFOLDING,
                scaffolding_state(2),
            ),
        ]);
        assert_eq!(ScaffoldingBlock::get_distance(&accessor, &pos), 3);
    }

    #[test]
    fn distance_prefers_vertical_over_horizontal() {
        let pos = BlockPos::new(0, 1, 0);
        let accessor = accessor_with(&[
            (pos.down(), &Block::SCAFFOLDING, scaffolding_state(1)),
            (
                pos.offset(BlockDirection::North.to_offset()),
                &Block::SCAFFOLDING,
                scaffolding_state(0),
            ),
        ]);
        // Vertical gives 1 directly; horizontal would give 0+1=1 too, min is still 1.
        assert_eq!(ScaffoldingBlock::get_distance(&accessor, &pos), 1);
    }

    #[test]
    fn is_bottom_false_when_resting_on_scaffolding() {
        let pos = BlockPos::new(0, 1, 0);
        let accessor = accessor_with(&[(pos.down(), &Block::SCAFFOLDING, scaffolding_state(0))]);
        assert!(!ScaffoldingBlock::is_bottom(&accessor, &pos, 1));
    }

    #[test]
    fn is_bottom_true_when_floating_and_not_fully_supported() {
        let pos = BlockPos::new(0, 1, 0);
        let accessor = accessor_with(&[(pos.down(), &Block::AIR, Block::AIR.default_state.id)]);
        assert!(ScaffoldingBlock::is_bottom(&accessor, &pos, 3));
    }

    #[test]
    fn is_bottom_false_when_fully_supported() {
        let pos = BlockPos::new(0, 1, 0);
        let accessor = accessor_with(&[(pos.down(), &Block::AIR, Block::AIR.default_state.id)]);
        assert!(!ScaffoldingBlock::is_bottom(&accessor, &pos, 0));
    }
}
