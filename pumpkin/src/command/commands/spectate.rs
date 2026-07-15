use std::sync::Arc;

use crate::command::args::entity::EntityArgumentConsumer;
use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs, FindArg};
use crate::command::dispatcher::CommandError::{self, InvalidConsumption, InvalidRequirement};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, require};
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::entity::EntityBase;
use pumpkin_data::translation;
use pumpkin_protocol::java::client::play::CSetCamera;
use pumpkin_util::GameMode;
use pumpkin_util::text::TextComponent;

const NAMES: [&str; 1] = ["spectate"];

const DESCRIPTION: &str = "Allows a player in spectator mode to spectate a given target entity.";

const ARG_TARGET: &str = "target";
const ARG_PLAYER: &str = "player";

struct StopSpectateExecutor;

impl CommandExecutor for StopSpectateExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(player) = sender.as_player() else {
                return Err(InvalidRequirement);
            };

            if player.gamemode.load() != GameMode::Spectator {
                let display_name = player.get_display_name().await;
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    translation::java::COMMANDS_SPECTATE_NOT_SPECTATOR,
                    translation::java::COMMANDS_SPECTATE_NOT_SPECTATOR,
                    [display_name],
                )));
            }

            player.camera_target_id.store(None);
            player
                .client
                .send_packet_now(&CSetCamera::new(player.entity_id().into()))
                .await;

            sender
                .send_message(TextComponent::translate_cross(
                    translation::java::COMMANDS_SPECTATE_SUCCESS_STOPPED,
                    translation::java::COMMANDS_SPECTATE_SUCCESS_STOPPED,
                    [],
                ))
                .await;

            Ok(1)
        })
    }
}

struct SpectateTargetSelfExecutor;

impl CommandExecutor for SpectateTargetSelfExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(player) = sender.as_player() else {
                return Err(InvalidRequirement);
            };

            if player.gamemode.load() != GameMode::Spectator {
                let display_name = player.get_display_name().await;
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    translation::java::COMMANDS_SPECTATE_NOT_SPECTATOR,
                    translation::java::COMMANDS_SPECTATE_NOT_SPECTATOR,
                    [display_name],
                )));
            }

            let target = EntityArgumentConsumer::find_arg(args, ARG_TARGET)?;

            let target_entity = target.get_entity();
            if target_entity.entity_id == player.entity_id() {
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    translation::java::COMMANDS_SPECTATE_SELF,
                    translation::java::COMMANDS_SPECTATE_SELF,
                    [],
                )));
            }

            let target_world = target_entity.world.load_full();
            let player_world = player.world();
            if !Arc::ptr_eq(&target_world, &player_world) {
                let target_name = target.get_display_name().await;
                return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                    translation::java::COMMANDS_SPECTATE_CANNOT_SPECTATE,
                    translation::java::COMMANDS_SPECTATE_CANNOT_SPECTATE,
                    [target_name],
                )));
            }

            let target_id = target_entity.entity_id;
            player.camera_target_id.store(Some(target_id));
            player
                .client
                .send_packet_now(&CSetCamera::new(target_id.into()))
                .await;

            let pos = target_entity.pos.load();
            let yaw = target_entity.yaw.load();
            let pitch = target_entity.pitch.load();
            player
                .clone()
                .teleport(pos, Some(yaw), Some(pitch), player_world)
                .await;

            let target_name = target.get_display_name().await;
            sender
                .send_message(TextComponent::translate_cross(
                    translation::java::COMMANDS_SPECTATE_SUCCESS_STARTED,
                    translation::java::COMMANDS_SPECTATE_SUCCESS_STARTED,
                    [target_name],
                ))
                .await;

            Ok(1)
        })
    }
}

struct SpectateTargetOtherExecutor;

impl CommandExecutor for SpectateTargetOtherExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Players(players)) = args.get(ARG_PLAYER) else {
                return Err(InvalidConsumption(Some(ARG_PLAYER.into())));
            };

            let target = EntityArgumentConsumer::find_arg(args, ARG_TARGET)?;
            let target_entity = target.get_entity();
            let target_world = target_entity.world.load_full();

            // First validate all players
            for player in players {
                if player.gamemode.load() != GameMode::Spectator {
                    let display_name = player.get_display_name().await;
                    return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                        translation::java::COMMANDS_SPECTATE_NOT_SPECTATOR,
                        translation::java::COMMANDS_SPECTATE_NOT_SPECTATOR,
                        [display_name],
                    )));
                }

                if target_entity.entity_id == player.entity_id() {
                    return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                        translation::java::COMMANDS_SPECTATE_SELF,
                        translation::java::COMMANDS_SPECTATE_SELF,
                        [],
                    )));
                }

                let player_world = player.world();
                if !Arc::ptr_eq(&target_world, &player_world) {
                    let target_name = target.get_display_name().await;
                    return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                        translation::java::COMMANDS_SPECTATE_CANNOT_SPECTATE,
                        translation::java::COMMANDS_SPECTATE_CANNOT_SPECTATE,
                        [target_name],
                    )));
                }
            }

            let mut succeeded = 0;
            for player in players {
                let target_id = target_entity.entity_id;
                player.camera_target_id.store(Some(target_id));
                player
                    .client
                    .send_packet_now(&CSetCamera::new(target_id.into()))
                    .await;

                let pos = target_entity.pos.load();
                let yaw = target_entity.yaw.load();
                let pitch = target_entity.pitch.load();
                let player_world = player.world();
                player
                    .clone()
                    .teleport(pos, Some(yaw), Some(pitch), player_world)
                    .await;
                succeeded += 1;
            }

            let target_name = target.get_display_name().await;
            sender
                .send_message(TextComponent::translate_cross(
                    translation::java::COMMANDS_SPECTATE_SUCCESS_STARTED,
                    translation::java::COMMANDS_SPECTATE_SUCCESS_STARTED,
                    [target_name],
                ))
                .await;

            Ok(succeeded)
        })
    }
}

#[expect(clippy::redundant_closure_for_method_calls)]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            argument(ARG_TARGET, EntityArgumentConsumer)
                .then(require(|sender| sender.is_player()).execute(SpectateTargetSelfExecutor))
                .then(
                    argument(ARG_PLAYER, PlayersArgumentConsumer)
                        .execute(SpectateTargetOtherExecutor),
                ),
        )
        .then(require(|sender| sender.is_player()).execute(StopSpectateExecutor))
}
