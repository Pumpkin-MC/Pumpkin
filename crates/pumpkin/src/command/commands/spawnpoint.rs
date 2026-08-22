use std::sync::Arc;

use pumpkin_data::translation;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::wrap_degrees;
use pumpkin_util::text::TextComponent;

use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::position_block::BlockPosArgumentConsumer;
use crate::command::args::rotation::RotationArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs, FindArg};
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
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &crate::server::Server,
        _args: &ConsumedArgs,
    ) -> CommandResult {
        let Some(player) = sender.as_player() else {
            return Err(CommandError::InvalidRequirement);
        };
        let pos = player.position().to_block_pos();
        set_spawnpoint(sender, &[player], pos, 0.0, 0.0);

        Ok(1)
    }
}

/// `/spawnpoint <targets>` - set targets at the sender's position
struct TargetsExecutor;

impl CommandExecutor for TargetsExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;
        let Some(first) = targets.first() else {
            return Ok(0);
        };
        let pos = sender
            .position()
            .unwrap_or_else(|| first.position())
            .to_block_pos();

        set_spawnpoint(sender, targets, pos, 0.0, 0.0);

        Ok(targets.len() as i32)
    }
}

/// `/spawnpoint <targets> <pos>` - set targets at specified position
struct TargetsPosExecutor;

impl CommandExecutor for TargetsPosExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;
        let Some(Arg::BlockPos(pos)) = args.get(ARG_POS) else {
            return Err(CommandError::InvalidConsumption(Some(ARG_POS.into())));
        };

        set_spawnpoint(sender, targets, *pos, 0.0, 0.0);

        Ok(targets.len() as i32)
    }
}

/// `/spawnpoint <targets> <pos> <angle>` - set targets at position with angle
struct TargetsPosAngleExecutor;

impl CommandExecutor for TargetsPosAngleExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        _server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;
        let Some(Arg::BlockPos(pos)) = args.get(ARG_POS) else {
            return Err(CommandError::InvalidConsumption(Some(ARG_POS.into())));
        };
        let Some(Arg::Rotation(yaw, _, pitch, _)) = args.get(ARG_ANGLE) else {
            return Err(CommandError::InvalidConsumption(Some(ARG_ANGLE.into())));
        };

        set_spawnpoint(sender, targets, *pos, *yaw, *pitch);

        Ok(targets.len() as i32)
    }
}

fn set_spawnpoint(
    sender: &CommandSender,
    targets: &[Arc<Player>],
    pos: BlockPos,
    yaw: f32,
    pitch: f32,
) {
    let yaw = wrap_degrees(yaw);
    let pitch = pitch.clamp(-90.0, 90.0);

    let Some(first) = targets.first() else {
        return;
    };
    let dimension = &first.world().dimension;

    for target in targets {
        target.set_respawn_point(target.world().dimension.clone(), pos, yaw, pitch, true);
    }

    let mut with = vec![
        TextComponent::text(pos.0.x.to_string()),
        TextComponent::text(pos.0.y.to_string()),
        TextComponent::text(pos.0.z.to_string()),
        TextComponent::text(yaw.to_string()),
        TextComponent::text(pitch.to_string()),
        TextComponent::text(dimension.minecraft_name),
    ];

    let message = if targets.len() == 1 {
        with.push(TextComponent::text(first.gameprofile.name.clone()));
        TextComponent::translate_cross(
            translation::java::COMMANDS_SPAWNPOINT_SUCCESS_SINGLE,
            translation::bedrock::COMMANDS_SPAWNPOINT_SUCCESS_SINGLE,
            with,
        )
    } else {
        with.push(TextComponent::text(targets.len().to_string()));
        TextComponent::translate_cross(
            translation::java::COMMANDS_SPAWNPOINT_SUCCESS_MULTIPLE,
            translation::bedrock::COMMANDS_SPAWNPOINT_SUCCESS_MULTIPLE_SPECIFIC,
            with,
        )
    };
    sender.send_message(message);
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
