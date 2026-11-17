use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::text::hover::HoverEvent;

use crate::command::argument_builder::{ArgumentBuilder, command};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Lists all plugins loaded on the server.";
const PERMISSION: &str = "pumpkin:command.plugins";

struct PluginsExecutor;

impl CommandExecutor for PluginsExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let server_arc = context.server();
        let plugins = server_arc.plugin_manager.active_plugins();

        let message_text = if plugins.is_empty() {
            TextComponent::text("No plugins are loaded on the server.").color_named(NamedColor::Red)
        } else {
            let mut base_message = TextComponent::text(format!("Plugins ({}): ", plugins.len()))
                .color_named(NamedColor::White);

            for (i, plugin) in plugins.iter().enumerate() {
                let name = &plugin.name;
                let version = plugin.version.strip_prefix('v').unwrap_or(&plugin.version);
                let author_text = plugin.authors.join(", ");
                let description = &plugin.description;

                let hover_text = format!(
                    "Version: {version}\nAuthors: {author_text}\nDescription: {description}"
                );

                let plugin_component = TextComponent::text(name.clone())
                    .color_named(NamedColor::Green)
                    .hover_event(HoverEvent::show_text(TextComponent::text(hover_text)));

                if i > 0 {
                    base_message = base_message
                        .add_child(TextComponent::text(", ").color_named(NamedColor::White));
                } else {
                    base_message = base_message
                        .add_child(TextComponent::text(" ").color_named(NamedColor::White));
                }
                base_message = base_message.add_child(plugin_component);
            }

            base_message
        };

        context.source.send_feedback(message_text, false);

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
