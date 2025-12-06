use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockMetadata, GetStateForNeighborUpdateArgs,
        OnScheduledTickArgs, PlacedArgs,
    },
    entity::falling::FallingEntity,
};
use pumpkin_data::{
    Block, BlockState,
    tag::{self, Taggable},
};
use pumpkin_world::{BlockStateId, tick::TickPriority};
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
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[Block::GRAVEL.name, Block::SAND.name, Block::RED_SAND.name]
    }
}

impl BlockBehaviour for FallingBlock {
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // TODO: make delay configurable
            args.world
                .schedule_block_tick(args.block, *args.position, 2, TickPriority::Normal)
                .await;
        })
    }
    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            // TODO: make delay configurable
            args.world
                .schedule_block_tick(args.block, *args.position, 2, TickPriority::Normal)
                .await;
            args.state_id
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let (block, state) = args.world.get_block_and_state(&args.position.down()).await;
            if !Self::can_fall_through(state, block) || args.position.0.y < args.world.min_y {
                return;
            }
            FallingEntity::replace_spawn(args.world, *args.position, args.block.default_state.id)
                .await;
        })
    }
}
