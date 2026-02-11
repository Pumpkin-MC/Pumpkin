use std::sync::atomic::Ordering;

use pumpkin_config::whitelist::WhitelistEntry;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::CommandResult;
use crate::entity::EntityBase;
use crate::{
    command::{
        CommandExecutor, CommandSender,
        args::{Arg, ConsumedArgs, players::PlayersArgumentConsumer},
        dispatcher::CommandError,
        tree::{
            CommandTree,
            builder::{argument, literal},
        },
    },
    data::{LoadJSONConfiguration, SaveJSONConfiguration, whitelist::WhitelistConfig},
    net::DisconnectReason,
    server::Server,
};

const NAMES: [&str; 1] = ["whitelist"];
const DESCRIPTION: &str = "Manage server whitelists.";
const ARG_TARGETS: &str = "targets";

async fn kick_non_whitelisted_players(server: &Server) {
    let whitelist = server.data.whitelist_config.read().await;
    if server.basic_config.enforce_whitelist && server.white_list.load(Ordering::Relaxed) {
        for player in server.get_all_players() {
            if whitelist.is_whitelisted(&player.gameprofile) {
                continue;
            }
            player
                .kick(
                    DisconnectReason::Kicked,
                    TextComponent::translate(
                        translation::MULTIPLAYER_DISCONNECT_NOT_WHITELISTED,
                        &[],
                    ),
                )
                .await;
        }
    }
}

struct OnExecutor;

impl CommandExecutor for OnExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let previous = server.white_list.swap(true, Ordering::Relaxed);
            if previous {
                Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_WHITELIST_ALREADYON,
                    &[],
                )))
            } else {
                kick_non_whitelisted_players(server).await;
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_WHITELIST_ENABLED,
                        &[],
                    ))
                    .await;
                Ok(1)
            }
        })
    }
}

struct OffExecutor;

impl CommandExecutor for OffExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let previous = server.white_list.swap(false, Ordering::Relaxed);
            if previous {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_WHITELIST_DISABLED,
                        &[],
                    ))
                    .await;
                Ok(1)
            } else {
                Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_WHITELIST_ALREADYOFF,
                    &[],
                )))
            }
        })
    }
}

struct ListExecutor;

impl CommandExecutor for ListExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let whitelist = &server.data.whitelist_config.read().await.whitelist;
            if whitelist.is_empty() {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_WHITELIST_NONE,
                        [],
                    ))
                    .await;
                return Ok(0);
            }

            let names = whitelist
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<&str>>()
                .join(", ");

            let names_len = names.len() as i32;

            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_WHITELIST_LIST,
                    [
                        TextComponent::text(whitelist.len().to_string()),
                        TextComponent::text(names),
                    ],
                ))
                .await;

            Ok(names_len)
        })
    }
}

struct ReloadExecutor;

impl CommandExecutor for ReloadExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            *server.data.whitelist_config.write().await = WhitelistConfig::load();
            kick_non_whitelisted_players(server).await;
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_WHITELIST_RELOADED,
                    &[],
                ))
                .await;
            Ok(1)
        })
    }
}

pub struct AddExecutor;

impl CommandExecutor for AddExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Players(targets)) = args.get(&ARG_TARGETS) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_TARGETS.into())));
            };

            let mut whitelist = server.data.whitelist_config.write().await;
            let mut successes: i32 = 0;
            for player in targets {
                let profile = &player.gameprofile;
                if whitelist.is_whitelisted(profile) {
                    continue;
                }
                whitelist
                    .whitelist
                    .push(WhitelistEntry::new(profile.id, profile.name.clone()));
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_WHITELIST_ADD_SUCCESS,
                        [TextComponent::text(profile.name.clone())],
                    ))
                    .await;
                successes += 1;
            }

            whitelist.save();

            if successes == 0 {
                Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_WHITELIST_ADD_FAILED,
                    &[],
                )))
            } else {
                Ok(successes)
            }
        })
    }
}

pub struct RemoveExecutor;

impl CommandExecutor for RemoveExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Players(targets)) = args.get(&ARG_TARGETS) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_TARGETS.into())));
            };

            let mut whitelist = server.data.whitelist_config.write().await;
            let mut successes: i32 = 0;
            for player in targets {
                let i = whitelist
                    .whitelist
                    .iter()
                    .position(|entry| entry.uuid == player.gameprofile.id);

                if let Some(i) = i {
                    whitelist.whitelist.remove(i);
                    sender
                        .send_message(TextComponent::translate(
                            translation::COMMANDS_WHITELIST_REMOVE_SUCCESS,
                            [player.get_display_name().await],
                        ))
                        .await;
                    successes += 1;
                }
            }

            whitelist.save();
            drop(whitelist);

            kick_non_whitelisted_players(server).await;

            if successes == 0 {
                Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_WHITELIST_REMOVE_FAILED,
                    &[],
                )))
            } else {
                Ok(successes)
            }
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(literal("on").execute(OnExecutor))
        .then(literal("off").execute(OffExecutor))
        .then(literal("list").execute(ListExecutor))
        .then(literal("reload").execute(ReloadExecutor))
        .then(
            literal("add")
                .then(argument(ARG_TARGETS, PlayersArgumentConsumer).execute(AddExecutor)),
        )
        .then(
            literal("remove")
                .then(argument(ARG_TARGETS, PlayersArgumentConsumer).execute(RemoveExecutor)),
        )
}
