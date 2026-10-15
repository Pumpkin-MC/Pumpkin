use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::core::bool::BoolArgumentType;
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Publishes the server to the local network.";
const PERMISSION: &str = "minecraft:command.publish";

const ARG_ALLOW_COMMANDS: &str = "allowCommands";
const ARG_PORT: &str = "port";

struct PublishExecutor {
    has_allow_commands: bool,
    has_port: bool,
}

impl CommandExecutor for PublishExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let server_port = context
            .server()
            .advanced_config
            .networking
            .java
            .address
            .port();

        let port = if self.has_port {
            IntegerArgumentType::get(context, ARG_PORT).map_or(server_port, |p: i32| p as u16)
        } else {
            server_port
        };

        let _allow_commands = if self.has_allow_commands {
            BoolArgumentType::get(context, ARG_ALLOW_COMMANDS).unwrap_or(false)
        } else {
            false
        };

        // On a dedicated server like Pumpkin, the server is always published and listening
        // on its network port. Per vanilla Minecraft parity (PublishCommand.java):
        // if (server.isPublished() && server.getPort() > -1) -> ERROR_ALREADY_PUBLISHED_LAN
        context.source.send_feedback(
            TextComponent::translate(
                translation::java::COMMANDS_PUBLISH_ALREADYPUBLISHED_LAN,
                [TextComponent::text(port.to_string())],
            ),
            false,
        );

        Ok(port as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Four),
    ));

    dispatcher.register(
        command("publish", DESCRIPTION)
            .requires(PERMISSION)
            .executes(PublishExecutor {
                has_allow_commands: false,
                has_port: false,
            })
            .then(
                argument(ARG_ALLOW_COMMANDS, BoolArgumentType)
                    .executes(PublishExecutor {
                        has_allow_commands: true,
                        has_port: false,
                    })
                    .then(
                        argument(ARG_PORT, IntegerArgumentType::new(0, 65535)).executes(
                            PublishExecutor {
                                has_allow_commands: true,
                                has_port: true,
                            },
                        ),
                    ),
            ),
    );
}
