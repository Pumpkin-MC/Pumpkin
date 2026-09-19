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
        if !can_place_at(
            args.world.as_ref(),
            Some(args.world.as_ref()),
            args.position,
        ) {
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
        if !can_place_at(args.world, Some(args.world), args.position) {
            args.world
                .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
        }
        args.state_id
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.world, args.position)
    }
}

fn can_place_at(
    block_accessor: &dyn BlockAccessor,
    world: Option<&World>,
    block_pos: &BlockPos,
) -> bool {
    let block_below = block_accessor.get_block(&block_pos.down());

    if block_below == &Block::SUGAR_CANE {
        return true;
    }

    if block_below.has_tag(&tag::Block::MINECRAFT_SUPPORTS_SUGAR_CANE) {
        for direction in HorizontalFacing::all() {
            let adj_pos = block_pos.down().offset(direction.to_offset());
            let block = block_accessor.get_block(&adj_pos);

            let fluid_ok = world
                .map(|w| {
                    w.get_fluid(&adj_pos)
                        .has_tag(&tag::Fluid::MINECRAFT_SUPPORTS_SUGAR_CANE_ADJACENTLY)
                })
                .unwrap_or(false);

            let block_ok = block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_SUGAR_CANE_ADJACENTLY);

            if fluid_ok || block_ok {
                return true;
            }
        }
    }

    false
}
