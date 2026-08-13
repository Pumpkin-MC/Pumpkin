use std::sync::atomic::Ordering;

use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::CommandSender;
use crate::command::argument_builder::{ArgumentBuilder, command, literal};
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::server::perf_profiler::{
    PERF_PROFILE_DURATION, PerfProfileResult, StartPerfError, StopPerfError,
};

const DESCRIPTION: &str = "Captures info and metrics about the game for 10 seconds.";
const PERMISSION: &str = "minecraft:command.perf";

// Bedrock has no `/perf` command, so both editions use the Java keys.
const ALREADY_RUNNING_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_PERF_ALREADYRUNNING,
    translation::java::COMMANDS_PERF_ALREADYRUNNING,
);

const NOT_RUNNING_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_PERF_NOTRUNNING,
    translation::java::COMMANDS_PERF_NOTRUNNING,
);

async fn send_stopped_feedback(source: &CommandSource, result: PerfProfileResult) {
    let seconds = result.duration.as_secs_f64();
    let tps = result.ticks_per_second();
    let feedback = if matches!(source.output, CommandSender::Player(_)) {
        TextComponent::translate_cross(
            translation::java::COMMANDS_PERF_STOPPED,
            translation::java::COMMANDS_PERF_STOPPED,
            [
                TextComponent::text(format!("{seconds:.2}")),
                TextComponent::text(result.ticks.to_string()),
                TextComponent::text(format!("{tps:.2}")),
            ],
        )
    } else {
        // Non-player command sources cannot resolve translation placeholders
        // server-side, so they receive an already-rendered message.
        TextComponent::text(format!(
            "Stopped performance profiling after {seconds:.2} seconds and {} ticks ({tps:.2} ticks per second)",
            result.ticks
        ))
    };
    source.send_feedback(feedback, true).await;
}

struct PerfStartExecutor;

impl CommandExecutor for PerfStartExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let server = context.server();
            let current_tick = server.tick_count.load(Ordering::Relaxed);
            let generation = server.perf_profiler.start(current_tick).map_err(
                |StartPerfError::AlreadyRunning| {
                    ALREADY_RUNNING_ERROR_TYPE.create_without_context()
                },
            )?;

            context
                .source
                .send_feedback(
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_PERF_STARTED,
                        translation::java::COMMANDS_PERF_STARTED,
                        [],
                    ),
                    false,
                )
                .await;

            // Vanilla profiling runs end on their own after 10 seconds unless
            // `/perf stop` ends them early.
            let server = server.clone();
            let source = context.source.clone();
            tokio::spawn(async move {
                tokio::time::sleep(PERF_PROFILE_DURATION).await;
                let current_tick = server.tick_count.load(Ordering::Relaxed);
                if let Some(result) = server
                    .perf_profiler
                    .stop_if_generation(generation, current_tick)
                {
                    send_stopped_feedback(&source, result).await;
                }
            });

            Ok(0)
        })
    }
}

struct PerfStopExecutor;

impl CommandExecutor for PerfStopExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let server = context.server();
            let current_tick = server.tick_count.load(Ordering::Relaxed);
            let result =
                server
                    .perf_profiler
                    .stop(current_tick)
                    .map_err(|StopPerfError::NotRunning| {
                        NOT_RUNNING_ERROR_TYPE.create_without_context()
                    })?;

            send_stopped_feedback(&context.source, result).await;

            Ok(0)
        })
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
