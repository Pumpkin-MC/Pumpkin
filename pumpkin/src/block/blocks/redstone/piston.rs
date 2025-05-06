use std::sync::Arc;

use crate::{block::BlockIsReplacing, entity::player::Player};
use async_trait::async_trait;
use pumpkin_data::{Block, block_properties::BlockProperties};
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, block::BlockDirection};

use crate::{
    block::pumpkin_block::{BlockMetadata, PumpkinBlock},
    server::Server,
    world::{BlockFlags, World},
};

use super::block_receives_redstone_power;

type PistonProps = pumpkin_data::block_properties::StickyPistonLikeProperties;

pub struct PistonBlock;

impl BlockMetadata for PistonBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[Block::PISTON.name, Block::STICKY_PISTON.name]
    }
}

#[async_trait]
impl PumpkinBlock for PistonBlock {
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        player: &Player,
        block: &Block,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        _replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let mut props = PistonProps::default(block);
        props.extended = false;
        props.facing = player.living_entity.entity.get_facing().opposite();
        props.to_state_id(block)
    }

    async fn placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        pos: &BlockPos,
        old_state_id: BlockStateId,
        _notify: bool,
    ) {
        if old_state_id == state_id {
            return;
        }
        try_move(world, block, pos).await;
    }

    async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        block_pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
        try_move(world, block, block_pos).await;
    }
}

async fn try_move(world: &Arc<World>, block: &Block, block_pos: &BlockPos) {
    let state = world.get_block_state(block_pos).await.unwrap();
    let mut props = PistonProps::from_state_id(state.id, block);
    let is_receiving_power = block_receives_redstone_power(world, block_pos).await;

    if is_receiving_power {
        props.extended = !props.extended;
        world
            .set_block_state(
                block_pos,
                props.to_state_id(block),
                BlockFlags::NOTIFY_LISTENERS,
            )
            .await;
    }
}
