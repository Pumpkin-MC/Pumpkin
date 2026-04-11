use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::float::FloatArgumentType;
use crate::command::argument_types::time::TimeArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::{
    TextComponent,
    color::{Color, NamedColor},
};
use std::sync::atomic::Ordering;

const DESCRIPTION: &str = "Controls or queries the game's ticking state.";
const PERMISSION: &str = "minecraft:command.tick";

// Helper function to format nanoseconds to milliseconds with 2 decimal places
fn nanos_to_millis_string(nanos: i64) -> String {
    format!("{:.2}", nanos as f64 / 1_000_000.0)
}

enum SubCommand {
    Query,
    Rate,
    Freeze(bool),
    StepDefault,
    StepTimed,
    StepStop,
    SprintTimed,
    SprintStop,
}

struct TickExecutor(SubCommand);

impl TickExecutor {
    async fn handle_query(
        source: &CommandSource,
        manager: &crate::server::tick_rate_manager::ServerTickRateManager,
    ) -> Result<i32, CommandSyntaxError> {
        let tickrate = manager.tickrate();
        let avg_tick_nanos = source.server().get_average_tick_time_nanos();
        let avg_mspt_str = nanos_to_millis_string(avg_tick_nanos);

        if manager.is_sprinting() {
            source
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TICK_STATUS_SPRINTING,
                    [],
                ))
                .await;
            source
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TICK_QUERY_RATE_SPRINTING,
                    [
                        TextComponent::text(format!("{tickrate:.1}")),
                        TextComponent::text(avg_mspt_str),
                    ],
                ))
                .await;
        } else {
            Self::handle_non_sprinting_status(source, manager, avg_tick_nanos).await;

            let target_mspt_str = nanos_to_millis_string(manager.nanoseconds_per_tick());
            source
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

        Self::send_percentiles(source, source.server()).await;
        Ok(tickrate as i32)
    }
    async fn handle_non_sprinting_status(
        sender: &CommandSource,
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

    async fn send_percentiles(sender: &CommandSource, server: &crate::server::Server) {
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
        source: &CommandSource,
        manager: &crate::server::tick_rate_manager::ServerTickRateManager,
        ticks: i32,
    ) {
        if manager.step_game_if_paused(source.server(), ticks).await {
            source
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TICK_STEP_SUCCESS,
                    [TextComponent::text(ticks.to_string())],
                ))
                .await;
        } else {
            source
                .send_message(
                    TextComponent::translate(translation::COMMANDS_TICK_STEP_FAIL, [])
                        .color_named(NamedColor::Red),
                )
                .await;
        }
    }
    async fn handle_sprint_command(
        source: &CommandSource,
        manager: &crate::server::tick_rate_manager::ServerTickRateManager,
        ticks: i32,
    ) {
        if manager
            .request_game_to_sprint(source.server(), i64::from(ticks))
            .await
        {
            source
                .send_message(TextComponent::translate(
                    translation::COMMANDS_TICK_SPRINT_STOP_SUCCESS,
                    [],
                ))
                .await;
        }
        source
            .send_message(TextComponent::translate(
                translation::COMMANDS_TICK_STATUS_SPRINTING,
                [],
            ))
            .await;
    }

    async fn handle_set_tick_rate(
        source: &CommandSource,
        manager: &crate::server::tick_rate_manager::ServerTickRateManager,
        rate: f32,
    ) -> Result<i32, CommandSyntaxError> {
        manager.set_tick_rate(source.server(), rate).await;
        source
            .send_message(TextComponent::translate(
                translation::COMMANDS_TICK_RATE_SUCCESS,
                [TextComponent::text(format!("{rate:.1}"))],
            ))
            .await;
        Ok(rate as i32)
    }
}

impl CommandExecutor for TickExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let manager = &context.server().tick_rate_manager;
            let source = context.source.as_ref();
            let server = source.server();
            match self.0 {
                SubCommand::Query => Self::handle_query(source, manager).await,
                SubCommand::Rate => {
                    let rate = FloatArgumentType::get(context, "rate")?;
                    Self::handle_set_tick_rate(source, manager, rate).await
                }
                SubCommand::Freeze(freeze) => {
                    manager.set_frozen(server, freeze).await;
                    let message_key = if freeze {
                        "commands.tick.status.frozen"
                    } else {
                        "commands.tick.status.running"
                    };
                    source
                        .send_message(TextComponent::translate(message_key, []))
                        .await;
                    Ok(freeze as i32)
                }
                SubCommand::StepDefault => {
                    Self::handle_step_command(source, manager, 1).await;
                    Ok(1)
                }
                SubCommand::StepTimed => {
                    let ticks = TimeArgumentType::get(context, "time")?;
                    Self::handle_step_command(source, manager, ticks).await;
                    Ok(1)
                }
                SubCommand::StepStop => {
                    if manager.stop_stepping(server).await {
                        source
                            .send_message(TextComponent::translate(
                                translation::COMMANDS_TICK_SPRINT_STOP_SUCCESS,
                                [],
                            ))
                            .await;
                        Ok(1)
                    } else {
                        // TODO: send feedback as error without Err
                        source
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
                        source,
                        manager,
                        TimeArgumentType::get(context, "time")?,
                    )
                    .await;
                    Ok(1)
                }
                SubCommand::SprintStop => {
                    if manager.stop_sprinting(server).await {
                        source
                            .send_message(TextComponent::translate(
                                translation::COMMANDS_TICK_SPRINT_STOP_SUCCESS,
                                [],
                            ))
                            .await;
                        Ok(1)
                    } else {
                        // TODO: send feedback as error without Err
                        source
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

const fn time_argument() -> TimeArgumentType {
    TimeArgumentType::new(1)
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Three),
    ));

    dispatcher.register(
        command("tick", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("query").executes(TickExecutor(SubCommand::Query)))
            .then(
                literal("rate").then(
                    argument("rate", FloatArgumentType::new(1.0, 10000.0))
                        .executes(TickExecutor(SubCommand::Rate)),
                ),
            )
            .then(literal("freeze").executes(TickExecutor(SubCommand::Freeze(true))))
            .then(literal("unfreeze").executes(TickExecutor(SubCommand::Freeze(false))))
            .then(
                literal("step")
                    .then(literal("stop").executes(TickExecutor(SubCommand::StepStop)))
                    .then(
                        argument("time", time_argument())
                            .executes(TickExecutor(SubCommand::StepTimed)),
                    )
                    .executes(TickExecutor(SubCommand::StepDefault)),
            )
            .then(
                literal("sprint")
                    .then(literal("stop").executes(TickExecutor(SubCommand::SprintStop)))
                    .then(
                        argument("time", time_argument())
                            .executes(TickExecutor(SubCommand::SprintTimed)),
                    ),
            ),
    );
}
