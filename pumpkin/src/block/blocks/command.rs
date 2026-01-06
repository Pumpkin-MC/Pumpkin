use std::sync::{Arc, atomic::Ordering};

use log::warn;
use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, CommandBlockLikeProperties, Facing},
};
use pumpkin_util::{
    GameMode, PermissionLvl,
    math::{position::BlockPos, vector3::Vector3},
};
use pumpkin_world::{
    BlockStateId,
    block::entities::{BlockEntity, command_block::CommandBlockEntity},
    tick::TickPriority,
};

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, NormalUseArgs,
        OnNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
        registry::BlockActionResult,
    },
    server::Server,
    world::World,
};

use super::redstone::block_receives_redstone_power;

pub struct CommandBlock;

impl CommandBlock {
    async fn get_relative_facing(
        world: &World,
        pos: &BlockPos,
        dir: Facing,
    ) -> Option<(BlockPos, CommandBlockLikeProperties)> {
        let offset = Self::facing_to_offset(dir);
        let target_pos = pos.offset(offset);
        let block = world.get_block(&target_pos).await;

        let allowed_blocks = [
            Block::COMMAND_BLOCK.name,
            Block::CHAIN_COMMAND_BLOCK.name,
            Block::REPEATING_COMMAND_BLOCK.name,
        ];
        if !allowed_blocks.contains(&block.name) {
            return None;
        }

        let state_id = world.get_block_state_id(&target_pos).await;
        let props = CommandBlockLikeProperties::from_state_id(state_id, block);

        Some((target_pos, props))
    }

    /// Convert a [Facing] into a [Vector3] one block forward in the direction of `facing`
    fn facing_to_offset(facing: Facing) -> Vector3<i32> {
        match facing {
            Facing::North => Vector3::new(0, 0, -1),
            Facing::South => Vector3::new(0, 0, 1),
            Facing::East => Vector3::new(1, 0, 0),
            Facing::West => Vector3::new(-1, 0, 0),
            Facing::Up => Vector3::new(0, 1, 0),
            Facing::Down => Vector3::new(0, -1, 0),
        }
    }

    async fn conditions_met(world: &Arc<World>, pos: &BlockPos, facing: Facing) -> bool {
        let state_id = world.get_block_state_id(pos).await;
        let block = world.get_block(pos).await;
        let props = CommandBlockLikeProperties::from_state_id(state_id, block);

        if !props.conditional {
            return true;
        }

        let Some(before) = Self::get_relative_facing(world, pos, facing.opposite()).await else {
            return false;
        };
        let Some(before_entity) = world.get_block_entity(&before.0).await else {
            warn!("Command block has no matching entity");
            return false;
        };
        let command_entity: &CommandBlockEntity = before_entity.as_any().downcast_ref().unwrap();

        command_entity.success_count.load(Ordering::Relaxed) > 0
    }

    async fn update(
        world: &World,
        block: &Block,
        command_block: &CommandBlockEntity,
        pos: &BlockPos,
        powered: bool,
    ) {
        let is_auto = command_block.auto.load(Ordering::Relaxed);
        if command_block.powered.load(Ordering::Relaxed) == powered && !is_auto {
            return;
        }
        command_block.powered.store(powered, Ordering::Relaxed);

        if block.id == Block::CHAIN_COMMAND_BLOCK.id || is_auto || !powered {
            return;
        }

        let state_id = world.get_block_state_id(pos).await;
        let props = CommandBlockLikeProperties::from_state_id(state_id, block);

        if !props.conditional {
            world
                .schedule_block_tick(block, *pos, 1, TickPriority::Normal)
                .await;
            return;
        }

        let Some(behind) = Self::get_relative_facing(world, pos, props.facing.opposite()).await
        else {
            return;
        };
        let Some(behind_entity) = world.get_block_entity(&behind.0).await else {
            warn!(
                "Command Block exists at {} with no matching block entity!",
                behind.0
            );
            return;
        };
        let behind_entity: &CommandBlockEntity = behind_entity
            .as_any()
            .downcast_ref()
            .expect("behind should always be a command block");

        if behind_entity.success_count.load(Ordering::Relaxed) > 0 {
            world
                .schedule_block_tick(block, *pos, 1, TickPriority::Normal)
                .await;
        }
    }

    async fn execute(
        server: &Arc<Server>,
        world: Arc<World>,
        block_entity: Arc<dyn BlockEntity>,
        command: &str,
    ) {
        let command_entity: &CommandBlockEntity = block_entity.as_any().downcast_ref().unwrap();
        if command.is_empty() {
            command_entity.success_count.store(0, Ordering::Release);
        } else {
            if command == "Searge" && command_entity.track_output.load(Ordering::Relaxed) {
                let mut last_output = command_entity.last_output.lock().await;
                *last_output = "#itzlipofutzli".to_string();
                return;
            }

            server
                .command_dispatcher
                .read()
                .await
                .handle_command(
                    &crate::command::CommandSender::CommandBlock(block_entity, world),
                    server,
                    command,
                )
                .await;
        }
    }

    async fn chain_execute(
        server: &Arc<Server>,
        world: Arc<World>,
        start: BlockPos,
        direction: Facing,
    ) {
        let mut i = u16::MAX;
        let mut pos = start;

        while i > 0 {
            let block = world.get_block(&pos).await;

            if block.id != Block::CHAIN_COMMAND_BLOCK.id {
                break;
            }
            let Some(block_entity) = world.get_block_entity(&pos).await else {
                warn!("Missing command block entity");
                break;
            };

            let command_entity: &CommandBlockEntity = block_entity.as_any().downcast_ref().unwrap();
            let powered = command_entity.powered.load(Ordering::Relaxed);
            let auto = command_entity.auto.load(Ordering::Relaxed);
            let state_id = world.get_block_state_id(&pos).await;
            let props = CommandBlockLikeProperties::from_state_id(state_id, block);

            if powered || auto {
                let conditions_met = Self::conditions_met(&world, &pos, direction).await;
                if conditions_met {
                    let command = command_entity.command.lock().await;
                    let entity = world.get_block_entity(&pos).await.unwrap();
                    Self::execute(server, world.clone(), entity, &command).await;
                } else if props.conditional {
                    command_entity.success_count.store(0, Ordering::Release);
                }
            }

            pos = pos.offset(Self::facing_to_offset(direction));

            i -= 1;
            if i == 0 {
                warn!(
                    "Command block chain executed {} times (the maximum)!",
                    u16::MAX
                );
            }
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

impl BlockBehaviour for CommandBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = CommandBlockLikeProperties::default(args.block);
            props.facing = args.player.living_entity.entity.get_facing().opposite();
            props.to_state_id(args.block)
        })
    }

    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if args.player.permission_lvl.load() < PermissionLvl::Two {
                return BlockActionResult::Pass;
            }
            let Some(block_entity) = args.world.get_block_entity(args.position).await else {
                return BlockActionResult::Pass;
            };
            args.world.update_block_entity(&block_entity).await;
            BlockActionResult::SuccessServer
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
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
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
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
            let props = CommandBlockLikeProperties::from_state_id(
                args.world.get_block_state_id(args.position).await,
                args.block,
            );

            Self::execute(
                &server,
                args.world.clone(),
                block_entity.clone(),
                &command_entity.command.lock().await,
            )
            .await;

            Self::chain_execute(
                &server,
                args.world.clone(),
                args.position.offset(Self::facing_to_offset(props.facing)),
                props.facing,
            )
            .await;

            let block = args.world.get_block(args.position).await;
            if block == &Block::REPEATING_COMMAND_BLOCK {
                args.world
                    .schedule_block_tick(block, *args.position, 1, TickPriority::Normal)
                    .await;
            }
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            if let Some(player) = args.player
                && player.gamemode.load() == GameMode::Creative
            {
                return true;
            }

            false
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let send_command_feedback = {
                let game_rules = &args.world.level_info.read().await.game_rules;
                game_rules.send_command_feedback
            };

            let entity = CommandBlockEntity::new(
                *args.position,
                send_command_feedback,
                args.block.id == Block::CHAIN_COMMAND_BLOCK.id,
            );
            args.world.add_block_entity(Arc::new(entity)).await;
        })
    }

    fn get_comparator_output<'a>(
        &'a self,
        args: crate::block::GetComparatorOutputArgs<'a>,
    ) -> BlockFuture<'a, Option<u8>> {
        Box::pin(async {
            let entity = args.world.get_block_entity(args.position).await;

            entity.map_or_else(
                || {
                    warn!("Command block is missing its corresponding block entity");
                    None
                },
                |entity| {
                    let command_block_entity: &CommandBlockEntity =
                        entity.as_any().downcast_ref().expect(
                            "Block entity command block's position should be a matching entity",
                        );
                    Some(command_block_entity.success_count.load(Ordering::Acquire) as u8)
                },
            )
        })
    }
}
