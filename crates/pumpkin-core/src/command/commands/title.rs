use pumpkin_protocol::java::client::play::CClearTitle;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::component::ComponentArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::time::TimeArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;
use crate::entity::player::TitleMode;

const DESCRIPTION: &str = "Displays a title.";
const PERMISSION: &str = "minecraft:command.title";

struct ClearOrResetExecutor(bool);

impl CommandExecutor for ClearOrResetExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_players(context, "targets")?;
        let reset = self.0;

        for target in &targets {
            target.try_send_client_packet(&CClearTitle::new(reset));
        }

        let msg = if targets.len() == 1 {
            let text = if reset {
                "commands.title.reset.single"
            } else {
                "commands.title.cleared.single"
            };
            TextComponent::translate_cross(text, text, [targets[0].as_ref().get_display_name()])
        } else {
            let text = if reset {
                "commands.title.reset.multiple"
            } else {
                "commands.title.cleared.multiple"
            };
            TextComponent::translate_cross(
                text,
                text,
                [TextComponent::text(targets.len().to_string())],
            )
        };
        context.source.send_feedback(msg, true);

        Ok(targets.len() as i32)
    }
}

struct TitleExecutor(TitleMode);

impl CommandExecutor for TitleExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_players(context, "targets")?;
        let text = ComponentArgumentType::get(context, "title")?;
        let mode = self.0;

        for target in &targets {
            target.show_title(&text, &mode);
        }

        let mode_name = format!("{mode:?}").to_lowercase();
        let msg = if targets.len() == 1 {
            TextComponent::translate_cross(
                format!("commands.title.show.{mode_name}.single"),
                format!("commands.title.show.{mode_name}.single"),
                [targets[0].as_ref().get_display_name()],
            )
        } else {
            TextComponent::translate_cross(
                format!("commands.title.show.{mode_name}.multiple"),
                format!("commands.title.show.{mode_name}.multiple"),
                [TextComponent::text(targets.len().to_string())],
            )
        };
        context.source.send_feedback(msg, true);

        Ok(targets.len() as i32)
    }
}

struct TimesTitleExecutor;

impl CommandExecutor for TimesTitleExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_players(context, "targets")?;
        let fade_in = TimeArgumentType::get(context, "fadeIn")?;
        let stay = TimeArgumentType::get(context, "stay")?;
        let fade_out = TimeArgumentType::get(context, "fadeOut")?;

        for target in &targets {
            target.send_title_animation(fade_in, stay, fade_out);
        }

        let msg = if targets.len() == 1 {
            TextComponent::translate_cross(
                "commands.title.times.single",
                "commands.title.times.single",
                [targets[0].as_ref().get_display_name()],
            )
        } else {
            TextComponent::translate_cross(
                "commands.title.times.multiple",
                "commands.title.times.multiple",
                [TextComponent::text(targets.len().to_string())],
            )
        };
        context.source.send_feedback(msg, true);

        Ok(targets.len() as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("title", DESCRIPTION).requires(PERMISSION).then(
            argument("targets", EntityArgumentType::Players)
                .then(literal("clear").executes(ClearOrResetExecutor(false)))
                .then(literal("reset").executes(ClearOrResetExecutor(true)))
                .then(
                    literal("title").then(
                        argument("title", ComponentArgumentType)
                            .executes(TitleExecutor(TitleMode::Title)),
                    ),
                )
                .then(
                    literal("subtitle").then(
                        argument("title", ComponentArgumentType)
                            .executes(TitleExecutor(TitleMode::SubTitle)),
                    ),
                )
                .then(
                    literal("actionbar").then(
                        argument("title", ComponentArgumentType)
                            .executes(TitleExecutor(TitleMode::ActionBar)),
                    ),
                )
                .then(
                    literal("times").then(
                        argument("fadeIn", TimeArgumentType::any()).then(
                            argument("stay", TimeArgumentType::any()).then(
                                argument("fadeOut", TimeArgumentType::any())
                                    .executes(TimesTitleExecutor),
                            ),
                        ),
                    ),
                ),
        ),
    );
}
