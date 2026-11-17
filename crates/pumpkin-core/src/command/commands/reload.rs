use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, command};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Reloads the server's datapacks.";
const PERMISSION: &str = "minecraft:command.reload";

struct ReloadExecutor;

impl CommandExecutor for ReloadExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        // Vanilla announces the reload before doing it, so the feedback
        // arrives even though reloading takes a moment.
        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_RELOAD_SUCCESS,
                translation::java::COMMANDS_RELOAD_SUCCESS,
                [],
            ),
            true,
        );

        let server = context.server().clone();
        server.reload_datapacks(&server);

        Ok(0)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
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
