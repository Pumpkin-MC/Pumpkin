use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, block_properties::CactusLikeProperties, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

use crate::block::{
    BlockBehaviour, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnScheduledTickArgs,
    RandomTickArgs,
};
use crate::world::World;

#[pumpkin_block("minecraft:sugar_cane")]
pub struct SugarCaneBlock;

impl BlockBehaviour for SugarCaneBlock {
    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !can_place_at(args.world.as_ref(), args.position) {
            args.world
                .break_block(args.position, None, BlockFlags::NOTIFY_ALL);
        }
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        if args.world.get_block_state(&args.position.up()).is_air()
            && !(args.world.get_block(&args.position.down()) == &Block::SUGAR_CANE
                && args.world.get_block(&args.position.down().down()) == &Block::SUGAR_CANE)
        {
            let state_id = args.world.get_block_state(args.position).id;
            let age = CactusLikeProperties::from_state_id(state_id).age;
            if age == 15 {
                args.world
                    .set_block_state(&args.position.up(), state_id, BlockFlags::NOTIFY_ALL);
                let props = CactusLikeProperties { age: 0 };
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS,
                );
            } else {
                let props = CactusLikeProperties { age: age + 1 };
                args.world.set_block_state(
                    args.position,
                    props.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS,
                );
            }
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !can_place_at(args.world, args.position) {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
        }
        args.state_id
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.position)
    }
}

fn can_place_at(block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    let block_below = block_accessor.get_block(&block_pos.down());

    if block_below == &Block::SUGAR_CANE {
        return true;
    }

    if block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_SUGAR_CANE) {
        for direction in HorizontalFacing::all() {
            let adj_pos = block_pos.down().offset(direction.to_offset());
            // During world generation, adjacent water is in the generation cache.
            let state_id = block_accessor.get_block_state_id(&adj_pos);
            let block = state_id.to_block();
            let fluid_ok = World::get_fluid_from_state_id(state_id)
                .has_tag(&tag::Fluid::MINECRAFT_SUPPORTS_SUGAR_CANE_ADJACENTLY);

            let block_ok = block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_SUGAR_CANE_ADJACENTLY);

            if fluid_ok || block_ok {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use pumpkin_data::BlockState;
    use rustc_hash::FxHashMap;

    use super::*;

    #[derive(Default)]
    struct TestBlocks(FxHashMap<BlockPos, BlockStateId>);

    impl BlockAccessor for TestBlocks {
        fn get_block(&self, position: &BlockPos) -> &'static Block {
            self.get_block_state_id(position).to_block()
        }

        fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
            BlockState::from_id(self.get_block_state_id(position))
        }

        fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
            self.0
                .get(position)
                .copied()
                .unwrap_or(Block::AIR.default_state.id)
        }

        fn get_block_and_state(
            &self,
            position: &BlockPos,
        ) -> (&'static Block, &'static BlockState) {
            BlockState::from_id_with_block(self.get_block_state_id(position))
        }
    }

    fn assert_placement_and_survival(adjacent: &Block, fluid: bool, block: bool, expected: bool) {
        let adjacent_state = adjacent.default_state.id;
        assert_eq!(
            World::get_fluid_from_state_id(adjacent_state)
                .has_tag(&tag::Fluid::MINECRAFT_SUPPORTS_SUGAR_CANE_ADJACENTLY),
            fluid
        );
        assert_eq!(
            adjacent.has_tag(&tag::Block::MINECRAFT_SUPPORTS_SUGAR_CANE_ADJACENTLY),
            block
        );

        for direction in HorizontalFacing::all() {
            let position = BlockPos::new(0, 64, 0);
            let mut blocks = TestBlocks::default();
            blocks
                .0
                .insert(position.down(), Block::SAND.default_state.id);
            blocks.0.insert(
                position.down().offset(direction.to_offset()),
                adjacent_state,
            );

            assert_eq!(
                SugarCaneBlock.can_place_at(CanPlaceAtArgs {
                    server: None,
                    world: None,
                    block_accessor: &blocks,
                    block: &Block::SUGAR_CANE,
                    state: Block::SUGAR_CANE.default_state,
                    position: &position,
                    direction: None,
                    player: None,
                    use_item_on: None,
                }),
                expected,
                "placement"
            );

            blocks
                .0
                .insert(position, Block::SUGAR_CANE.default_state.id);
            // Both neighbor updates and scheduled ticks use this survival predicate.
            assert_eq!(can_place_at(&blocks, &position), expected, "survival");
        }
    }

    #[test]
    fn fluid_only_support_allows_placement_and_survival() {
        assert_placement_and_survival(&Block::WATER, true, false, true);
    }

    #[test]
    fn block_only_support_allows_placement_and_survival() {
        assert_placement_and_survival(&Block::FROSTED_ICE, false, true, true);
    }

    #[test]
    fn no_adjacent_support_rejects_placement_and_survival() {
        assert_placement_and_survival(&Block::AIR, false, false, false);
    }
}
