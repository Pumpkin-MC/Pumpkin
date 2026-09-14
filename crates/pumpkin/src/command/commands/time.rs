use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::float::FloatArgumentType;
use crate::command::argument_types::identifier::IdentifierArgumentType;
use crate::command::argument_types::time::TimeArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Query or modify the world time and clocks.";
const PERMISSION: &str = "minecraft:command.time";

const DEFAULT_CLOCK: &str = "minecraft:overworld";

#[derive(Clone, Copy)]
enum PresetTime {
    Day,
    Noon,
    Night,
    Midnight,
}

impl PresetTime {
    const fn to_ticks(self) -> i32 {
        match self {
            Self::Day => 1000,
            Self::Noon => 6000,
            Self::Night => 13000,
            Self::Midnight => 18000,
        }
    }
}

#[derive(Clone, Copy)]
enum Action {
    Set(Option<PresetTime>),
    Add,
    Pause,
    Resume,
    Rate,
}

#[derive(Clone, Copy)]
enum QueryMode {
    Time,
    GameTime,
    DayTime,
    Day,
}

const fn wrap_time(ticks: i64) -> i32 {
    (ticks % 2_147_483_647) as i32
}

struct QueryExecutor {
    mode: QueryMode,
    has_clock: bool,
}

impl CommandExecutor for QueryExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let clock_name = if self.has_clock {
            IdentifierArgumentType::get(context, "clock")
                .map_or_else(|_| DEFAULT_CLOCK.to_string(), |id| id.to_string())
        } else {
            DEFAULT_CLOCK.to_string()
        };

        let world = context.source.world();
        let level_time = world
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();

        match self.mode {
            QueryMode::GameTime => {
                let game_time = level_time.query_gametime();
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_TIME_QUERY_GAMETIME,
                        translation::bedrock::COMMANDS_TIME_QUERY_GAMETIME,
                        TextComponent::text(game_time.to_string())
                    ),
                    false,
                );
                Ok(wrap_time(game_time))
            }
            QueryMode::Time => {
                let total_ticks = level_time.time_of_day;
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_TIME_QUERY_ABSOLUTE,
                        translation::bedrock::COMMANDS_TIME_QUERY_DAYTIME,
                        TextComponent::text(clock_name),
                        TextComponent::text(total_ticks.to_string())
                    ),
                    false,
                );
                Ok(wrap_time(total_ticks))
            }
            QueryMode::DayTime => {
                let curr_time = level_time.query_daytime();
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_TIME_QUERY,
                        translation::bedrock::COMMANDS_TIME_QUERY_DAYTIME,
                        TextComponent::text(curr_time.to_string())
                    ),
                    false,
                );
                Ok(wrap_time(curr_time))
            }
            QueryMode::Day => {
                let curr_time = level_time.query_day();
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_TIME_QUERY,
                        translation::bedrock::COMMANDS_TIME_QUERY_DAY,
                        TextComponent::text(curr_time.to_string())
                    ),
                    false,
                );
                Ok(curr_time as i32)
            }
        }
    }
}

struct ActionExecutor {
    action: Action,
    has_clock: bool,
}

impl CommandExecutor for ActionExecutor {
    #[allow(clippy::too_many_lines)]
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let clock_name = if self.has_clock {
            IdentifierArgumentType::get(context, "clock")
                .map_or_else(|_| DEFAULT_CLOCK.to_string(), |id| id.to_string())
        } else {
            DEFAULT_CLOCK.to_string()
        };

        let world = context.source.world();

        match self.action {
            Action::Set(preset) => {
                let time_count = if let Some(p) = preset {
                    p.to_ticks()
                } else {
                    TimeArgumentType::get(context, "time")?
                };
                let level_time = {
                    let mut guard = world
                        .level_time
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    guard.set_time(time_count.into());
                    guard.clone()
                };
                level_time.send_time(world);
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_TIME_SET_ABSOLUTE,
                        translation::bedrock::COMMANDS_TIME_SET,
                        TextComponent::text(clock_name),
                        TextComponent::text(time_count.to_string())
                    ),
                    true,
                );
                Ok(time_count)
            }
            Action::Add => {
                let time_count = TimeArgumentType::get(context, "time")?;
                let (level_time, total_ticks) = {
                    let mut guard = world
                        .level_time
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    guard.add_time(time_count.into());
                    let total_ticks = guard.time_of_day;
                    (guard.clone(), total_ticks)
                };
                level_time.send_time(world);
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_TIME_SET_ABSOLUTE,
                        translation::bedrock::COMMANDS_TIME_ADDED,
                        TextComponent::text(clock_name),
                        TextComponent::text(total_ticks.to_string())
                    ),
                    true,
                );
                Ok(wrap_time(total_ticks))
            }
            Action::Pause => {
                let level_time = {
                    let mut guard = world
                        .level_time
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    guard.set_paused(true);
                    guard.clone()
                };
                level_time.send_time(world);
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_TIME_PAUSE,
                        translation::bedrock::COMMANDS_TIME_STOP,
                        TextComponent::text(clock_name)
                    ),
                    true,
                );
                Ok(1)
            }
            Action::Resume => {
                let level_time = {
                    let mut guard = world
                        .level_time
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    guard.set_paused(false);
                    guard.clone()
                };
                level_time.send_time(world);
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_TIME_RESUME,
                        translation::bedrock::COMMANDS_TIME_SET,
                        TextComponent::text(clock_name)
                    ),
                    true,
                );
                Ok(1)
            }
            Action::Rate => {
                let rate = FloatArgumentType::get(context, "rate")?;
                let level_time = {
                    let mut guard = world
                        .level_time
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    guard.set_rate(rate);
                    guard.clone()
                };
                level_time.send_time(world);
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        translation::java::COMMANDS_TIME_RATE,
                        translation::bedrock::COMMANDS_TIME_SET,
                        TextComponent::text(clock_name),
                        TextComponent::text(rate.to_string())
                    ),
                    true,
                );
                Ok(1)
            }
        }
    }
}

macro_rules! build_branches {
    ($builder:expr, $has_clock:expr) => {{
        let set_node = literal("set")
            .then(literal("day").executes(ActionExecutor {
                action: Action::Set(Some(PresetTime::Day)),
                has_clock: $has_clock,
            }))
            .then(literal("noon").executes(ActionExecutor {
                action: Action::Set(Some(PresetTime::Noon)),
                has_clock: $has_clock,
            }))
            .then(literal("night").executes(ActionExecutor {
                action: Action::Set(Some(PresetTime::Night)),
                has_clock: $has_clock,
            }))
            .then(literal("midnight").executes(ActionExecutor {
                action: Action::Set(Some(PresetTime::Midnight)),
                has_clock: $has_clock,
            }))
            .then(
                argument("time", TimeArgumentType::any()).executes(ActionExecutor {
                    action: Action::Set(None),
                    has_clock: $has_clock,
                }),
            );

        let add_node = literal("add").then(
            argument("time", TimeArgumentType::new(i32::MIN)).executes(ActionExecutor {
                action: Action::Add,
                has_clock: $has_clock,
            }),
        );

        let pause_node = literal("pause").executes(ActionExecutor {
            action: Action::Pause,
            has_clock: $has_clock,
        });
        let resume_node = literal("resume").executes(ActionExecutor {
            action: Action::Resume,
            has_clock: $has_clock,
        });

        let rate_node = literal("rate").then(
            argument("rate", FloatArgumentType::new(1.0e-5, 1000.0)).executes(ActionExecutor {
                action: Action::Rate,
                has_clock: $has_clock,
            }),
        );

        let query_node = literal("query")
            .then(literal("time").executes(QueryExecutor {
                mode: QueryMode::Time,
                has_clock: $has_clock,
            }))
            .then(literal("gametime").executes(QueryExecutor {
                mode: QueryMode::GameTime,
                has_clock: $has_clock,
            }))
            .then(literal("daytime").executes(QueryExecutor {
                mode: QueryMode::DayTime,
                has_clock: $has_clock,
            }))
            .then(literal("day").executes(QueryExecutor {
                mode: QueryMode::Day,
                has_clock: $has_clock,
            }));

        $builder
            .then(set_node)
            .then(add_node)
            .then(pause_node)
            .then(resume_node)
            .then(rate_node)
            .then(query_node)
    }};
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let of_clock_node = literal("of").then(build_branches!(
        argument("clock", IdentifierArgumentType),
        true
    ));

    let time_cmd = build_branches!(command("time", DESCRIPTION).requires(PERMISSION), false)
        .then(of_clock_node);

    dispatcher.register(time_cmd);
}
