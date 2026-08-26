use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use tracing::error;

use crate::command::argument_builder::{ArgumentBuilder, command};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Saves the server to disk.";

const PERMISSION: &str = "minecraft:command.save-all";

struct SaveAllExecutor;

impl CommandExecutor for SaveAllExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_SAVE_SAVING,
                translation::bedrock::COMMANDS_SAVE_START,
                [],
            ),
            false,
        );

        let server = context.server();
        if let Err(err) = futures::executor::block_on(server.save_all()) {
            error!("Failed to save server data: {err}");
            context.source.send_error(TextComponent::translate_cross(
                translation::java::COMMANDS_SAVE_FAILED,
                translation::bedrock::COMMANDS_SAVE_FAILED,
                [],
            ));
        } else {
            context.source.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_SAVE_SUCCESS,
                    translation::bedrock::COMMANDS_SAVE_SUCCESS,
                    [],
                ),
                true,
            );
        }

        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Four),
    ));

    dispatcher.register(
        command("save-all", DESCRIPTION)
            .requires(PERMISSION)
            .executes(SaveAllExecutor),
    );
}
