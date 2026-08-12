use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::command;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Reloads JSON server lists from disk.";

const PERMISSION: &str = "minecraft:command.reload";

use crate::data::LoadJSONConfiguration;

struct ReloadExecutor;

impl CommandExecutor for ReloadExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let server = context.server;

            // Reload all JSON configurations (load from disk before taking write locks)
            let banned_ip_list = crate::data::banned_ip::BannedIpList::load();
            let banned_player_list = crate::data::banned_player::BannedPlayerList::load();
            let operator_config = crate::data::op::OperatorConfig::load();
            let whitelist_config = crate::data::whitelist::WhitelistConfig::load();

            *server.data.banned_ip_list.write().await = banned_ip_list;
            *server.data.banned_player_list.write().await = banned_player_list;
            *server.data.operator_config.write().await = operator_config;
            *server.data.whitelist_config.write().await = whitelist_config;

            context
                .source
                .send_feedback(
                    TextComponent::translate(translation::java::COMMANDS_RELOAD_SUCCESS, []),
                    true,
                )
                .await;
            Ok(1)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("reload", DESCRIPTION)
            .requires(PERMISSION)
            .executes(ReloadExecutor),
    );
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_reload_permission() {
        assert_eq!(PERMISSION, "minecraft:command.reload");
    }
}
