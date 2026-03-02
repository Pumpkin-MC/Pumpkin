use std::sync::Arc;

use pumpkin_data::translation;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;

use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::position_block::BlockPosArgumentConsumer;
use crate::command::args::rotation::RotationArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs, FindArgDefaultName};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::argument;
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::entity::player::Player;

const NAMES: [&str; 1] = ["spawnpoint"];

const DESCRIPTION: &str = "Sets the spawn point for a player.";

const ARG_TARGETS: &str = "targets";
const ARG_POS: &str = "pos";
const ARG_ANGLE: &str = "angle";

/// `/spawnpoint` - set self at current position
struct SelfExecutor;

impl CommandExecutor for SelfExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(player) = sender.as_player() else {
                return Err(CommandError::InvalidRequirement);
            };
            let pos = player.position().to_block_pos();
            let yaw = player.living_entity.entity.yaw.load();
            set_spawnpoint(sender, &player, pos, yaw).await;

            Ok(1)
        })
    }
}

/// `/spawnpoint <targets>` - set targets at their current positions
struct TargetsExecutor;

impl CommandExecutor for TargetsExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = PlayersArgumentConsumer.find_arg_default_name(args)?;

            for target in targets {
                let pos = target.position().to_block_pos();
                let yaw = target.living_entity.entity.yaw.load();
                set_spawnpoint(sender, target, pos, yaw).await;
            }

            Ok(targets.len() as i32)
        })
    }
}

/// `/spawnpoint <targets> <pos>` - set targets at specified position
struct TargetsPosExecutor;

impl CommandExecutor for TargetsPosExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = PlayersArgumentConsumer.find_arg_default_name(args)?;
            let pos = BlockPosArgumentConsumer::find_spawnable_arg(args, ARG_POS)?;

            for target in targets {
                let yaw = target.living_entity.entity.yaw.load();
                set_spawnpoint(sender, target, pos, yaw).await;
            }

            Ok(targets.len() as i32)
        })
    }
}

/// `/spawnpoint <targets> <pos> <angle>` - set targets at position with angle
struct TargetsPosAngleExecutor;

impl CommandExecutor for TargetsPosAngleExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = PlayersArgumentConsumer.find_arg_default_name(args)?;
            let pos = BlockPosArgumentConsumer::find_spawnable_arg(args, ARG_POS)?;
            let Some(Arg::Rotation(yaw, _, _, _)) = args.get(ARG_ANGLE) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_ANGLE.into())));
            };

            for target in targets {
                set_spawnpoint(sender, target, pos, *yaw).await;
            }

            Ok(targets.len() as i32)
        })
    }
}

async fn set_spawnpoint(sender: &CommandSender, target: &Arc<Player>, pos: BlockPos, yaw: f32) {
    let dimension = target.world().dimension;

    target
        .set_respawn_point_forced(dimension, pos, yaw, 0.0)
        .await;

    sender
        .send_message(TextComponent::translate(
            translation::COMMANDS_SPAWNPOINT_SUCCESS_SINGLE,
            [
                TextComponent::text(pos.0.x.to_string()),
                TextComponent::text(pos.0.y.to_string()),
                TextComponent::text(pos.0.z.to_string()),
                TextComponent::text(format!("{yaw:.2}")),
                TextComponent::text("0.00".to_string()),
                TextComponent::text(dimension.minecraft_name),
                TextComponent::text(target.gameprofile.name.clone()),
            ],
        ))
        .await;
}

#[must_use]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .execute(SelfExecutor)
        .then(
            argument(ARG_TARGETS, PlayersArgumentConsumer)
                .execute(TargetsExecutor)
                .then(
                    argument(ARG_POS, BlockPosArgumentConsumer)
                        .execute(TargetsPosExecutor)
                        .then(
                            argument(ARG_ANGLE, RotationArgumentConsumer)
                                .execute(TargetsPosAngleExecutor),
                        ),
                ),
        )
}
