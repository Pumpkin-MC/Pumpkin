use std::sync::atomic::Ordering;

use crate::command::CommandResult;
use crate::command::{
    CommandError, CommandExecutor, CommandSender, args::ConsumedArgs, tree::CommandTree,
    tree::builder::literal,
};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

const NAMES: [&str; 1] = ["debug"];

const DESCRIPTION: &str = "Starts or stops a debugging session.";

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
                    "commands.debug.alreadyRunning",
                    [],
                )));
            }

            server.tick_profiler.reset_slow_count();
            server.tick_profiler.set_enabled(true);

            sender
                .send_message(
                    TextComponent::translate("commands.debug.started", [])
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
                    "commands.debug.notRunning",
                    [],
                )));
            }

            server.tick_profiler.set_enabled(false);

            let snapshot = server.tick_profiler.snapshot();
            let tick_count = server.tick_count.load(Ordering::Relaxed);

            // Build a summary message similar to vanilla debug report
            let summary = format!(
                "Stopped debug profiling after {} ticks ({:.1}ms avg, {:.1}ms peak, {} slow ticks, {:.1}% budget)",
                snapshot.total_ticks,
                snapshot.total_avg_ms(),
                snapshot.total_peak_nanos as f64 / 1_000_000.0,
                snapshot.slow_tick_count,
                snapshot.budget_usage_percent(),
            );

            sender
                .send_message(
                    TextComponent::translate("commands.debug.stopped", [])
                        .color_named(NamedColor::Green),
                )
                .await;

            // Send detailed breakdown
            let detail = format!(
                "World: {:.2}ms avg | Player/Net: {:.2}ms avg | Total: {:.2}ms avg | Ticks: {} | Slow: {}",
                snapshot.world_avg_ms(),
                snapshot.player_avg_ms(),
                snapshot.total_avg_ms(),
                tick_count,
                snapshot.slow_tick_count,
            );
            sender
                .send_message(TextComponent::text(detail).color_named(NamedColor::Gray))
                .await;

            // Send budget usage line
            let budget_color = if snapshot.budget_usage_percent() > 80.0 {
                NamedColor::Red
            } else if snapshot.budget_usage_percent() > 50.0 {
                NamedColor::Yellow
            } else {
                NamedColor::Green
            };
            let budget_msg = format!(
                "Tick budget usage: {:.1}% of 50ms",
                snapshot.budget_usage_percent(),
            );
            sender
                .send_message(TextComponent::text(budget_msg).color_named(budget_color))
                .await;

            log::info!("{summary}");
            Ok(1)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(literal("start").execute(StartExecutor))
        .then(literal("stop").execute(StopExecutor))
}
