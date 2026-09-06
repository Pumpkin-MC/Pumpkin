use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, block_properties::CactusLikeProperties, fluid::Fluid, tag};
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
            let side = block_pos.down().offset(direction.to_offset());
            let (block, state) = block_accessor.get_block_and_state(&side);
            // Vanilla `SugarCaneBlock.canSurvive` takes *either* side, and reads the fluid
            // tag off the fluid state, not off the block:
            //
            //     FluidState fluidState = level.getFluidState(blockPos);
            //     if (fluidState.is(FluidTags.SUPPORTS_SUGAR_CANE_ADJACENTLY)
            //         || blockState2.is(BlockTags.SUPPORTS_SUGAR_CANE_ADJACENTLY)) return true;
            //
            // `#minecraft:supports_sugar_cane_adjacently` is `#minecraft:water` as a fluid
            // tag and `{minecraft:frosted_ice}` as a block tag, so asking one block to carry
            // both (as this used to) can never hold: sugar cane never survived anywhere.
            let fluid_supports = if state.is_waterlogged() {
                Fluid::WATER.has_tag(&tag::Fluid::MINECRAFT_SUPPORTS_SUGAR_CANE_ADJACENTLY)
            } else {
                Fluid::from_state_id(state.id).is_some_and(|fluid| {
                    fluid.has_tag(&tag::Fluid::MINECRAFT_SUPPORTS_SUGAR_CANE_ADJACENTLY)
                })
            };
            if fluid_supports
                || block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_SUGAR_CANE_ADJACENTLY)
            {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use pumpkin_data::{Block, BlockState, BlockStateId};
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_world::world::BlockAccessor;

    use super::can_place_at;

    #[derive(Default)]
    struct Blocks(HashMap<BlockPos, &'static Block>);

    impl BlockAccessor for Blocks {
        fn get_block(&self, position: &BlockPos) -> &'static Block {
            self.0.get(position).copied().unwrap_or(&Block::AIR)
        }

        fn get_block_state(&self, position: &BlockPos) -> &'static BlockState {
            BlockState::from_id(self.get_block(position).default_state.id)
        }

        fn get_block_state_id(&self, position: &BlockPos) -> BlockStateId {
            self.get_block(position).default_state.id
        }

        fn get_block_and_state(
            &self,
            position: &BlockPos,
        ) -> (&'static Block, &'static BlockState) {
            (self.get_block(position), self.get_block_state(position))
        }
    }

    /// `minecraft:supports_sugar_cane_adjacently` is `#minecraft:water` as a *fluid* tag and
    /// `{minecraft:frosted_ice}` as a *block* tag; vanilla takes either.
    #[test]
    fn sand_next_to_water_supports_sugar_cane() {
        let pos = BlockPos::new(0, 64, 0);
        let mut world = Blocks::default();
        world.0.insert(pos.down(), &Block::SAND);

        assert!(
            !can_place_at(&world, &pos),
            "sand with nothing beside it must not support sugar cane"
        );

        world.0.insert(
            pos.down()
                .offset(pumpkin_util::math::vector3::Vector3::new(1, 0, 0)),
            &Block::WATER,
        );
        assert!(
            can_place_at(&world, &pos),
            "sand with water beside it must support sugar cane"
        );
    }

    #[test]
    fn frosted_ice_beside_the_soil_also_supports_sugar_cane() {
        let pos = BlockPos::new(0, 64, 0);
        let mut world = Blocks::default();
        world.0.insert(pos.down(), &Block::SAND);
        world.0.insert(
            pos.down()
                .offset(pumpkin_util::math::vector3::Vector3::new(0, 0, -1)),
            &Block::FROSTED_ICE,
        );
        assert!(can_place_at(&world, &pos));
    }

    #[test]
    fn sugar_cane_stacks_on_itself() {
        let pos = BlockPos::new(0, 64, 0);
        let mut world = Blocks::default();
        world.0.insert(pos.down(), &Block::SUGAR_CANE);
        assert!(can_place_at(&world, &pos));
    }

    #[test]
    fn stone_never_supports_sugar_cane() {
        let pos = BlockPos::new(0, 64, 0);
        let mut world = Blocks::default();
        world.0.insert(pos.down(), &Block::STONE);
        world.0.insert(
            pos.down()
                .offset(pumpkin_util::math::vector3::Vector3::new(1, 0, 0)),
            &Block::WATER,
        );
        assert!(!can_place_at(&world, &pos));
    }
}
