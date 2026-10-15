use crate::command::argument_builder::{ArgumentBuilder, command};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use pumpkin_data::translation;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

const DESCRIPTION: &str = "Display Minecraft and server version information.";
const PERMISSION: &str = "minecraft:command.version";

struct VersionCommandExecutor;

impl CommandExecutor for VersionCommandExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let source = &context.source;

        // 1: Header
        source.send_feedback(
            TextComponent::translate(translation::java::COMMANDS_VERSION_HEADER, []),
            false,
        );

        // 2: ID
        source.send_feedback(
            TextComponent::translate(
                translation::java::COMMANDS_VERSION_ID,
                [TextComponent::text("26.3")],
            ),
            false,
        );

        // 3: Name
        source.send_feedback(
            TextComponent::translate(
                translation::java::COMMANDS_VERSION_NAME,
                [TextComponent::text("26.3")],
            ),
            false,
        );

        // 4: Data version
        source.send_feedback(
            TextComponent::translate(
                translation::java::COMMANDS_VERSION_DATA,
                [TextComponent::text("5023")],
            ),
            false,
        );

        // 5: Series
        source.send_feedback(
            TextComponent::translate(
                translation::java::COMMANDS_VERSION_SERIES,
                [TextComponent::text("main")],
            ),
            false,
        );

        // 6: Protocol
        source.send_feedback(
            TextComponent::translate(
                translation::java::COMMANDS_VERSION_PROTOCOL,
                [TextComponent::text("777"), TextComponent::text("0x309")],
            ),
            false,
        );

        // 7: Build Time
        source.send_feedback(
            TextComponent::translate(
                translation::java::COMMANDS_VERSION_BUILD_TIME,
                [TextComponent::text("2026-03-24")],
            ),
            false,
        );

        // 8: Pack Resource
        source.send_feedback(
            TextComponent::translate(
                translation::java::COMMANDS_VERSION_PACK_RESOURCE,
                [TextComponent::text("97.1")],
            ),
            false,
        );

        // 9: Pack Data
        source.send_feedback(
            TextComponent::translate(
                translation::java::COMMANDS_VERSION_PACK_DATA,
                [TextComponent::text("121.0")],
            ),
            false,
        );

        // 10: Stable
        source.send_feedback(
            TextComponent::translate(translation::java::COMMANDS_VERSION_STABLE_YES, []),
            false,
        );

        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::AllowAll,
    ));

    dispatcher.register(command("version").execute(&VersionCommandExecutor));
}
