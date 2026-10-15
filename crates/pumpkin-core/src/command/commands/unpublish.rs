use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, command};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Stops publishing the server to the local network.";
const PERMISSION: &str = "minecraft:command.unpublish";

struct UnpublishExecutor;

impl CommandExecutor for UnpublishExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        // Dedicated servers are standalone processes that cannot be un-published like LAN worlds.
        // In vanilla UnpublishCommand.java, if !server.unpublishServer(), throws ERROR_NOT_PUBLISHED.
        context.source.send_feedback(
            TextComponent::translate(translation::java::COMMANDS_UNPUBLISH_NOTPUBLISHED, []),
            false,
        );

        Ok(0)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Four),
    ));

    dispatcher.register(
        command("unpublish", DESCRIPTION)
            .requires(PERMISSION)
            .executes(UnpublishExecutor),
    );
}
