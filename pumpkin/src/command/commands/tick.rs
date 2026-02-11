use pumpkin_data::translation;
use pumpkin_util::text::{
    TextComponent,
    color::{Color, NamedColor},
};
use std::sync::atomic::Ordering;

use crate::command::{
    CommandExecutor, CommandResult, CommandSender,
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

const fn rate_consumer() -> BoundedNumArgumentConsumer<f32> {
    BoundedNumArgumentConsumer::new()
        .name("rate")
        .min(1.0)
        .max(10000.0)
}

const fn time_consumer() -> TimeArgumentConsumer {
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
        sender: &CommandSender,
        server: &crate::server::Server,
        manager: &crate::server::tick_rate_manager::ServerTickRateManager,
    ) -> Result<i32, CommandError> {
        let tickrate = manager.tickrate();
        let avg_tick_nanos = server.get_average_tick_time_nanos();
        let avg_mspt_str = nanos_to_millis_string(avg_tick_nanos);

        if manager.is_sprinting() {
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TICK_STATUS_SPRINTING,
                    [],
                ))
                .await;
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TICK_QUERY_RATE_SPRINTING,
                    [
                        TextComponent::text(format!("{tickrate:.1}")),
                        TextComponent::text(avg_mspt_str),
                    ],
                ))
                .await;
        } else {
            Self::handle_non_sprinting_status(sender, manager, avg_tick_nanos).await;

            let target_mspt_str = nanos_to_millis_string(manager.nanoseconds_per_tick());
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TICK_QUERY_RATE_RUNNING,
                    [
                        TextComponent::text(format!("{tickrate:.1}")),
                        TextComponent::text(avg_mspt_str),
                        TextComponent::text(target_mspt_str),
                    ],
                ))
                .await;
        }

        Self::send_percentiles(sender, server).await;
        Ok(tickrate as i32)
    }
    async fn handle_non_sprinting_status(
        sender: &CommandSender,
        manager: &crate::server::tick_rate_manager::ServerTickRateManager,
        avg_tick_nanos: i64,
    ) {
        if manager.is_frozen() {
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TICK_STATUS_FROZEN,
                    [],
                ))
                .await;
        } else if avg_tick_nanos > manager.nanoseconds_per_tick() {
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TICK_STATUS_LAGGING,
                    [],
                ))
                .await;
        } else {
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TICK_STATUS_RUNNING,
                    [],
                ))
                .await;
        }
    }

    async fn send_percentiles(sender: &CommandSender, server: &crate::server::Server) {
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
                    translation::COMMANDS_TICK_QUERY_PERCENTILES,
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
        sender: &CommandSender,
        server: &crate::server::Server,
        manager: &crate::server::tick_rate_manager::ServerTickRateManager,
        ticks: i32,
    ) {
        if manager.step_game_if_paused(server, ticks).await {
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TICK_STEP_SUCCESS,
                    [TextComponent::text(ticks.to_string())],
                ))
                .await;
        } else {
            sender
                .send_message(
                    TextComponent::translate(translation::COMMANDS_TICK_STEP_FAIL, [])
                        .color_named(NamedColor::Red),
                )
                .await;
        }
    }
    async fn handle_sprint_command(
        sender: &CommandSender,
        server: &crate::server::Server,
        manager: &crate::server::tick_rate_manager::ServerTickRateManager,
        ticks: i32,
    ) {
        if manager
            .request_game_to_sprint(server, i64::from(ticks))
            .await
        {
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TICK_SPRINT_STOP_SUCCESS,
                    [],
                ))
                .await;
        }
        sender
            .send_message(TextComponent::translate(
                translation::COMMANDS_TICK_STATUS_SPRINTING,
                [],
            ))
            .await;
    }

    async fn handle_set_tick_rate<E>(
        sender: &CommandSender,
        server: &crate::server::Server,
        manager: &crate::server::tick_rate_manager::ServerTickRateManager,
        rate: f32,
    ) -> Result<i32, E> {
        manager.set_tick_rate(server, rate).await;
        sender
            .send_message(TextComponent::translate(
                translation::COMMANDS_TICK_RATE_SUCCESS,
                [TextComponent::text(format!("{rate:.1}"))],
            ))
            .await;
        Ok(rate as i32)
    }
}

impl CommandExecutor for TickExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let manager = &server.tick_rate_manager;
            match self.0 {
                SubCommand::Query => Self::handle_query(sender, server, manager).await,
                SubCommand::Rate => {
                    let rate = BoundedNumArgumentConsumer::<f32>::find_arg(args, "rate")??;
                    Self::handle_set_tick_rate(sender, server, manager, rate).await
                }
                SubCommand::RateLiteral(rate) => {
                    Self::handle_set_tick_rate(sender, server, manager, rate).await
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
                    Ok(freeze as i32)
                }
                SubCommand::StepDefault => {
                    Self::handle_step_command(sender, server, manager, 1).await;
                    Ok(1)
                }
                SubCommand::StepTimed => {
                    let ticks = TimeArgumentConsumer::find_arg(args, "time")?;
                    Self::handle_step_command(sender, server, manager, ticks).await;
                    Ok(1)
                }
                SubCommand::StepLiteral(ticks) => {
                    Self::handle_step_command(sender, server, manager, ticks).await;
                    Ok(1)
                }
                SubCommand::StepStop => {
                    if manager.stop_stepping(server).await {
                        sender
                            .send_message(TextComponent::translate(
                                translation::COMMANDS_TICK_SPRINT_STOP_SUCCESS,
                                [],
                            ))
                            .await;
                        Ok(1)
                    } else {
                        // TODO: send feedback as error without Err
                        sender
                            .send_message(TextComponent::translate(
                                translation::COMMANDS_TICK_SPRINT_STOP_FAIL,
                                [],
                            ))
                            .await;
                        Ok(0)
                    }
                }
                SubCommand::SprintTimed => {
                    Self::handle_sprint_command(
                        sender,
                        server,
                        manager,
                        TimeArgumentConsumer::find_arg(args, "time")?,
                    )
                    .await;
                    Ok(1)
                }
                SubCommand::SprintLiteral(ticks) => {
                    Self::handle_sprint_command(sender, server, manager, ticks).await;
                    Ok(1)
                }
                SubCommand::SprintStop => {
                    if manager.stop_sprinting(server).await {
                        sender
                            .send_message(TextComponent::translate(
                                translation::COMMANDS_TICK_SPRINT_STOP_SUCCESS,
                                [],
                            ))
                            .await;
                        Ok(1)
                    } else {
                        // TODO: send feedback as error without Err
                        sender
                            .send_message(
                                TextComponent::translate(
                                    translation::COMMANDS_TICK_SPRINT_STOP_FAIL,
                                    [],
                                )
                                .color(Color::Named(NamedColor::Red)),
                            )
                            .await;
                        Ok(0)
                    }
                }
            }
        })
    }
}

pub fn init_command_tree(default_tps: f32) -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(literal("query").execute(TickExecutor(SubCommand::Query)))
        .then(
            literal("rate")
                .then(literal("20").execute(TickExecutor(SubCommand::RateLiteral(default_tps))))
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
