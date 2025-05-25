use std::sync::Arc;

use crate::{block::BlockIsReplacing, entity::player::Player};
use async_trait::async_trait;
use pumpkin_data::{Block, BlockDirection, block_properties::BlockProperties};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{BlockStateId, chunk::TickPriority, world::BlockFlags};

use crate::{block::pumpkin_block::PumpkinBlock, server::Server, world::World};

use super::block_receives_redstone_power;

type RedstoneLampProperties = pumpkin_data::block_properties::RedstoneOreLikeProperties;

#[pumpkin_block("minecraft:redstone_lamp")]
pub struct RedstoneLamp;

#[async_trait]
impl PumpkinBlock for RedstoneLamp {
    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        _player: &Player,
        block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let mut props = RedstoneLampProperties::default(block);
        props.lit = block_receives_redstone_power(world, block_pos).await;
        props.to_state_id(block)
    }

    async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        block_pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
        let state = world.get_block_state(block_pos).await;
        let mut props = RedstoneLampProperties::from_state_id(state.id, block);
        let is_lit = props.lit;
        let is_receiving_power = block_receives_redstone_power(world, block_pos).await;

        if is_lit != is_receiving_power {
            if is_lit {
                world
                    .schedule_block_tick(block, *block_pos, 4, TickPriority::Normal)
                    .await;
            } else {
                props.lit = !props.lit;
                world
                    .set_block_state(
                        block_pos,
                        props.to_state_id(block),
                        BlockFlags::NOTIFY_LISTENERS,
                    )
                    .await;
            }
        }
    }

    async fn on_scheduled_tick(&self, world: &Arc<World>, block: &Block, block_pos: &BlockPos) {
        let state = world.get_block_state(block_pos).await;
        let mut props = RedstoneLampProperties::from_state_id(state.id, block);
        let is_lit = props.lit;
        let is_receiving_power = block_receives_redstone_power(world, block_pos).await;

        if is_lit && !is_receiving_power {
            props.lit = !props.lit;
            world
                .set_block_state(
                    block_pos,
                    props.to_state_id(block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
        }
    }
}
