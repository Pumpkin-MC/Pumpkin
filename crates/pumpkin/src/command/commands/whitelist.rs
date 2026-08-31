use std::sync::atomic::Ordering;

use pumpkin_config::whitelist::WhitelistEntry;
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::game_profile::GameProfileArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::data::{LoadJSONConfiguration, SaveJSONConfiguration, whitelist::WhitelistConfig};
use crate::net::DisconnectReason;
use crate::server::Server;

const DESCRIPTION: &str = "Manage server whitelists.";
const PERMISSION: &str = "minecraft:command.whitelist";

const ERROR_ALREADY_ON: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_WHITELIST_ALREADYON,
    translation::bedrock::COMMANDS_ALLOWLIST_ENABLED,
);

const ERROR_ALREADY_OFF: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_WHITELIST_ALREADYOFF,
    translation::bedrock::COMMANDS_ALLOWLIST_DISABLED,
);

const ERROR_ADD_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_WHITELIST_ADD_FAILED,
    translation::bedrock::COMMANDS_ALLOWLIST_ADD_FAILED,
);

const ERROR_REMOVE_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_WHITELIST_REMOVE_FAILED,
    translation::bedrock::COMMANDS_ALLOWLIST_REMOVE_FAILED,
);

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
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let server = context.source.server();
        let previous = server.white_list.swap(true, Ordering::Relaxed);
        if previous {
            Err(ERROR_ALREADY_ON.create_without_context())
        } else {
            kick_non_whitelisted_players(server);
            context.source.send_feedback(
                pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WHITELIST_ENABLED,
                    translation::bedrock::COMMANDS_ALLOWLIST_ENABLED
                ),
                true,
            );
            Ok(1)
        }
    }
}

struct OffExecutor;

impl CommandExecutor for OffExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let server = context.source.server();
        let previous = server.white_list.swap(false, Ordering::Relaxed);
        if previous {
            context.source.send_feedback(
                pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WHITELIST_DISABLED,
                    translation::bedrock::COMMANDS_ALLOWLIST_DISABLED
                ),
                true,
            );
            Ok(1)
        } else {
            Err(ERROR_ALREADY_OFF.create_without_context())
        }
    }
}

struct ListExecutor;

impl CommandExecutor for ListExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let server = context.source.server();
        let whitelist_guard = server.data.whitelist_config.read().unwrap();
        let whitelist = &whitelist_guard.whitelist;
        if whitelist.is_empty() {
            context.source.send_feedback(
                pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WHITELIST_NONE,
                    translation::bedrock::COMMANDS_ALLOWLIST_LIST
                ),
                false,
            );
            return Ok(0);
        }

        let names = whitelist
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<Vec<&str>>()
            .join(", ");

        let names_len = names.len() as i32;

        context.source.send_feedback(
            pumpkin_macros::translate_cross!(
                translation::java::COMMANDS_WHITELIST_LIST,
                translation::bedrock::COMMANDS_ALLOWLIST_LIST,
                TextComponent::text(whitelist.len().to_string()),
                TextComponent::text(names)
            ),
            false,
        );

        Ok(names_len)
    }
}

struct ReloadExecutor;

impl CommandExecutor for ReloadExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let server = context.source.server();
        *server.data.whitelist_config.write().unwrap() = WhitelistConfig::load();
        kick_non_whitelisted_players(server);
        context.source.send_feedback(
            pumpkin_macros::translate_cross!(
                translation::java::COMMANDS_WHITELIST_RELOADED,
                translation::bedrock::COMMANDS_ALLOWLIST_RELOADED
            ),
            true,
        );
        Ok(1)
    }
}

struct AddExecutor;

impl CommandExecutor for AddExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = GameProfileArgumentType::get(context, "targets")?;
        let server = context.source.server();
        let mut whitelist = server.data.whitelist_config.write().unwrap();
        let mut successes: i32 = 0;
        for profile in &targets {
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
            context.source.send_feedback(
                pumpkin_macros::translate_cross!(
                    translation::java::COMMANDS_WHITELIST_ADD_SUCCESS,
                    translation::bedrock::COMMANDS_ALLOWLIST_ADD_SUCCESS,
                    TextComponent::text(profile.name.clone())
                ),
                true,
            );
            successes += 1;
        }

        whitelist.save();

        if successes == 0 {
            Err(ERROR_ADD_FAILED.create_without_context())
        } else {
            Ok(successes)
        }
    }
}

struct RemoveExecutor;

impl CommandExecutor for RemoveExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = GameProfileArgumentType::get(context, "targets")?;
        let server = context.source.server();
        let mut whitelist = server.data.whitelist_config.write().unwrap();
        let mut successes: i32 = 0;
        for player in &targets {
            let i = whitelist
                .whitelist
                .iter()
                .position(|entry| entry.uuid == player.id);

            if let Some(i) = i {
                whitelist.whitelist.remove(i);
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_WHITELIST_REMOVE_SUCCESS,
                        translation::bedrock::COMMANDS_ALLOWLIST_REMOVE_SUCCESS,
                        TextComponent::text(player.name.clone())
                    ),
                    true,
                );
                successes += 1;
            }
        }

        whitelist.save();
        drop(whitelist);

        kick_non_whitelisted_players(server);

        if successes == 0 {
            Err(ERROR_REMOVE_FAILED.create_without_context())
        } else {
            Ok(successes)
        }
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Three),
    ));

    dispatcher.register(
        command("whitelist", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("on").executes(OnExecutor))
            .then(literal("off").executes(OffExecutor))
            .then(literal("list").executes(ListExecutor))
            .then(literal("reload").executes(ReloadExecutor))
            .then(
                literal("add")
                    .then(argument("targets", GameProfileArgumentType).executes(AddExecutor)),
            )
            .then(
                literal("remove")
                    .then(argument("targets", GameProfileArgumentType).executes(RemoveExecutor)),
            ),
    );
}
