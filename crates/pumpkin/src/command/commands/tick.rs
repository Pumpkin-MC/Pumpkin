use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::float::FloatArgumentType;
use crate::command::argument_types::time::TimeArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;
use crate::server::Server;
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::{
    TextComponent,
    color::{Color, NamedColor},
};

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
    fn handle_query(
        source: &CommandSource,
        manager: &crate::server::tick_rate_manager::ServerTickRateManager,
    ) -> i32 {
        let tick_rate = manager.tickrate();
        let avg_tick_nanos = source.server().get_average_tick_time_nanos();
        let avg_mspt_str = nanos_to_millis_string(avg_tick_nanos);

        if manager.is_sprinting() {
            source.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_TICK_STATUS_SPRINTING,
                    translation::java::COMMANDS_TICK_STATUS_SPRINTING,
                    [],
                ),
                false,
            );
            source.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_TICK_QUERY_RATE_SPRINTING,
                    translation::java::COMMANDS_TICK_QUERY_RATE_SPRINTING,
                    [
                        TextComponent::text(format!("{tick_rate:.1}")),
                        TextComponent::text(avg_mspt_str),
                    ],
                ),
                false,
            );
        } else {
            Self::handle_non_sprinting_status(source, manager, avg_tick_nanos);

            let target_mspt_str = nanos_to_millis_string(manager.nanoseconds_per_tick());
            source.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_TICK_QUERY_RATE_RUNNING,
                    translation::java::COMMANDS_TICK_QUERY_RATE_RUNNING,
                    [
                        TextComponent::text(format!("{tick_rate:.1}")),
                        TextComponent::text(avg_mspt_str),
                        TextComponent::text(target_mspt_str),
                    ],
                ),
                false,
            );
        }

        Self::send_percentiles(source, source.server());
        tick_rate as i32
    }

    fn handle_non_sprinting_status(
        sender: &CommandSource,
        manager: &crate::server::tick_rate_manager::ServerTickRateManager,
        avg_tick_nanos: i64,
    ) {
        if manager.is_frozen() {
            sender.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_TICK_STATUS_FROZEN,
                    translation::java::COMMANDS_TICK_STATUS_FROZEN,
                    [],
                ),
                false,
            );
        } else if avg_tick_nanos > manager.nanoseconds_per_tick() {
            sender.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_TICK_STATUS_LAGGING,
                    translation::java::COMMANDS_TICK_STATUS_LAGGING,
                    [],
                ),
                false,
            );
        } else {
            sender.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_TICK_STATUS_RUNNING,
                    translation::java::COMMANDS_TICK_STATUS_RUNNING,
                    [],
                ),
                false,
            );
        }
    }

    fn send_percentiles(source: &CommandSource, server: &Server) {
        let mut times = server.get_tick_times_nanos_copy();
        let tick_count =
            (server.tick_count.load(std::sync::atomic::Ordering::Relaxed) as usize).min(100);
        let slice = &mut times[..tick_count];
        slice.sort_unstable();

        let (p50, p95, p99) = if slice.is_empty() {
            (0, 0, 0)
        } else {
            (
                slice[(slice.len() * 50) / 100],
                slice[(slice.len() * 95) / 100],
                slice[(slice.len() * 99) / 100],
            )
        };

        source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_TICK_QUERY_PERCENTILES,
                translation::java::COMMANDS_TICK_QUERY_PERCENTILES,
                [
                    TextComponent::text(nanos_to_millis_string(p50)),
                    TextComponent::text(nanos_to_millis_string(p95)),
                    TextComponent::text(nanos_to_millis_string(p99)),
                    TextComponent::text(slice.len().to_string()),
                ],
            ),
            false,
        );
    }

    fn send_sprint_report(source: &CommandSource, ticks: i32) {
        source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_TICK_SPRINT_REPORT,
                translation::java::COMMANDS_TICK_SPRINT_REPORT,
                [TextComponent::text(ticks.to_string())],
            ),
            true,
        );
        source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_TICK_STATUS_SPRINTING,
                translation::java::COMMANDS_TICK_STATUS_SPRINTING,
                [],
            ),
            true,
        );
    }

    fn handle_set_tick_rate(
        source: &CommandSource,
        manager: &crate::server::tick_rate_manager::ServerTickRateManager,
        rate: f32,
    ) -> i32 {
        manager.set_tick_rate(source.server(), rate);
        source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_TICK_RATE_SUCCESS,
                translation::java::COMMANDS_TICK_RATE_SUCCESS,
                [TextComponent::text(format!("{rate:.1}"))],
            ),
            true,
        );
        rate as i32
    }
}

impl CommandExecutor for TickExecutor {
    #[expect(clippy::too_many_lines)]
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let manager = &context.server().tick_rate_manager;
        let source = context.source.as_ref();
        let server = source.server();
        match self.0 {
            SubCommand::Query => Ok(Self::handle_query(source, manager)),
            SubCommand::Rate => {
                let rate = FloatArgumentType::get(context, "rate")?;
                Ok(Self::handle_set_tick_rate(source, manager, rate))
            }
            SubCommand::Freeze(freeze) => {
                manager.set_frozen(server, freeze);
                let message_key = if freeze {
                    "commands.tick.status.frozen"
                } else {
                    "commands.tick.status.running"
                };
                source.send_feedback(TextComponent::translate(message_key, []), true);
                Ok(freeze as i32)
            }
            SubCommand::StepDefault => {
                if manager.step_game_if_paused(server, 1) {
                    source.send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_TICK_STEP_SUCCESS,
                            translation::java::COMMANDS_TICK_STEP_SUCCESS,
                            [TextComponent::text("1")],
                        ),
                        true,
                    );
                    Ok(1)
                } else {
                    source.send_error(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_TICK_STEP_FAIL,
                            translation::java::COMMANDS_TICK_STEP_FAIL,
                            [],
                        )
                        .color(Color::Named(NamedColor::Red)),
                    );
                    Ok(0)
                }
            }
            SubCommand::StepTimed => {
                let ticks = TimeArgumentType::get(context, "time")?;
                if manager.step_game_if_paused(server, ticks) {
                    source.send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_TICK_STEP_SUCCESS,
                            translation::java::COMMANDS_TICK_STEP_SUCCESS,
                            [TextComponent::text(ticks.to_string())],
                        ),
                        true,
                    );
                    Ok(1)
                } else {
                    source.send_error(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_TICK_STEP_FAIL,
                            translation::java::COMMANDS_TICK_STEP_FAIL,
                            [],
                        )
                        .color(Color::Named(NamedColor::Red)),
                    );
                    Ok(0)
                }
            }
            SubCommand::StepStop => {
                if manager.stop_stepping(server) {
                    source.send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_TICK_STEP_STOP_SUCCESS,
                            translation::java::COMMANDS_TICK_STEP_STOP_SUCCESS,
                            [],
                        ),
                        true,
                    );
                    Ok(1)
                } else {
                    source.send_error(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_TICK_STEP_STOP_FAIL,
                            translation::java::COMMANDS_TICK_STEP_STOP_FAIL,
                            [],
                        )
                        .color(Color::Named(NamedColor::Red)),
                    );
                    Ok(0)
                }
            }
            SubCommand::SprintTimed => {
                let ticks = TimeArgumentType::get(context, "time")?;
                manager.request_game_to_sprint(server, ticks as i64);
                Self::send_sprint_report(source, ticks);
                Ok(1)
            }
            SubCommand::SprintStop => {
                if manager.stop_sprinting(server) {
                    source.send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_TICK_SPRINT_STOP_SUCCESS,
                            translation::java::COMMANDS_TICK_SPRINT_STOP_SUCCESS,
                            [],
                        ),
                        true,
                    );
                    Ok(1)
                } else {
                    source.send_error(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_TICK_SPRINT_STOP_FAIL,
                            translation::java::COMMANDS_TICK_SPRINT_STOP_FAIL,
                            [],
                        )
                        .color(Color::Named(NamedColor::Red)),
                    );
                    Ok(0)
                }
            }
        }
    }
}

struct TickSuggestionProvider(&'static [&'static str]);

impl SuggestionProvider for TickSuggestionProvider {
    fn suggest(
        &self,
        _context: &CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult {
        for suggestion in self.0 {
            builder = builder.suggest(*suggestion);
        }
        builder.build()
    }
}

const fn time_argument() -> TimeArgumentType {
    TimeArgumentType::new(1)
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
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
                        .suggests(TickSuggestionProvider(&["20"]))
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
                            .suggests(TickSuggestionProvider(&["1t", "1s"]))
                            .executes(TickExecutor(SubCommand::StepTimed)),
                    )
                    .executes(TickExecutor(SubCommand::StepDefault)),
            )
            .then(
                literal("sprint")
                    .then(literal("stop").executes(TickExecutor(SubCommand::SprintStop)))
                    .then(
                        argument("time", time_argument())
                            .suggests(TickSuggestionProvider(&["60s", "1d", "3d"]))
                            .executes(TickExecutor(SubCommand::SprintTimed)),
                    ),
            ),
    );
}
