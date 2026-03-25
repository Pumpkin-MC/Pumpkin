use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

use crate::command::argument_builder::{ArgumentBuilder, command};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::stop_server;

const DESCRIPTION: &str = "Stop the server.";

const PERMISSION: &str = "minecraft:command.stop";

struct StopCommandExecutor;

impl CommandExecutor for StopCommandExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            context
                .source
                .send_message(
                    TextComponent::translate(translation::COMMANDS_STOP_STOPPING, [])
                        .color_named(NamedColor::Red),
                )
                .await;
            stop_server();
            Ok(1)
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry
        .register_permission(Permission::new(
            PERMISSION,
            DESCRIPTION,
            PermissionDefault::Op(PermissionLvl::Four),
        ))
        .expect("Permission should have registered successfully");

    dispatcher.register(
        command("stop", DESCRIPTION)
            .requires(PERMISSION)
            .executes(StopCommandExecutor),
    );
}
