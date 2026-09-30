use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::fluid::Fluid;
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
            let neighbor_pos = block_pos.down().offset(direction.to_offset());
            let block = block_accessor.get_block(&neighbor_pos);
            let state = block_accessor.get_block_state(&neighbor_pos);

            let is_water = state.is_waterlogged()
                || Fluid::from_state_id(state.id).is_some_and(|fluid| {
                    fluid.has_tag(&tag::Fluid::MINECRAFT_SUPPORTS_SUGAR_CANE_ADJACENTLY)
                });
            if is_water || block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_SUGAR_CANE_ADJACENTLY) {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::BlockState;
    use pumpkin_data::block_properties::WhiteWoolSlabLikeProperties;
    use pumpkin_world::world::BlockAccessor;

    #[derive(Default)]
    struct TestWorld {
        entries: Vec<(BlockPos, &'static Block, &'static BlockState)>,
    }

    impl TestWorld {
        fn set_block(&mut self, pos: BlockPos, block: &'static Block) {
            self.entries.push((pos, block, block.default_state));
        }

        fn set_state(&mut self, pos: BlockPos, block: &'static Block, state: &'static BlockState) {
            self.entries.push((pos, block, state));
        }
    }

    impl BlockAccessor for TestWorld {
        fn get_block(&self, position: &BlockPos) -> &'static Block {
            self.entries
                .iter()
                .find(|(pos, _, _)| pos == position)
                .map_or(&Block::AIR, |(_, block, _)| *block)
        }

        fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
            self.entries
                .iter()
                .find(|(pos, _, _)| pos == position)
                .map_or(Block::AIR.default_state, |(_, _, state)| *state)
        }

        fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
            self.get_block_state(position).id
        }

        fn get_block_and_state(
            &self,
            position: &BlockPos,
        ) -> (&'static Block, &'static BlockState) {
            self.entries
                .iter()
                .find(|(pos, _, _)| pos == position)
                .map_or(
                    (&Block::AIR, Block::AIR.default_state),
                    |(_, block, state)| (*block, *state),
                )
        }
    }

    fn world_with_support() -> (BlockPos, TestWorld) {
        let pos = BlockPos::new(0, 64, 0);
        let mut world = TestWorld::default();
        world.set_block(pos.down(), &Block::DIRT);
        (pos, world)
    }

    #[test]
    fn placeable_next_to_water() {
        use pumpkin_data::block_properties::WaterLikeProperties;

        let (pos, mut world) = world_with_support();
        world.set_block(
            pos.down().offset(HorizontalFacing::North.to_offset()),
            &Block::WATER,
        );
        assert!(can_place_at(&world, &pos));

        let (pos, mut world) = world_with_support();
        let mut props = WaterLikeProperties::default(&Block::WATER);
        props.r#level = 1;
        let state = BlockState::from_id(props.to_state_id(&Block::WATER));
        assert!(Fluid::from_state_id(state.id).is_some());
        world.set_state(
            pos.down().offset(HorizontalFacing::East.to_offset()),
            &Block::WATER,
            state,
        );
        assert!(can_place_at(&world, &pos));
    }

    #[test]
    fn placeable_next_to_waterlogged_or_frosted_blocks() {
        let (pos, mut world) = world_with_support();
        let mut props = WhiteWoolSlabLikeProperties::default(&Block::OAK_SLAB);
        props.waterlogged = true;
        let state = BlockState::from_id(props.to_state_id(&Block::OAK_SLAB));
        assert!(state.is_waterlogged());
        world.set_state(
            pos.down().offset(HorizontalFacing::South.to_offset()),
            &Block::OAK_SLAB,
            state,
        );
        assert!(can_place_at(&world, &pos));

        let (pos, mut world) = world_with_support();
        world.set_block(
            pos.down().offset(HorizontalFacing::West.to_offset()),
            &Block::FROSTED_ICE,
        );
        assert!(can_place_at(&world, &pos));
    }

    #[test]
    fn needs_support_below_and_water_beside() {
        let (pos, mut world) = world_with_support();
        world.set_block(
            pos.down().offset(HorizontalFacing::North.to_offset()),
            &Block::STONE,
        );
        assert!(!can_place_at(&world, &pos));

        let pos = BlockPos::new(0, 64, 0);
        let mut world = TestWorld::default();
        world.set_block(pos.down(), &Block::STONE);
        world.set_block(
            pos.down().offset(HorizontalFacing::North.to_offset()),
            &Block::WATER,
        );
        assert!(!can_place_at(&world, &pos));
    }

    #[test]
    fn placeable_on_existing_cane() {
        let pos = BlockPos::new(0, 64, 0);
        let mut world = TestWorld::default();
        world.set_block(pos.down(), &Block::SUGAR_CANE);
        assert!(can_place_at(&world, &pos));
    }
}
