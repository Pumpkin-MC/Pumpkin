use crate::command::CommandResult;
use crate::command::{
    CommandError, CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree,
    tree::builder::literal,
};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

const NAMES: [&str; 1] = ["perf"];

const DESCRIPTION: &str = "Captures performance metrics for 10 seconds.";

struct StartExecutor;

impl CommandExecutor for StartExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            if server.tick_profiler.is_enabled() {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "commands.perf.alreadyRunning",
                    [],
                )));
            }

            server.tick_profiler.reset_slow_count();
            server.tick_profiler.set_enabled(true);

            sender
                .send_message(
                    TextComponent::translate("commands.perf.started", [])
                        .color_named(NamedColor::Green),
                )
                .await;
            Ok(1)
        })
    }
}

struct StopExecutor;

impl CommandExecutor for StopExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            if !server.tick_profiler.is_enabled() {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "commands.perf.notRunning",
                    [],
                )));
            }

            server.tick_profiler.set_enabled(false);
            let snapshot = server.tick_profiler.snapshot();

            sender
                .send_message(
                    TextComponent::translate("commands.perf.stopped", [])
                        .color_named(NamedColor::Green),
                )
                .await;

            // Send performance summary
            let summary = format!(
                "Average tick: {:.2}ms (world: {:.2}ms, player/net: {:.2}ms) | Peak: {:.2}ms | Slow ticks: {}",
                snapshot.total_avg_ms(),
                snapshot.world_avg_ms(),
                snapshot.player_avg_ms(),
                snapshot.total_peak_nanos as f64 / 1_000_000.0,
                snapshot.slow_tick_count,
            );
            sender
                .send_message(TextComponent::text(summary).color_named(NamedColor::Gray))
                .await;

            log::info!(
                "Perf profiling stopped: {:.2}ms avg tick, {} slow ticks, {:.1}% budget",
                snapshot.total_avg_ms(),
                snapshot.slow_tick_count,
                snapshot.budget_usage_percent(),
            );

            Ok(1)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(literal("start").execute(StartExecutor))
        .then(literal("stop").execute(StopExecutor))
}
