use std::sync::atomic::Ordering;

use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_util::{GameMode, math::position::BlockPos};
use pumpkin_world::{
    block::entities::{BlockEntity, command_block::CommandBlockEntity},
    chunk::TickPriority,
};

use crate::{
    block::pumpkin_block::{
        BlockMetadata, CanPlaceAtArgs, OnNeighborUpdateArgs, OnScheduledTickArgs, PumpkinBlock,
    },
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
    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        if let Some((nbt, block_entity)) = args.world.get_block_entity(args.location).await {
            let command_entity = CommandBlockEntity::from_nbt(&nbt, *args.location);

            if block_entity.resource_location() != command_entity.resource_location() {
                return;
            }
            Self::update(
                args.world,
                args.block,
                command_entity,
                args.location,
                block_receives_redstone_power(args.world, args.location).await,
            )
            .await;
        }
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if let Some((nbt, block_entity)) = args.world.get_block_entity(args.location).await {
            let command_entity = CommandBlockEntity::from_nbt(&nbt, *args.location);

            if block_entity.resource_location() != command_entity.resource_location() {
                return;
            }
            // TODO
        }
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        if let Some(player) = args.player {
            if player.gamemode.load() == GameMode::Creative {
                return true;
            }
        }

        false
    }
}
