use async_trait::async_trait;
use pumpkin_config::BASIC_CONFIG;
use pumpkin_util::text::{
    TextComponent,
    color::{Color, NamedColor},
};
use std::sync::atomic::Ordering;

use crate::command::{
    CommandExecutor, CommandSender,
    args::{
        ConsumedArgs, FindArg, bounded_num::BoundedNumArgumentConsumer, time::TimeArgumentConsumer,
    },
    dispatcher::CommandError,
    tree::{
        CommandTree,
        builder::{argument, literal},
    },
};

const NAMES: [&str; 1] = ["tick"];
const DESCRIPTION: &str = "Controls or queries the game's ticking state.";

// Helper function to format nanoseconds to milliseconds with 2 decimal places
fn nanos_to_millis_string(nanos: i64) -> String {
    format!("{:.2}", nanos as f64 / 1_000_000.0)
}

fn rate_consumer() -> BoundedNumArgumentConsumer<f32> {
    BoundedNumArgumentConsumer::new()
        .name("rate")
        .min(1.0)
        .max(10000.0)
}

fn time_consumer() -> TimeArgumentConsumer {
    TimeArgumentConsumer
}

enum SubCommand {
    Query,
    Rate,
    RateLiteral(f32),
    Freeze(bool),
    StepDefault,
    StepTimed,
    StepLiteral(i32),
    StepStop,
    SprintTimed,
    SprintLiteral(i32),
    SprintStop,
}

struct TickExecutor(SubCommand);

impl TickExecutor {
    async fn handle_query(
        &self,
        sender: &mut CommandSender,
        server: &crate::server::Server,
        manager: &crate::server::tick_rate_manager::ServerTickRateManager,
    ) -> Result<(), CommandError> {
        let tickrate = manager.tickrate();
        let avg_tick_nanos = server.get_average_tick_time_nanos();
        let avg_mspt_str = nanos_to_millis_string(avg_tick_nanos);

        if manager.is_sprinting() {
            sender
                .send_message(TextComponent::translate(
                    "commands.tick.status.sprinting",
                    [],
                ))
                .await;
            sender
                .send_message(TextComponent::translate(
                    "commands.tick.query.rate.sprinting",
                    [
                        TextComponent::text(format!("{tickrate:.1}")),
                        TextComponent::text(avg_mspt_str),
                    ],
                ))
                .await;
        } else {
            self.handle_non_sprinting_status(sender, manager, avg_tick_nanos)
                .await;

            let target_mspt_str = nanos_to_millis_string(manager.nanoseconds_per_tick());
            sender
                .send_message(TextComponent::translate(
                    "commands.tick.query.rate.running",
                    [
                        TextComponent::text(format!("{tickrate:.1}")),
                        TextComponent::text(avg_mspt_str),
                        TextComponent::text(target_mspt_str),
                    ],
                ))
                .await;
        }

        self.send_percentiles(sender, server).await;
        Ok(())
    }
    async fn handle_non_sprinting_status(
        &self,
        sender: &mut CommandSender,
        manager: &crate::server::tick_rate_manager::ServerTickRateManager,
        avg_tick_nanos: i64,
    ) {
        if manager.is_frozen() {
            sender
                .send_message(TextComponent::translate("commands.tick.status.frozen", []))
                .await;
        } else if avg_tick_nanos > manager.nanoseconds_per_tick() {
            sender
                .send_message(TextComponent::translate("commands.tick.status.lagging", []))
                .await;
        } else {
            sender
                .send_message(TextComponent::translate("commands.tick.status.running", []))
                .await;
        }
    }

    async fn send_percentiles(&self, sender: &mut CommandSender, server: &crate::server::Server) {
        let tick_count = server.tick_count.load(Ordering::Relaxed);
        let sample_size = (tick_count as usize).min(100);

        if sample_size > 0 {
            let mut tick_times = server.get_tick_times_nanos_copy().await;
            let relevant_ticks = &mut tick_times[..sample_size];
            relevant_ticks.sort_unstable();

            let p50_nanos = relevant_ticks[sample_size / 2];
            let p95_nanos = relevant_ticks[(sample_size as f32 * 0.95).floor() as usize];
            let p99_nanos = relevant_ticks[(sample_size as f32 * 0.99).floor() as usize];

            sender
                .send_message(TextComponent::translate(
                    "commands.tick.query.percentiles",
                    [
                        TextComponent::text(nanos_to_millis_string(p50_nanos)),
                        TextComponent::text(nanos_to_millis_string(p95_nanos)),
                        TextComponent::text(nanos_to_millis_string(p99_nanos)),
                        TextComponent::text(sample_size.to_string()),
                    ],
                ))
                .await;
        }
    }
    async fn handle_step_command(
        &self,
        sender: &mut CommandSender,
        server: &crate::server::Server,
        manager: &crate::server::tick_rate_manager::ServerTickRateManager,
        ticks: i32,
    ) -> Result<(), CommandError> {
        if manager.step_game_if_paused(server, ticks).await {
            sender
                .send_message(TextComponent::translate(
                    "commands.tick.step.success",
                    [TextComponent::text(ticks.to_string())],
                ))
                .await;
        } else {
            sender
                .send_message(
                    TextComponent::translate("commands.tick.step.fail", [])
                        .color_named(NamedColor::Red),
                )
                .await;
        }
        Ok(())
    }
    async fn handle_sprint_command(
        &self,
        sender: &mut CommandSender,
        server: &crate::server::Server,
        manager: &crate::server::tick_rate_manager::ServerTickRateManager,
        ticks: i32,
    ) -> Result<(), CommandError> {
        if manager
            .request_game_to_sprint(server, i64::from(ticks))
            .await
        {
            sender
                .send_message(TextComponent::translate(
                    "commands.tick.sprint.stop.success",
                    [],
                ))
                .await;
        }
        sender
            .send_message(TextComponent::translate(
                "commands.tick.status.sprinting",
                [],
            ))
            .await;
        Ok(())
    }
}

#[async_trait]
impl CommandExecutor for TickExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let manager = &server.tick_rate_manager;

        match self.0 {
            SubCommand::Query => self.handle_query(sender, server, manager).await,
            SubCommand::Rate => {
                let rate = BoundedNumArgumentConsumer::<f32>::find_arg(args, "rate")??;
                manager.set_tick_rate(server, rate).await;
                sender
                    .send_message(TextComponent::translate(
                        "commands.tick.rate.success",
                        [TextComponent::text(format!("{rate:.1}"))],
                    ))
                    .await;
                Ok(())
            }
            SubCommand::RateLiteral(rate) => {
                manager.set_tick_rate(server, rate).await;
                sender
                    .send_message(TextComponent::translate(
                        "commands.tick.rate.success",
                        [TextComponent::text(format!("{rate:.1}"))],
                    ))
                    .await;
                Ok(())
            }
            SubCommand::Freeze(freeze) => {
                manager.set_frozen(server, freeze).await;
                let message_key = if freeze {
                    "commands.tick.status.frozen"
                } else {
                    "commands.tick.status.running"
                };
                sender
                    .send_message(TextComponent::translate(message_key, []))
                    .await;
                Ok(())
            }
            SubCommand::StepDefault => self.handle_step_command(sender, server, manager, 1).await,
            SubCommand::StepTimed => {
                let ticks = TimeArgumentConsumer::find_arg(args, "time")?;
                self.handle_step_command(sender, server, manager, ticks)
                    .await
            }
            SubCommand::StepLiteral(ticks) => {
                self.handle_step_command(sender, server, manager, ticks)
                    .await
            }
            SubCommand::StepStop => {
                if manager.stop_stepping(server).await {
                    sender
                        .send_message(TextComponent::translate(
                            "commands.tick.step.stop.success",
                            [],
                        ))
                        .await;
                } else {
                    sender
                        .send_message(TextComponent::translate("commands.tick.step.stop.fail", []))
                        .await;
                }
                Ok(())
            }
            SubCommand::SprintTimed => {
                let ticks = TimeArgumentConsumer::find_arg(args, "time")?;
                self.handle_sprint_command(sender, server, manager, ticks)
                    .await
            }
            SubCommand::SprintLiteral(ticks) => {
                self.handle_sprint_command(sender, server, manager, ticks)
                    .await
            }
            SubCommand::SprintStop => {
                if manager.stop_sprinting(server).await {
                    sender
                        .send_message(TextComponent::translate(
                            "commands.tick.sprint.stop.success",
                            [],
                        ))
                        .await;
                } else {
                    sender
                        .send_message(
                            TextComponent::translate("commands.tick.sprint.stop.fail", [])
                                .color(Color::Named(NamedColor::Red)),
                        )
                        .await;
                }
                Ok(())
            }
        }
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(literal("query").execute(TickExecutor(SubCommand::Query)))
        .then(
            literal("rate")
                .then(
                    literal("20").execute(TickExecutor(SubCommand::RateLiteral(BASIC_CONFIG.tps))),
                )
                .then(argument("rate", rate_consumer()).execute(TickExecutor(SubCommand::Rate))),
        )
        .then(literal("freeze").execute(TickExecutor(SubCommand::Freeze(true))))
        .then(literal("unfreeze").execute(TickExecutor(SubCommand::Freeze(false))))
        .then(
            literal("step")
                .then(literal("stop").execute(TickExecutor(SubCommand::StepStop)))
                .then(literal("1s").execute(TickExecutor(SubCommand::StepLiteral(20))))
                .then(literal("1t").execute(TickExecutor(SubCommand::StepLiteral(1))))
                .then(
                    argument("time", time_consumer()).execute(TickExecutor(SubCommand::StepTimed)),
                )
                .execute(TickExecutor(SubCommand::StepDefault)),
        )
        .then(
            literal("sprint")
                .then(literal("stop").execute(TickExecutor(SubCommand::SprintStop)))
                .then(literal("1d").execute(TickExecutor(SubCommand::SprintLiteral(24000))))
                .then(literal("3d").execute(TickExecutor(SubCommand::SprintLiteral(72000))))
                .then(literal("60s").execute(TickExecutor(SubCommand::SprintLiteral(1200))))
                .then(
                    argument("time", time_consumer())
                        .execute(TickExecutor(SubCommand::SprintTimed)),
                ),
        )
}
