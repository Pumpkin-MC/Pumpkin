use pumpkin_i18n::PUMPKIN_NAMESPACE;
use std::path::Path;

use pumpkin_util::{
    PermissionLvl,
    text::{TextComponent, color::NamedColor, hover::HoverEvent},
};

use crate::command::{
    CommandExecutor, CommandResult, CommandSender,
    args::{Arg, ConsumedArgs, simple::SimpleArgConsumer},
    dispatcher::CommandError,
    tree::{
        CommandTree,
        builder::{argument, literal, require},
    },
};

use crate::command::CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["plugin"];

const DESCRIPTION: &str = "commands.plugin.description";

const PLUGIN_NAME: &str = "plugin_name";

struct ListExecutor;

impl CommandExecutor for ListExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let plugins = server.plugin_manager.active_plugins().await;
            let locale = sender.get_locale();

            let mut message = if plugins.is_empty() {
                TextComponent::custom(PUMPKIN_NAMESPACE, "commands.plugin.no_plugins", locale, [])
            } else if plugins.len() == 1 {
                TextComponent::custom(PUMPKIN_NAMESPACE, "commands.plugin.one_plugin", locale, [])
            } else {
                TextComponent::custom(
                    PUMPKIN_NAMESPACE,
                    "commands.plugin.multiple_plugins",
                    locale,
                    [TextComponent::text(plugins.len().to_string())],
                )
            };

            for (i, metadata) in plugins.iter().enumerate() {
                let mut component = TextComponent::text(metadata.name.clone())
                    .color_named(NamedColor::Green)
                    .hover_event(HoverEvent::show_text(TextComponent::custom(
                        PUMPKIN_NAMESPACE,
                        "commands.plugin.hover_text",
                        locale,
                        [
                            TextComponent::text(metadata.version.clone()),
                            TextComponent::text(metadata.authors.join(", ")),
                            TextComponent::text(metadata.description.clone()),
                        ],
                    )));
                if i != plugins.len() - 1 {
                    component = component.add_child(TextComponent::custom(
                        PUMPKIN_NAMESPACE,
                        "commands.plugin.list.separator",
                        locale,
                        [],
                    ));
                }

                message = message.add_child(component);
            }

            sender.send_message(message).await;

            Ok(plugins.len() as i32)
        })
    }
}

struct LoadExecutor;

impl CommandExecutor for LoadExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(plugin_name)) = args.get(PLUGIN_NAME) else {
                return Err(InvalidConsumption(Some(PLUGIN_NAME.into())));
            };
            let locale = sender.get_locale();

            if server.plugin_manager.is_plugin_active(plugin_name).await {
                return Err(CommandError::CommandFailed(TextComponent::custom(
                    PUMPKIN_NAMESPACE,
                    "commands.plugin.already_loaded",
                    locale,
                    [TextComponent::text(plugin_name.to_string())],
                )));
            }

            let result = server
                .plugin_manager
                .try_load_plugin(Path::new(plugin_name))
                .await;

            match result {
                Ok(()) => {
                    sender
                        .send_message(
                            TextComponent::custom(
                                PUMPKIN_NAMESPACE,
                                "commands.plugin.loaded_successfully",
                                locale,
                                [TextComponent::text(plugin_name.to_string())],
                            )
                            .color_named(NamedColor::Green),
                        )
                        .await;
                    Ok(1)
                }
                Err(e) => Err(CommandError::CommandFailed(TextComponent::custom(
                    PUMPKIN_NAMESPACE,
                    "commands.plugin.failed_to_load",
                    locale,
                    [
                        TextComponent::text(plugin_name.to_string()),
                        TextComponent::text(e.to_string()),
                    ],
                ))),
            }
        })
    }
}

struct UnloadExecutor;

impl CommandExecutor for UnloadExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Simple(plugin_name)) = args.get(PLUGIN_NAME) else {
                return Err(InvalidConsumption(Some(PLUGIN_NAME.into())));
            };
            let locale = sender.get_locale();

            if !server.plugin_manager.is_plugin_active(plugin_name).await {
                return Err(CommandError::CommandFailed(TextComponent::custom(
                    PUMPKIN_NAMESPACE,
                    "commands.plugin.not_loaded",
                    locale,
                    [TextComponent::text(plugin_name.to_string())],
                )));
            }

            let result = server.plugin_manager.unload_plugin(plugin_name).await;

            match result {
                Ok(()) => {
                    sender
                        .send_message(
                            TextComponent::custom(
                                PUMPKIN_NAMESPACE,
                                "commands.plugin.unloaded_successfully",
                                locale,
                                [TextComponent::text(plugin_name.to_string())],
                            )
                            .color_named(NamedColor::Green),
                        )
                        .await;

                    Ok(1)
                }
                Err(e) => Err(CommandError::CommandFailed(TextComponent::custom(
                    PUMPKIN_NAMESPACE,
                    "commands.plugin.failed_to_unload",
                    locale,
                    [
                        TextComponent::text(plugin_name.to_string()),
                        TextComponent::text(e.to_string()),
                    ],
                ))),
            }
        })
    }
}

struct HotReloadExecutor(bool);

impl CommandExecutor for HotReloadExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let enabled = self.0;
            let locale = sender.get_locale();

            if enabled {
                if let Err(e) = server.plugin_manager.start_watcher().await {
                    return Err(CommandError::CommandFailed(TextComponent::custom(
                        PUMPKIN_NAMESPACE,
                        "commands.plugin.failed_to_start_watcher",
                        locale,
                        [TextComponent::text(e.to_string())],
                    )));
                }

                sender
                    .send_message(
                        TextComponent::custom(
                            PUMPKIN_NAMESPACE,
                            "commands.plugin.hotreload_enabled",
                            locale,
                            [],
                        )
                        .color_named(NamedColor::Green),
                    )
                    .await;
                sender
                    .send_message(
                        TextComponent::custom(
                            PUMPKIN_NAMESPACE,
                            "commands.plugin.hotreload_warning",
                            locale,
                            [],
                        )
                        .color_named(NamedColor::Red),
                    )
                    .await;
            } else {
                server.plugin_manager.stop_watcher().await;

                sender
                    .send_message(
                        TextComponent::custom(
                            PUMPKIN_NAMESPACE,
                            "commands.plugin.hotreload_disabled",
                            locale,
                            [],
                        )
                        .color_named(NamedColor::Green),
                    )
                    .await;
            }

            Ok(1)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        require(|sender| sender.has_permission_lvl(PermissionLvl::Three))
            .then(
                literal("load")
                    .then(argument(PLUGIN_NAME, SimpleArgConsumer).execute(LoadExecutor)),
            )
            .then(
                literal("unload")
                    .then(argument(PLUGIN_NAME, SimpleArgConsumer).execute(UnloadExecutor)),
            )
            .then(
                literal("hotreload")
                    .then(literal("enable").execute(HotReloadExecutor(true)))
                    .then(literal("disable").execute(HotReloadExecutor(false))),
            )
            .then(literal("list").execute(ListExecutor)),
    )
}
