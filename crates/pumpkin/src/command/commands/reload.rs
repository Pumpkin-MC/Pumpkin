use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, command};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Reloads the server configuration and data.";

const PERMISSION: &str = "minecraft:command.reload";

use crate::data::LoadJSONConfiguration;

struct ReloadExecutor;

impl CommandExecutor for ReloadExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let server = context.server;

            // Reload all JSON configurations
            *server.data.banned_ip_list.write().await =
                crate::data::banned_ip::BannedIpList::load();
            *server.data.banned_player_list.write().await =
                crate::data::banned_player::BannedPlayerList::load();
            *server.data.operator_config.write().await = crate::data::op::OperatorConfig::load();
            *server.data.whitelist_config.write().await =
                crate::data::whitelist::WhitelistConfig::load();

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
