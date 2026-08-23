use std::sync::atomic::Ordering;

use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::CommandSender;
use crate::command::argument_builder::{ArgumentBuilder, command, literal};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::server::debug_profiler::{StartDebugProfileError, StopDebugProfileError};

const DESCRIPTION: &str = "Starts or stops a tick profiling session.";
const PERMISSION: &str = "minecraft:command.debug";

const ALREADY_RUNNING_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_DEBUG_ALREADYRUNNING,
    translation::java::COMMANDS_DEBUG_ALREADYRUNNING,
);

const NOT_RUNNING_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_DEBUG_NOTRUNNING,
    translation::bedrock::COMMANDS_DEBUG_NOTSTARTED,
);

struct DebugStartExecutor;

impl CommandExecutor for DebugStartExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let server = context.server();
            let current_tick = server.tick_count.load(Ordering::Relaxed);
            server.debug_profiler.start(current_tick).map_err(
                |StartDebugProfileError::AlreadyRunning| {
                    ALREADY_RUNNING_ERROR_TYPE.create_without_context()
                },
            )?;

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_DEBUG_STARTED,
                        translation::bedrock::COMMANDS_DEBUG_START,
                        [],
                    ),
                    true,
                )
                .await;

            Ok(1)
        })
    }
}

struct DebugStopExecutor;

impl CommandExecutor for DebugStopExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let server = context.server();
            let current_tick = server.tick_count.load(Ordering::Relaxed);
            let result = server.debug_profiler.stop(current_tick).map_err(
                |StopDebugProfileError::NotRunning| NOT_RUNNING_ERROR_TYPE.create_without_context(),
            )?;

            let seconds = result.duration.as_secs_f64();
            let tps = result.ticks_per_second();
            let arguments = [
                TextComponent::text(format!("{seconds:.2}")),
                TextComponent::text(result.ticks.to_string()),
                TextComponent::text(format!("{tps:.2}")),
            ];
            let feedback = if matches!(context.source.output, CommandSender::Player(_)) {
                TextComponent::translate_cross(
                    translation::java::COMMANDS_DEBUG_STOPPED,
                    translation::bedrock::COMMANDS_DEBUG_STOP,
                    arguments,
                )
            } else {
                // Bedrock's debug-stop translation uses printf-style placeholders that the
                // server-side console renderer cannot resolve. Non-player command sources need
                // an already-rendered message; players still receive their native translation.
                TextComponent::text(format!(
                    "Stopped tick profiling after {seconds:.2} second(s) and {} tick(s) ({tps:.2} tick(s) per second)",
                    result.ticks
                ))
            };
            context.source.send_feedback(feedback, true).await;

            Ok(result.command_result())
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Three),
    ));

    dispatcher.register(
        command("debug", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("start").executes(DebugStartExecutor))
            .then(literal("stop").executes(DebugStopExecutor)),
    );
}
