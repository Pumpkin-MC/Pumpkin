use std::sync::{LazyLock, Mutex};
use std::time::Instant;

use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, command, literal};
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Starts or stops dedicated server performance profiling.";
const PERMISSION: &str = "minecraft:command.perf";

static PROFILING_START: LazyLock<Mutex<Option<Instant>>> = LazyLock::new(|| Mutex::new(None));

struct PerfStartExecutor;

impl CommandExecutor for PerfStartExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let mut guard = PROFILING_START
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if guard.is_some() {
            context.source.send_feedback(
                TextComponent::translate(translation::java::COMMANDS_PERF_ALREADYRUNNING, []),
                false,
            );
            return Ok(0);
        }

        *guard = Some(Instant::now());
        context.source.send_feedback(
            TextComponent::translate(translation::java::COMMANDS_PERF_STARTED, []),
            false,
        );

        Ok(1)
    }
}

struct PerfStopExecutor;

impl CommandExecutor for PerfStopExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let mut guard = PROFILING_START
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let Some(start_time) = guard.take() else {
            context.source.send_feedback(
                TextComponent::translate(translation::java::COMMANDS_PERF_NOTRUNNING, []),
                false,
            );
            return Ok(0);
        };

        let duration_secs = start_time.elapsed().as_secs_f64();
        let ticks = (duration_secs * 20.0).round() as u64;
        let tps = if duration_secs > 0.0 {
            ticks as f64 / duration_secs
        } else {
            20.0
        };

        context.source.send_feedback(
            TextComponent::translate(
                translation::java::COMMANDS_PERF_STOPPED,
                [
                    TextComponent::text(format!("{duration_secs:.2}")),
                    TextComponent::text(ticks.to_string()),
                    TextComponent::text(format!("{tps:.2}")),
                ],
            ),
            false,
        );

        Ok(ticks as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Four),
    ));

    dispatcher.register(
        command("perf", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("start").executes(PerfStartExecutor))
            .then(literal("stop").executes(PerfStopExecutor)),
    );
}
