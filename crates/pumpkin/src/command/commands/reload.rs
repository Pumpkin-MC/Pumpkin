use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use tracing::error;

use crate::command::argument_builder::{ArgumentBuilder, command};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Reloads the server configuration.";

const PERMISSION: &str = "minecraft:command.reload";

const RELOAD_FAILED_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_RELOAD_FAILURE,
    translation::bedrock::COMMANDS_RELOAD_ERROR,
);

struct ReloadCommandExecutor;

impl CommandExecutor for ReloadCommandExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_RELOAD_SUCCESS,
                        translation::bedrock::COMMANDS_RELOAD_SUCCESS,
                        [],
                    ),
                    true,
                )
                .await;

            let server = context.server();

            if let Err(err) = server.reload_config().await {
                error!("Failed to reload server configuration: {err}");
                return Err(RELOAD_FAILED_ERROR_TYPE.create_without_context());
            }

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
            .executes(ReloadCommandExecutor),
    );
}
