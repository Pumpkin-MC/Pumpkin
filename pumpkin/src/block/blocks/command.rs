use std::sync::{Arc, atomic::Ordering};

use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_util::{GameMode, math::position::BlockPos};
use pumpkin_world::{
    block::entities::{BlockEntity, command_block::CommandBlockEntity},
    chunk::TickPriority,
};

use crate::{
    block::pumpkin_block::{BlockMetadata, PumpkinBlock},
    world::World,
};

use super::redstone::block_receives_redstone_power;

pub struct CommandBlock;

impl CommandBlock {
    pub async fn update(
        world: &World,
        block: &Block,
        command_block: CommandBlockEntity,
        pos: &BlockPos,
        powered: bool,
    ) {
        if command_block.powered.load(Ordering::Relaxed) == powered {
            return;
        }
        command_block.powered.store(powered, Ordering::Relaxed);
        if powered {
            world
                .schedule_block_tick(block, *pos, 1, TickPriority::Normal)
                .await;
        }
    }
}

impl BlockMetadata for CommandBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        &[
            Block::COMMAND_BLOCK.name,
            Block::CHAIN_COMMAND_BLOCK.name,
            Block::REPEATING_COMMAND_BLOCK.name,
        ]
    }
}

#[async_trait]
impl PumpkinBlock for CommandBlock {
    async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
        if let Some((nbt, block_entity)) = world.get_block_entity(pos).await {
            let command_entity = CommandBlockEntity::from_nbt(&nbt, *pos);

            if block_entity.identifier() != command_entity.identifier() {
                return;
            }
            Self::update(
                world,
                block,
                command_entity,
                pos,
                block_receives_redstone_power(world, pos).await,
            )
            .await;
        }
    }

    async fn on_scheduled_tick(&self, world: &Arc<World>, _block: &Block, pos: &BlockPos) {
        if let Some((nbt, block_entity)) = world.get_block_entity(pos).await {
            let command_entity = CommandBlockEntity::from_nbt(&nbt, *pos);

            if block_entity.identifier() != command_entity.identifier() {
                return;
            }
            // TODO
        }
    }

    async fn can_place_at(
        &self,
        _server: Option<&crate::server::Server>,
        _world: Option<&World>,
        _block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        player: Option<&crate::entity::player::Player>,
        _block: &Block,
        _block_pos: &BlockPos,
        _face: pumpkin_data::BlockDirection,
        _use_item_on: Option<&pumpkin_protocol::server::play::SUseItemOn>,
    ) -> bool {
        if let Some(player) = player {
            if player.gamemode.load() == GameMode::Creative {
                return true;
            }
        }

        return false;
    }
}
