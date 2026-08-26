use crate::{
    block::{
        BlockBehaviour, BlockMetadata, GetStateForNeighborUpdateArgs, OnScheduledTickArgs,
        PlacedArgs,
    },
    entity::falling::FallingEntity,
};
use pumpkin_data::{
    Block, BlockId, BlockState, BlockStateId,
    tag::{self, Taggable},
};
use pumpkin_world::tick::TickPriority;
pub struct FallingBlock;

impl FallingBlock {
    #[must_use]
    pub fn can_fall_through(state: &BlockState, block: &Block) -> bool {
        state.is_air()
            || block.has_tag(&tag::Block::MINECRAFT_FIRE)
            || state.is_liquid()
            || state.replaceable()
    }
}

impl BlockMetadata for FallingBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::GRAVEL, BlockId::SAND, BlockId::RED_SAND].into()
    }
}

impl BlockBehaviour for FallingBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        {
            // TODO: make delay configurable
            args.world
                .schedule_block_tick(args.block, *args.position, 2, TickPriority::Normal);
        }
    }
    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        // TODO: make delay configurable
        args.world
            .schedule_block_tick(args.block, *args.position, 2, TickPriority::Normal);
        args.state_id
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let (block, state) = args.world.get_block_and_state(&args.position.down());
        if !Self::can_fall_through(state, block) || args.position.0.y < args.world.min_y {
            return;
        }
        let state = args.world.get_block_state(args.position);
        FallingEntity::replace_spawn(args.world, *args.position, state.id);
    }
}
