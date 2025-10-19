use crate::{
    block::{
        BlockBehaviour, BlockMetadata, GetStateForNeighborUpdateArgs, OnScheduledTickArgs,
        PlacedArgs,
    },
    entity::falling::FallingEntity,
};
use async_trait::async_trait;
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

#[async_trait]
impl BlockBehaviour for FallingBlock {
    async fn placed(&self, args: PlacedArgs<'_>) {
        // TODO: make delay configurable
        args.world
            .schedule_block_tick(args.block, *args.position, 2, TickPriority::Normal)
            .await;
    }
    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        // TODO: make delay configurable
        args.world
            .schedule_block_tick(args.block, *args.position, 2, TickPriority::Normal)
            .await;
        args.state_id
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let (block, state) = args.world.get_block_and_state(&args.position.down()).await;
        if !Self::can_fall_through(state, block) || args.position.0.y < args.world.min_y {
            return;
        }
        FallingEntity::replace_spawn(args.world, *args.position, args.block.default_state.id).await;
    }
}
