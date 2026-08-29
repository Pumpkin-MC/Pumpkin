use std::sync::atomic::Ordering;

use pumpkin_config::whitelist::WhitelistEntry;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::CommandResult;
use crate::{
    command::{
        CommandExecutor, CommandSender,
        args::{
            Arg, ConsumedArgs,
            gameprofile::{GameProfileSuggestionMode, GameProfilesArgumentConsumer},
        },
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

fn kick_non_whitelisted_players(server: &Server) {
    let whitelist = server.data.whitelist_config.read().unwrap();
    if server.basic_config.enforce_whitelist && server.white_list.load(Ordering::Relaxed) {
        for player in server.get_all_players() {
            if whitelist.is_whitelisted(&player.gameprofile) {
                continue;
            }
            player.kick(
                DisconnectReason::Kicked,
                &pumpkin_macros::translate_cross!(
                    translation::java::MULTIPLAYER_DISCONNECT_NOT_WHITELISTED,
                    translation::bedrock::DISCONNECT_KICKED
                ),
            );
        }
    }
}

struct OnExecutor;

impl CommandExecutor for OnExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        server: &crate::server::Server,
        _args: &ConsumedArgs,
    ) -> CommandResult {
        let previous = server.white_list.swap(true, Ordering::Relaxed);
        if previous {
            Err(CommandError::CommandFailed(
                pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WHITELIST_ALREADYON,
                    translation::bedrock::COMMANDS_ALLOWLIST_ENABLED
                ),
            ))
        } else {
            kick_non_whitelisted_players(server);
            sender.send_message(pumpkin_macros::translate_cross!(
                translation::java::COMMANDS_WHITELIST_ENABLED,
                translation::bedrock::COMMANDS_ALLOWLIST_ENABLED
            ));
            Ok(1)
        }
    }
}

struct OffExecutor;

impl CommandExecutor for OffExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        server: &crate::server::Server,
        _args: &ConsumedArgs,
    ) -> CommandResult {
        let previous = server.white_list.swap(false, Ordering::Relaxed);
        if previous {
            sender.send_message(pumpkin_macros::translate_cross!(
                translation::java::COMMANDS_WHITELIST_DISABLED,
                translation::bedrock::COMMANDS_ALLOWLIST_DISABLED
            ));
            Ok(1)
        } else {
            Err(CommandError::CommandFailed(
                pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WHITELIST_ALREADYOFF,
                    translation::bedrock::COMMANDS_ALLOWLIST_DISABLED
                ),
            ))
        }
    }
}

struct ListExecutor;

impl CommandExecutor for ListExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        server: &crate::server::Server,
        _args: &ConsumedArgs,
    ) -> CommandResult {
        let whitelist_guard = server.data.whitelist_config.read().unwrap();
        let whitelist = &whitelist_guard.whitelist;
        if whitelist.is_empty() {
            sender.send_message(pumpkin_macros::translate_cross!(
                translation::java::COMMANDS_WHITELIST_NONE,
                translation::bedrock::COMMANDS_ALLOWLIST_LIST
            ));
            return Ok(0);
        }

        let names = whitelist
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<&str>>()
            .join(", ");

        let names_len = names.len() as i32;

        sender.send_message(pumpkin_macros::translate_cross!(
            translation::java::COMMANDS_WHITELIST_LIST,
            translation::bedrock::COMMANDS_ALLOWLIST_LIST,
            TextComponent::text(whitelist.len().to_string()),
            TextComponent::text(names)
        ));

        Ok(names_len)
    }
}

struct ReloadExecutor;

impl CommandExecutor for ReloadExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        server: &crate::server::Server,
        _args: &ConsumedArgs,
    ) -> CommandResult {
        *server.data.whitelist_config.write().unwrap() = WhitelistConfig::load();
        kick_non_whitelisted_players(server);
        sender.send_message(pumpkin_macros::translate_cross!(
            translation::java::COMMANDS_WHITELIST_RELOADED,
            translation::bedrock::COMMANDS_ALLOWLIST_RELOADED
        ));
        Ok(1)
    }
}

pub struct AddExecutor;

impl CommandExecutor for AddExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let Some(Arg::GameProfiles(targets)) = args.get(ARG_TARGETS) else {
            return Err(CommandError::InvalidConsumption(Some(ARG_TARGETS.into())));
        };

        let mut whitelist = server.data.whitelist_config.write().unwrap();
        let mut successes: i32 = 0;
        for profile in targets {
            if let Some(existing_entry) = whitelist
                .whitelist
                .iter_mut()
                .find(|entry| entry.uuid == profile.id)
            {
                if existing_entry.name != profile.name {
                    existing_entry.name.clone_from(&profile.name);
                }
                continue;
            }
            whitelist
                .whitelist
                .push(WhitelistEntry::new(profile.id, profile.name.clone()));
            sender.send_message(pumpkin_macros::translate_cross!(
                translation::java::COMMANDS_WHITELIST_ADD_SUCCESS,
                translation::bedrock::COMMANDS_ALLOWLIST_ADD_SUCCESS,
                TextComponent::text(profile.name.clone())
            ));
            successes += 1;
        }

        whitelist.save();

        if successes == 0 {
            Err(CommandError::CommandFailed(
                pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WHITELIST_ADD_FAILED,
                    translation::bedrock::COMMANDS_ALLOWLIST_ADD_FAILED
                ),
            ))
        } else {
            Ok(successes)
        }
    }
}

pub struct RemoveExecutor;

impl CommandExecutor for RemoveExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let Some(Arg::GameProfiles(targets)) = args.get(ARG_TARGETS) else {
            return Err(CommandError::InvalidConsumption(Some(ARG_TARGETS.into())));
        };

        let mut whitelist = server.data.whitelist_config.write().unwrap();
        let mut successes: i32 = 0;
        for player in targets {
            let i = whitelist
                .whitelist
                .iter()
                .position(|entry| entry.uuid == player.id);

            if let Some(i) = i {
                whitelist.whitelist.remove(i);
                sender.send_message(pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WHITELIST_REMOVE_SUCCESS,
                    translation::bedrock::COMMANDS_ALLOWLIST_REMOVE_SUCCESS,
                    TextComponent::text(player.name.clone())
                ));
                successes += 1;
            }
        }

        whitelist.save();
        drop(whitelist);

        kick_non_whitelisted_players(server);

        if successes == 0 {
            Err(CommandError::CommandFailed(
                pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WHITELIST_REMOVE_FAILED,
                    translation::bedrock::COMMANDS_ALLOWLIST_REMOVE_FAILED
                ),
            ))
        } else {
            Ok(successes)
        }
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(literal("on").execute(OnExecutor))
        .then(literal("off").execute(OffExecutor))
        .then(literal("list").execute(ListExecutor))
        .then(literal("reload").execute(ReloadExecutor))
        .then(
            literal("add").then(
                argument(
                    ARG_TARGETS,
                    GameProfilesArgumentConsumer::new(
                        GameProfileSuggestionMode::NonWhitelistedOnlinePlayers,
                        false,
                    ),
                )
                .execute(AddExecutor),
            ),
        )
        .then(
            literal("remove").then(
                argument(
                    ARG_TARGETS,
                    GameProfilesArgumentConsumer::new(
                        GameProfileSuggestionMode::WhitelistedNames,
                        false,
                    ),
                )
                .execute(RemoveExecutor),
            ),
        )
}
