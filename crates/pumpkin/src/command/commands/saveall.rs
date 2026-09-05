use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use tracing::error;

use crate::command::argument_builder::{ArgumentBuilder, command, literal};
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

        let server_arc = context.server().clone();
        let server_clone = server_arc.clone();
        let source = context.source.clone();
        server_arc.spawn_task(async move {
            if let Err(err) = server_clone.save_all().await {
                error!("Failed to save server data: {err}");
                source.send_error(TextComponent::translate_cross(
                    translation::java::COMMANDS_SAVE_FAILED,
                    translation::bedrock::COMMANDS_SAVE_FAILED,
                    [],
                ));
            } else {
                source.send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_SAVE_SUCCESS,
                        translation::bedrock::COMMANDS_SAVE_SUCCESS,
                        [],
                    ),
                    true,
                );
            }
        });

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
            .executes(SaveAllExecutor)
            .then(literal("flush").executes(SaveAllExecutor)),
    );
}
