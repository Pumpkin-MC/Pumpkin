use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::text::hover::HoverEvent;

use crate::command::argument_builder::{ArgumentBuilder, command};
use crate::command::commands::plugin::{hover_text, status_color};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Lists all plugins on the server.";
const PERMISSION: &str = "pumpkin:command.plugins";

struct PluginsExecutor;

impl CommandExecutor for PluginsExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let server_arc = context.server().clone();
        let source_clone = context.source.clone();
        let server_clone = server_arc.clone();

        server_arc.spawn_task(async move {
            let entries = server_clone.plugin_manager.plugin_entries().await;

            let message_text = if entries.is_empty() {
                TextComponent::text("No plugins are installed on the server.")
                    .color_named(NamedColor::Red)
            } else {
                let mut base_message =
                    TextComponent::text(format!("Plugins ({}): ", entries.len()))
                        .color_named(NamedColor::White);

                for (i, entry) in entries.iter().enumerate() {
                    let plugin_component = TextComponent::text(entry.name().into_owned())
                        .color_named(status_color(&entry.status))
                        .hover_event(HoverEvent::show_text(TextComponent::text(hover_text(
                            entry,
                        ))));

                    let separator = if i > 0 { ", " } else { " " };
                    base_message = base_message
                        .add_child(TextComponent::text(separator).color_named(NamedColor::White))
                        .add_child(plugin_component);
                }

                base_message
            };

            source_clone.send_feedback(message_text, false);
        });

        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Three),
    ));

    dispatcher.register(
        command("plugins", DESCRIPTION)
            .requires(PERMISSION)
            .executes(PluginsExecutor),
    );
}
