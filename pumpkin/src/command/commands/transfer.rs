use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CTransfer;
use pumpkin_util::text::TextComponent;

use crate::command::CommandResult;
use crate::command::args::bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, FindArgDefaultName};
use crate::command::dispatcher::CommandError::{self, InvalidConsumption, InvalidRequirement};
use crate::command::tree::builder::{argument, argument_default_name, require};
use crate::command::{CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree};
use crate::entity::EntityBase;

const NAMES: [&str; 1] = ["transfer"];

const DESCRIPTION: &str = "Triggers a transfer of a player to another server.";

const ARG_HOSTNAME: &str = "hostname";

const ARG_PLAYERS: &str = "players";

const fn port_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new()
        .name("port")
        .min(1)
        .max(65535)
}

struct TargetSelfExecutor;

impl CommandExecutor for TargetSelfExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(hostname)) = args.get(ARG_HOSTNAME) else {
                return Err(InvalidConsumption(Some(ARG_HOSTNAME.into())));
            };

            let port = match port_consumer().find_arg_default_name(args) {
                Err(_) => 25565,
                Ok(Ok(count)) => count,
                Ok(Err(_)) => {
                    return Err(InvalidConsumption(Some(
                        "Port must be between 1 and 65535.".into(),
                    )));
                }
            };

            if let CommandSender::Player(player) = sender {
                let name = &player.gameprofile.name;
                log::info!("[{name}: Transferring {name} to {hostname}:{port}]");
                player
                    .client
                    .enqueue_packet(&CTransfer::new(hostname, VarInt(port)))
                    .await;
                Ok(1)
            } else {
                Err(InvalidRequirement)
            }
        })
    }
}

struct TargetPlayerExecutor;

impl CommandExecutor for TargetPlayerExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(hostname)) = args.get(ARG_HOSTNAME) else {
                return Err(InvalidConsumption(Some(ARG_HOSTNAME.into())));
            };
            let hostname = *hostname;

            let port = match port_consumer().find_arg_default_name(args) {
                Err(_) => 25565,
                Ok(Ok(count)) => count,
                Ok(Err(_)) => {
                    return Err(InvalidConsumption(Some(
                        "Port must be between 1 and 65535.".into(),
                    )));
                }
            };

            let Some(Arg::Players(players)) = args.get(ARG_PLAYERS) else {
                return Err(InvalidConsumption(Some(ARG_PLAYERS.into())));
            };

            if players.is_empty() {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "commands.transfer.error.no_players",
                    [],
                )));
            }

            for p in players {
                p.client
                    .enqueue_packet(&CTransfer::new(hostname, VarInt(port)))
                    .await;
                log::info!(
                    "[{sender}: Transferring {} to {hostname}:{port}]",
                    p.gameprofile.name
                );
            }

            if players.len() == 1 {
                sender
                    .send_message(TextComponent::translate(
                        "commands.transfer.success.single",
                        [
                            players[0].get_display_name().await,
                            TextComponent::text(hostname.to_owned()),
                            TextComponent::text(port.to_string()),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        "commands.transfer.success.multiple",
                        [
                            TextComponent::text(players.len().to_string()),
                            TextComponent::text(hostname.to_owned()),
                            TextComponent::text(port.to_string()),
                        ],
                    ))
                    .await;
            }

            Ok(players.len() as i32)
        })
    }
}

#[expect(clippy::redundant_closure_for_method_calls)]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_HOSTNAME, SimpleArgConsumer)
            .then(require(|sender| sender.is_player()).execute(TargetSelfExecutor))
            .then(
                argument_default_name(port_consumer())
                    .then(require(|sender| sender.is_player()).execute(TargetSelfExecutor))
                    .then(
                        argument(ARG_PLAYERS, PlayersArgumentConsumer)
                            .execute(TargetPlayerExecutor),
                    ),
            ),
    )
}
