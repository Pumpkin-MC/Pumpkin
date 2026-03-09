use std::sync::Arc;

use pumpkin_data::translation;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CSetCamera;
use pumpkin_util::GameMode;
use pumpkin_util::text::TextComponent;

use crate::command::args::entity::EntityArgumentConsumer;
use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, require};
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::entity::EntityBase;
use crate::entity::player::Player;

const NAMES: [&str; 1] = ["spectate"];

const DESCRIPTION: &str = "Makes a player in Spectator mode spectate an entity.";

const ARG_TARGET: &str = "target";
const ARG_PLAYER: &str = "player";

struct SpectateExecutor {
    has_player_arg: bool,
}

impl CommandExecutor for SpectateExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        let has_player_arg = self.has_player_arg;
        Box::pin(async move {
            let target: Arc<dyn EntityBase> =
                EntityArgumentConsumer::find_arg(args, ARG_TARGET)?;

            let player: Arc<Player> = if has_player_arg {
                let players = PlayersArgumentConsumer::find_arg(args, ARG_PLAYER)?;
                players
                    .first()
                    .ok_or(CommandError::InvalidConsumption(Some(ARG_PLAYER.into())))?
                    .clone()
            } else {
                sender
                    .as_player()
                    .ok_or(CommandError::InvalidRequirement)?
                    .clone()
            };

            // Must be in spectator mode
            if player.gamemode.load() != GameMode::Spectator {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_SPECTATE_NOT_SPECTATOR,
                        [],
                    ))
                    .await;
                return Ok(0);
            }

            // Can't spectate yourself
            let target_entity = target.get_entity();
            if target_entity.entity_id == player.entity_id() {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_SPECTATE_SELF,
                        [],
                    ))
                    .await;
                return Ok(0);
            }

            player
                .client
                .enqueue_packet(&CSetCamera::new(VarInt(target_entity.entity_id)))
                .await;

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_SPECTATE_SUCCESS_STARTED,
                    [],
                ))
                .await;
            Ok(1)
        })
    }
}

struct StopSpectateExecutor;

impl CommandExecutor for StopSpectateExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let player = sender
                .as_player()
                .ok_or(CommandError::InvalidRequirement)?;

            if player.gamemode.load() != GameMode::Spectator {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_SPECTATE_NOT_SPECTATOR,
                        [],
                    ))
                    .await;
                return Ok(0);
            }

            // Reset camera to self
            player
                .client
                .enqueue_packet(&CSetCamera::new(VarInt(player.entity_id())))
                .await;

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_SPECTATE_SUCCESS_STOPPED,
                    [],
                ))
                .await;
            Ok(1)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            require(CommandSender::is_player)
                .execute(StopSpectateExecutor),
        )
        .then(
            argument(ARG_TARGET, EntityArgumentConsumer)
                .then(
                    require(CommandSender::is_player)
                        .execute(SpectateExecutor {
                            has_player_arg: false,
                        }),
                )
                .then(
                    argument(ARG_PLAYER, PlayersArgumentConsumer)
                        .execute(SpectateExecutor {
                            has_player_arg: true,
                        }),
                ),
        )
}
