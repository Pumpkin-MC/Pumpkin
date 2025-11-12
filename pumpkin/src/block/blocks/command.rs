use std::sync::{Arc, atomic::Ordering};

use async_trait::async_trait;
use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, CommandBlockLikeProperties},
};
use pumpkin_util::{GameMode, PermissionLvl, math::position::BlockPos};
use pumpkin_world::{
    BlockStateId, block::entities::command_block::CommandBlockEntity, tick::TickPriority,
};

use crate::{
    block::{
        BlockBehaviour, BlockMetadata, CanPlaceAtArgs, NormalUseArgs, OnNeighborUpdateArgs,
        OnPlaceArgs, OnScheduledTickArgs, PlacedArgs, registry::BlockActionResult,
    },
    world::World,
};

use super::redstone::block_receives_redstone_power;

// todo: chaining command blocks
// conditional execution
pub struct CommandBlock;

impl CommandBlock {
    pub async fn update(
        world: &World,
        block: &Block,
        command_block: &CommandBlockEntity,
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
impl BlockBehaviour for CommandBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = CommandBlockLikeProperties::default(args.block);
        props.facing = args.player.living_entity.entity.get_facing().opposite();
        props.to_state_id(args.block)
    }

    async fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        if args.player.permission_lvl.load() < PermissionLvl::Two {
            return BlockActionResult::Pass;
        }
        let Some(block_entity) = args.world.get_block_entity(args.position).await else {
            return BlockActionResult::Pass;
        };
        args.world.update_block_entity(&block_entity).await;
        BlockActionResult::SuccessServer
    }

    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        if let Some(block_entity) = args.world.get_block_entity(args.position).await {
            if block_entity.resource_location() != CommandBlockEntity::ID {
                return;
            }
            let command_entity = block_entity
                .as_any()
                .downcast_ref::<CommandBlockEntity>()
                .unwrap();

            Self::update(
                args.world,
                args.block,
                command_entity,
                args.position,
                block_receives_redstone_power(args.world, args.position).await,
            )
            .await;
        }
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let Some(block_entity) = args.world.get_block_entity(args.position).await else {
            return;
        };
        if block_entity.resource_location() != CommandBlockEntity::ID {
            return;
        }

        let command_entity: &CommandBlockEntity = block_entity.as_any().downcast_ref().unwrap();

        let Some(server) = args.world.server.upgrade() else {
            return;
        };

        let _props = CommandBlockLikeProperties::from_state_id(
            args.world.get_block_state_id(args.position).await,
            args.block,
        );

        server
            .command_dispatcher
            .read()
            .await
            .handle_command(
                &mut crate::command::CommandSender::CommandBlock(
                    block_entity.clone(),
                    args.world.clone(),
                ),
                &server,
                &command_entity.command.lock().await,
            )
            .await;

        let block = args.world.get_block(args.position).await;
        if block == &Block::REPEATING_COMMAND_BLOCK {
            if !command_entity.auto.load(Ordering::SeqCst)
                && !command_entity.powered.load(Ordering::SeqCst)
            {
                return;
            }
            args.world
                .schedule_block_tick(block, *args.position, 1, TickPriority::Normal)
                .await;
        }
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        if let Some(player) = args.player
            && player.gamemode.load() == GameMode::Creative
        {
            return true;
        }

        false
    }

    async fn placed(&self, args: PlacedArgs<'_>) {
        let entity = CommandBlockEntity::new(*args.position);
        args.world.add_block_entity(Arc::new(entity)).await;
    }
}
