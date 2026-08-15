use std::sync::Arc;

use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use tracing::error;

use crate::command::argument_builder::{ArgumentBuilder, command};
use crate::command::client_suggestions;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::net::ClientPlatform;
use crate::server::Server;

const DESCRIPTION: &str = "Reloads the server's plugins.";
const PERMISSION: &str = "minecraft:command.reload";

// Bedrock's own reload command is client-side only, so both editions use the Java key.
const RELOAD_FAILED_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_RELOAD_FAILURE,
    translation::java::COMMANDS_RELOAD_FAILURE,
);

/// Pushes the rebuilt command tree to every online player, so commands that
/// plugins registered or dropped during the reload show up right away.
async fn resend_command_tree(server: &Arc<Server>) {
    let command_dispatcher = server.command_dispatcher.read().await;
    for player in server.get_all_players() {
        if let ClientPlatform::Bedrock(_) = player.client.as_ref() {
            client_suggestions::send_bedrock_commands_packet(&player, server, &command_dispatcher)
                .await;
        } else {
            client_suggestions::send_c_commands_packet(&player, server, &command_dispatcher).await;
        }
    }
}

struct ReloadExecutor;

impl CommandExecutor for ReloadExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_RELOAD_SUCCESS,
                        translation::java::COMMANDS_RELOAD_SUCCESS,
                        [],
                    ),
                    true,
                )
                .await;

            let server = context.server();

            // Vanilla keeps the old data when a reload fails. Unloading is the
            // destructive half, so a failure there is what we report.
            if let Err(err) = server.plugin_manager.unload_all_plugins().await {
                error!("Failed to unload plugins during /reload: {err}");
                resend_command_tree(server).await;
                return Err(RELOAD_FAILED_ERROR_TYPE.create_without_context());
            }

            let result = server.plugin_manager.load_plugins(server).await;
            resend_command_tree(server).await;

            if let Err(err) = result {
                error!("Failed to load plugins during /reload: {err}");
                return Err(RELOAD_FAILED_ERROR_TYPE.create_without_context());
            }

            Ok(0)
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
