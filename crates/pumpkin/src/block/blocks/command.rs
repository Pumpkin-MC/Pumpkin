use std::sync::{Arc, atomic::Ordering};

use super::redstone::block_receives_redstone_power;
use crate::block::entities::{BlockEntity, command_block::CommandBlockEntity};
use crate::command::CommandSender;
use crate::entity::EntityBase;
use crate::{
    block::{
        BlockBehaviour, BlockMetadata, CanPlaceAtArgs, NormalUseArgs, OnNeighborUpdateArgs,
        OnPlaceArgs, OnScheduledTickArgs, PlacedArgs, registry::BlockActionResult,
    },
    server::Server,
    world::World,
};

use pumpkin_data::block_properties::{CommandBlockLikeProperties, Facing};
use pumpkin_data::{Block, BlockId, BlockState, BlockStateId, FacingExt, Rotation};

use pumpkin_util::{GameMode, PermissionLvl, math::position::BlockPos};
use pumpkin_world::tick::TickPriority;
use tracing::warn;

pub struct CommandBlock;

impl CommandBlock {
    fn get_relative_facing(
        world: &World,
        pos: &BlockPos,
        dir: Facing,
    ) -> Option<(BlockPos, CommandBlockLikeProperties)> {
        let target_pos = pos.offset(dir.to_block_direction().to_offset());
        let (block, state_id) = world.get_block_and_state_id(&target_pos);
        if !matches!(
            block.id,
            BlockId::COMMAND_BLOCK
                | BlockId::CHAIN_COMMAND_BLOCK
                | BlockId::REPEATING_COMMAND_BLOCK
        ) {
            return None;
        }

        let props = CommandBlockLikeProperties::from_state_id(state_id);

        Some((target_pos, props))
    }

    /// Equivalent to vanilla `CommandBlockEntity.markConditionMet()`. Conditional
    /// command blocks inspect the command block behind their own facing direction.
    fn conditions_met(world: &World, pos: &BlockPos) -> bool {
        let (_block, state_id) = world.get_block_and_state_id(pos);
        let props = CommandBlockLikeProperties::from_state_id(state_id);

        if !props.conditional {
            return true;
        }

        let Some(before) = Self::get_relative_facing(world, pos, props.facing.opposite()) else {
            return false;
        };
        let Some(before_entity) = world.get_block_entity(&before.0) else {
            warn!("Command block has no matching entity");
            return false;
        };
        let Some(command_entity) = before_entity.as_any().downcast_ref::<CommandBlockEntity>()
        else {
            warn!("Block entity at {} is not a command block", before.0);
            return false;
        };

        command_entity.success_count.load(Ordering::Relaxed) > 0
    }

    fn mark_condition_met(
        world: &World,
        command_block: &CommandBlockEntity,
        pos: &BlockPos,
    ) -> bool {
        let condition_met = Self::conditions_met(world, pos);
        command_block
            .condition_met
            .store(condition_met, Ordering::Release);
        condition_met
    }

    /// Mirrors vanilla CommandBlock.setPoweredAndUpdate: only a power edge mutates
    /// the powered state, and a REDSTONE-mode rising edge records the condition and
    /// schedules execution one tick later. Chain/automatic blocks do not schedule
    /// themselves from the edge.
    fn update(
        world: &World,
        block: &Block,
        command_block: &CommandBlockEntity,
        pos: &BlockPos,
        powered: bool,
    ) {
        let was_powered = command_block.powered.load(Ordering::Relaxed);
        if was_powered == powered {
            return;
        }

        command_block.powered.store(powered, Ordering::Relaxed);
        if !powered
            || command_block.auto.load(Ordering::Relaxed)
            || block.id == Block::CHAIN_COMMAND_BLOCK.id
        {
            return;
        }

        Self::mark_condition_met(world, command_block, pos);
        world.schedule_block_tick(block, *pos, 1, TickPriority::Normal);
    }

    fn execute(
        server: &Arc<Server>,
        world: Arc<World>,
        block_entity: Arc<dyn BlockEntity>,
        command: &str,
    ) {
        let command_blocks_work = { world.level_info.load().game_rules.command_blocks_work };
        if !command_blocks_work {
            return;
        }

        let Ok(command_entity) = Arc::downcast::<CommandBlockEntity>(block_entity) else {
            warn!("Failed to downcast block entity to CommandBlockEntity");
            return;
        };

        if command.is_empty() {
            command_entity.success_count.store(0, Ordering::Release);
        } else {
            let source = CommandSender::CommandBlock(command_entity, world).into_source(server);

            server
                .command_dispatcher
                .load()
                .handle_command(&source, command);
        }
    }

    fn chain_execute(server: &Arc<Server>, world: &Arc<World>, start: BlockPos) {
        let mut i = u16::MAX;
        let mut pos = start;

        while i > 0 {
            let command_blocks_work = { world.level_info.load().game_rules.command_blocks_work };
            if !command_blocks_work {
                return;
            }
            let (block, state_id) = world.get_block_and_state_id(&pos);

            if block.id != Block::CHAIN_COMMAND_BLOCK.id {
                break;
            }
            let Some(block_entity) = world.get_block_entity(&pos) else {
                warn!("Missing command block entity");
                break;
            };

            let Some(command_entity) = block_entity.as_any().downcast_ref::<CommandBlockEntity>()
            else {
                warn!("Block entity at {} is not a command block", pos);
                break;
            };
            let powered = command_entity.powered.load(Ordering::Relaxed);
            let auto = command_entity.auto.load(Ordering::Relaxed);
            let props = CommandBlockLikeProperties::from_state_id(state_id);

            if powered || auto {
                let condition_met = Self::mark_condition_met(world, command_entity, &pos);
                if condition_met {
                    let command = command_entity
                        .command
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .clone();
                    let Some(entity) = world.get_block_entity(&pos) else {
                        warn!("Command block entity disappeared during execution");
                        break;
                    };
                    Self::execute(server, world.clone(), entity, &command);
                } else if props.conditional {
                    command_entity.success_count.store(0, Ordering::Release);
                }
            }

            // Vanilla follows each chain block's own FACING, allowing chains to turn.
            pos = pos.offset(props.facing.to_block_direction().to_offset());

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
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::COMMAND_BLOCK,
            BlockId::CHAIN_COMMAND_BLOCK,
            BlockId::REPEATING_COMMAND_BLOCK,
        ]
        .into()
    }
}

impl BlockBehaviour for CommandBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = CommandBlockLikeProperties::default(args.block);
        props.facing = args.player.get_entity().get_facing().opposite();
        props.to_state_id(args.block)
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        {
            if args.player.permission_lvl.load() < PermissionLvl::Two {
                return BlockActionResult::Pass;
            }
            let Some(block_entity) = args.world.get_block_entity(args.position) else {
                return BlockActionResult::Pass;
            };
            args.world.update_block_entity(&block_entity);
            BlockActionResult::SuccessServer
        }
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        {
            let command_blocks_work =
                { args.world.level_info.load().game_rules.command_blocks_work };
            if !command_blocks_work {
                return;
            }
            if let Some(block_entity) = args.world.get_block_entity(args.position) {
                if block_entity.resource_location() != CommandBlockEntity::ID {
                    return;
                }
                let Some(command_entity) =
                    block_entity.as_any().downcast_ref::<CommandBlockEntity>()
                else {
                    warn!("Block entity at {} is not a command block", args.position);
                    return;
                };

                Self::update(
                    args.world,
                    args.block,
                    command_entity,
                    args.position,
                    block_receives_redstone_power(args.world, args.position),
                );
            }
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let command_blocks_work = { args.world.level_info.load().game_rules.command_blocks_work };
        if !command_blocks_work {
            return;
        }
        let Some(block_entity) = args.world.get_block_entity(args.position) else {
            return;
        };
        if block_entity.resource_location() != CommandBlockEntity::ID {
            return;
        }

        let Some(command_entity) = block_entity.as_any().downcast_ref::<CommandBlockEntity>()
        else {
            warn!("Block entity at {} is not a command block", args.position);
            return;
        };
        let Some(server) = args.world.server.upgrade() else {
            return;
        };

        let block = args.world.get_block(args.position);
        let state_id = args.world.get_block_state_id(args.position);
        let props = CommandBlockLikeProperties::from_state_id(state_id);
        let was_condition_met = command_entity.condition_met.load(Ordering::Acquire);

        let should_execute = if block == &Block::REPEATING_COMMAND_BLOCK {
            // Vanilla computes the next condition before using the condition captured
            // for this tick.
            Self::mark_condition_met(args.world, command_entity, args.position);
            was_condition_met
        } else if block == &Block::COMMAND_BLOCK {
            was_condition_met
        } else {
            // Sequence command blocks execute only as part of a chain.
            false
        };

        if should_execute {
            let command = command_entity
                .command
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            let world = args.world.clone();
            let position = *args.position;
            let facing = props.facing;
            Self::execute(&server, world.clone(), block_entity.clone(), &command);
            Self::chain_execute(
                &server,
                &world,
                position.offset(facing.to_block_direction().to_offset()),
            );
        } else if props.conditional {
            command_entity.success_count.store(0, Ordering::Release);
        }

        let is_auto = command_entity.auto.load(Ordering::Relaxed);
        let can_run = command_entity.powered.load(Ordering::Relaxed) || is_auto;
        if block == &Block::REPEATING_COMMAND_BLOCK && can_run {
            args.world
                .schedule_block_tick(block, *args.position, 1, TickPriority::Normal);
        }
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        if let Some(player) = args.player
            && player.gamemode.load() == GameMode::Creative
        {
            return true;
        }

        false
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        {
            let send_command_feedback = {
                let game_rules = &args.world.level_info.load().game_rules;
                game_rules.send_command_feedback
            };

            let entity = CommandBlockEntity::new(
                *args.position,
                send_command_feedback,
                args.block.id == Block::CHAIN_COMMAND_BLOCK.id,
            );
            args.world.add_block_entity(Arc::new(entity));
        }
    }

    fn get_comparator_output(&self, args: crate::block::GetComparatorOutputArgs<'_>) -> Option<u8> {
        {
            let entity = args.world.get_block_entity(args.position);

            entity.map_or_else(
                || {
                    warn!("Command block is missing its corresponding block entity");
                    None
                },
                |entity| {
                    let command_block_entity: Option<&CommandBlockEntity> =
                        entity.as_any().downcast_ref();
                    command_block_entity.map(|e| e.success_count.load(Ordering::Acquire) as u8)
                },
            )
        }
    }

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        let mut props = CommandBlockLikeProperties::from_state_id(state_id);
        props.facing = rotation
            .rotate(props.facing.to_block_direction())
            .to_facing();
        BlockState::from_id(props.to_state_id(block))
    }
}
