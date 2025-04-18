use std::sync::atomic::Ordering;

use async_trait::async_trait;
use pumpkin_config::{BASIC_CONFIG, whitelist::WhitelistEntry};
use pumpkin_util::text::TextComponent;

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
    server::Server,
};

const NAMES: [&str; 1] = ["whitelist"];
const DESCRIPTION: &str = "Manage server whitelists.";
const ARG_TARGETS: &str = "targets";

async fn kick_non_whitelisted_players(server: &Server) {
    let whitelist = WHITELIST_CONFIG.read().await;
    if BASIC_CONFIG.enforce_whitelist && server.white_list.load(Ordering::Relaxed) {
        for player in server.get_all_players().await {
            if whitelist.is_whitelisted(&player.gameprofile) {
                continue;
            }
            player
                .kick(TextComponent::translate(
                    "multiplayer.disconnect.not_whitelisted",
                    &[],
                ))
                .await;
        }
    }
}

struct OnExecutor;

#[async_trait]
impl CommandExecutor for OnExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
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
    }
}

struct OffExecutor;

#[async_trait]
impl CommandExecutor for OffExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
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
    }
}

struct ListExecutor;

#[async_trait]
impl CommandExecutor for ListExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
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
    }
}

struct ReloadExecutor;

#[async_trait]
impl CommandExecutor for ReloadExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        *WHITELIST_CONFIG.write().await = WhitelistConfig::load();
        kick_non_whitelisted_players(server).await;
        sender
            .send_message(TextComponent::translate("commands.whitelist.reloaded", &[]))
            .await;
        Ok(())
    }
}

pub struct AddExecutor;

#[async_trait]
impl CommandExecutor for AddExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        _server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
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
    }
}

pub struct RemoveExecutor;

#[async_trait]
impl CommandExecutor for RemoveExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
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
                            [TextComponent::text(player.gameprofile.name.clone())],
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
