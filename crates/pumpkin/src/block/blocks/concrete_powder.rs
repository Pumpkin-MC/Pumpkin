use pumpkin_data::tag::{Fluid, Taggable};
use pumpkin_data::{Block, BlockStateId};
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::block::blocks::coral::scan_for_water;
use crate::block::blocks::falling::FallingBlock;
use crate::block::{
    BlockBehaviour, GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
};
use crate::world::World;

#[pumpkin_block_from_tag("minecraft:concrete_powders")]
pub struct ConcretePowderBlock;

fn concrete_for(block: &Block) -> Option<&'static Block> {
    Block::from_name(block.name.strip_suffix("_powder")?)
}

fn should_solidify(world: &World, pos: &BlockPos) -> bool {
    world.get_fluid(pos).has_tag(&Fluid::MINECRAFT_WATER) || scan_for_water(world, pos)
}

impl BlockBehaviour for ConcretePowderBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if should_solidify(args.world, args.position)
            && let Some(concrete) = concrete_for(args.block)
        {
            return concrete.default_state.id;
        }
        args.block.default_state.id
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        if should_solidify(args.world, args.position)
            && let Some(concrete) = concrete_for(args.block)
        {
            args.world.set_block_state(
                args.position,
                concrete.default_state.id,
                BlockFlags::NOTIFY_ALL,
            );
            return;
        }
        FallingBlock::placed(&FallingBlock, args);
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if should_solidify(args.world, args.position)
            && let Some(concrete) = concrete_for(args.block)
        {
            return concrete.default_state.id;
        }
        FallingBlock::get_state_for_neighbor_update(&FallingBlock, args)
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        FallingBlock::on_scheduled_tick(&FallingBlock, args);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::{BlockId, tag};

    #[test]
    fn every_powder_solidifies_to_concrete() {
        for id in tag::Block::MINECRAFT_CONCRETE_POWDERS.1 {
            let powder = BlockId::new_or_air(*id).to_block();
            let concrete = concrete_for(powder).expect(powder.name);
            assert_ne!(concrete.id, powder.id);
            assert!(concrete.name.ends_with("concrete"));
        }
    }
}
