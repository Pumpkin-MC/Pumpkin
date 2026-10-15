use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};

use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, command, literal};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Starts or stops JFR (Flight Recorder) profiling.";
const PERMISSION: &str = "minecraft:command.jfr";

static JFR_RUNNING: AtomicBool = AtomicBool::new(false);

struct JfrStartExecutor;

impl CommandExecutor for JfrStartExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        if JFR_RUNNING.swap(true, Ordering::SeqCst) {
            context.source.send_feedback(
                TextComponent::translate(translation::java::COMMANDS_JFR_START_FAILED, []),
                false,
            );
            return Ok(0);
        }

        context.source.send_feedback(
            TextComponent::translate(translation::java::COMMANDS_JFR_STARTED, []),
            false,
        );

        Ok(1)
    }
}

struct JfrStopExecutor;

impl CommandExecutor for JfrStopExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        if !JFR_RUNNING.swap(false, Ordering::SeqCst) {
            context.source.send_feedback(
                TextComponent::translate(
                    translation::java::COMMANDS_JFR_DUMP_FAILED,
                    [TextComponent::text("JFR is not currently recording")],
                ),
                false,
            );
            return Ok(0);
        }

        let recording_path = "debug/jfr-profile.jfr";
        let file_text = TextComponent::text(recording_path)
            .underlined()
            .click_event(ClickEvent::CopyToClipboard {
                value: Cow::from(recording_path),
            })
            .hover_event(HoverEvent::show_text(TextComponent::translate_cross(
                translation::java::CHAT_COPY_CLICK,
                translation::java::CHAT_COPY_CLICK,
                [],
            )));

        context.source.send_feedback(
            TextComponent::translate(translation::java::COMMANDS_JFR_STOPPED, [file_text]),
            false,
        );

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
        command("jfr", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("start").executes(JfrStartExecutor))
            .then(literal("stop").executes(JfrStopExecutor)),
    );
}
