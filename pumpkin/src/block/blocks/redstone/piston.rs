use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block::{Block, BlockProperties, Boolean, HorizontalFacing};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::{BlockDirection, HorizontalFacingExt};

use crate::{
    block::pumpkin_block::PumpkinBlock,
    server::Server,
    world::{BlockFlags, World},
};

use super::block_receives_redstone_power;

type PistonProps = pumpkin_data::block::StickyPistonLikeProperties;

#[pumpkin_block("minecraft:piston")]
pub struct PistonBlock;

#[async_trait]
impl PumpkinBlock for PistonBlock {
    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        block: &Block,
        _face: &BlockDirection,
        block_pos: &BlockPos,
        _use_item_on: &SUseItemOn,
        player_direction: &HorizontalFacing,
        _other: bool,
    ) -> u16 {
        let mut props = PistonProps::default(block);
        props.extended = Boolean::from_bool(block_receives_redstone_power(world, block_pos).await);
        props.facing = player_direction.to_block_direction().to_facing();
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
        let state = world.get_block_state(block_pos).await.unwrap();
        let mut props = PistonProps::from_state_id(state.id, block);
        let is_receiving_power = block_receives_redstone_power(world, block_pos).await;

        if is_receiving_power {
            props.extended = props.extended.flip();
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
