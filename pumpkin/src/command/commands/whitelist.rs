use std::sync::atomic::Ordering;

use pumpkin_config::whitelist::WhitelistEntry;
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
    data::{
        LoadJSONConfiguration, SaveJSONConfiguration,
        whitelist_data::{WHITELIST_CONFIG, WhitelistConfig},
    },
    net::DisconnectReason,
    server::Server,
};

const NAMES: [&str; 1] = ["whitelist"];
const DESCRIPTION: &str = "Manage server whitelists.";
const ARG_TARGETS: &str = "targets";

async fn kick_non_whitelisted_players(server: &Server) {
    let whitelist = WHITELIST_CONFIG.read().await;
    if server.basic_config.enforce_whitelist && server.white_list.load(Ordering::Relaxed) {
        for player in server.get_all_players().await {
            if whitelist.is_whitelisted(&player.gameprofile) {
                continue;
            }
            player
                .kick(
                    DisconnectReason::Kicked,
                    TextComponent::translate("multiplayer.disconnect.not_whitelisted", &[]),
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
                sender
                    .send_message(TextComponent::translate(
                        "commands.whitelist.alreadyOn",
                        &[],
                    ))
                    .await;
            } else {
                kick_non_whitelisted_players(server).await;
                sender
                    .send_message(TextComponent::translate("commands.whitelist.enabled", &[]))
                    .await;
            }
            Ok(())
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
                    .send_message(TextComponent::translate("commands.whitelist.disabled", &[]))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        "commands.whitelist.alreadyOff",
                        &[],
                    ))
                    .await;
            }
            Ok(())
        })
    }
}

struct ListExecutor;

impl CommandExecutor for ListExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let whitelist = &WHITELIST_CONFIG.read().await.whitelist;
            if whitelist.is_empty() {
                sender
                    .send_message(TextComponent::translate("commands.whitelist.none", []))
                    .await;
                return Ok(());
            }

            let names = whitelist
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<&str>>()
                .join(", ");

            sender
                .send_message(TextComponent::translate(
                    "commands.whitelist.list",
                    [
                        TextComponent::text(whitelist.len().to_string()),
                        TextComponent::text(names),
                    ],
                ))
                .await;

            Ok(())
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
            *WHITELIST_CONFIG.write().await = WhitelistConfig::load();
            kick_non_whitelisted_players(server).await;
            sender
                .send_message(TextComponent::translate("commands.whitelist.reloaded", &[]))
                .await;
            Ok(())
        })
    }
}

pub struct AddExecutor;

impl CommandExecutor for AddExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Players(targets)) = args.get(&ARG_TARGETS) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_TARGETS.into())));
            };

            let mut whitelist = WHITELIST_CONFIG.write().await;
            for player in targets {
                let profile = &player.gameprofile;
                if whitelist.is_whitelisted(profile) {
                    sender
                        .send_message(TextComponent::translate(
                            "commands.whitelist.add.failed",
                            &[],
                        ))
                        .await;
                    continue;
                }
                whitelist
                    .whitelist
                    .push(WhitelistEntry::new(profile.id, profile.name.clone()));
                sender
                    .send_message(TextComponent::translate(
                        "commands.whitelist.add.success",
                        [TextComponent::text(profile.name.clone())],
                    ))
                    .await;
            }

            whitelist.save();
            Ok(())
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

            let mut whitelist = WHITELIST_CONFIG.write().await;
            for player in targets {
                let i = whitelist
                    .whitelist
                    .iter()
                    .position(|entry| entry.uuid == player.gameprofile.id);

                match i {
                    Some(i) => {
                        whitelist.whitelist.remove(i);
                        sender
                            .send_message(TextComponent::translate(
                                "commands.whitelist.remove.success",
                                [player.get_display_name().await],
                            ))
                            .await;
                    }
                    None => {
                        sender
                            .send_message(TextComponent::translate(
                                "commands.whitelist.remove.failed",
                                [],
                            ))
                            .await;
                    }
                }
            }

            whitelist.save();
            drop(whitelist);

            kick_non_whitelisted_players(server).await;
            Ok(())
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
