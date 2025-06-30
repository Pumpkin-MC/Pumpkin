use crate::block::BlockIsReplacing;
use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

type SkeletonSkullLikeProperties = pumpkin_data::block_properties::SkeletonSkullLikeProperties;

pub struct SkullBlock;

impl BlockMetadata for SkullBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[
            "skeleton_skull",
            "wither_skeleton_skull",
            "player_head",
            "zombie_head",
            "creeper_head",
            "piglin_head",
            "dragon_head",
        ]
    }
}

#[async_trait]
impl PumpkinBlock for SkullBlock {
    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        player: &Player,
        block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let mut props = SkeletonSkullLikeProperties::default(block);
        props.rotation = player.get_entity().get_rotation_16();
        props.powered = block_receives_redstone_power(world, block_pos).await;
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
        let mut props = SkeletonSkullLikeProperties::from_state_id(state.id, block);
        let is_receiving_power = block_receives_redstone_power(world, block_pos).await;
        if props.powered != is_receiving_power {
            props.powered = is_receiving_power;
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
