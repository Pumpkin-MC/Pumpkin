use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::args::{
    FindArg, bounded_num::BoundedNumArgumentConsumer,
    resource_location::ResourceLocationArgumentConsumer, time::TimeArgumentConsumer,
};
use crate::command::tree::builder::{argument, literal};
use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender, ConsumedArgs, tree::CommandTree,
};

const NAMES: [&str; 1] = ["time"];
const DESCRIPTION: &str = "Query or modify the world time and clocks.";
const ARG_TIME: &str = "time";
const ARG_RATE: &str = "rate";
const ARG_CLOCK: &str = "clock";

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

struct QueryExecutor(QueryMode);

impl CommandExecutor for QueryExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let clock_name = ResourceLocationArgumentConsumer::find_arg(args, ARG_CLOCK)
                .map_or_else(|_| DEFAULT_CLOCK.to_string(), ToString::to_string);
            let mode = self.0;
            let worlds = server.worlds.load();
            let world = worlds.first().ok_or(CommandError::InvalidRequirement)?;
            let level_time = world.level_time.lock().await;

            match mode {
                QueryMode::GameTime => {
                    let game_time = level_time.query_gametime();
                    sender
                        .send_message(pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_TIME_QUERY_GAMETIME,
                            translation::bedrock::COMMANDS_TIME_QUERY_GAMETIME,
                            TextComponent::text(game_time.to_string())
                        ))
                        .await;
                    Ok(wrap_time(game_time))
                }
                QueryMode::Time => {
                    let total_ticks = level_time.time_of_day;
                    sender
                        .send_message(pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_TIME_QUERY_ABSOLUTE,
                            translation::bedrock::COMMANDS_TIME_QUERY_DAYTIME,
                            TextComponent::text(clock_name.clone()),
                            TextComponent::text(total_ticks.to_string())
                        ))
                        .await;
                    Ok(wrap_time(total_ticks))
                }
                QueryMode::DayTime => {
                    let curr_time = level_time.query_daytime();
                    sender
                        .send_message(pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_TIME_QUERY,
                            translation::bedrock::COMMANDS_TIME_QUERY_DAYTIME,
                            TextComponent::text(curr_time.to_string())
                        ))
                        .await;
                    Ok(wrap_time(curr_time))
                }
                QueryMode::Day => {
                    let curr_time = level_time.query_day();
                    sender
                        .send_message(pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_TIME_QUERY,
                            translation::bedrock::COMMANDS_TIME_QUERY_DAY,
                            TextComponent::text(curr_time.to_string())
                        ))
                        .await;
                    Ok(curr_time as i32)
                }
            }
        })
    }
}

struct ActionExecutor(Action);

impl CommandExecutor for ActionExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let clock_name = ResourceLocationArgumentConsumer::find_arg(args, ARG_CLOCK)
                .map_or_else(|_| DEFAULT_CLOCK.to_string(), ToString::to_string);
            let action = self.0;
            let worlds = server.worlds.load();
            let world = worlds.first().ok_or(CommandError::InvalidRequirement)?;
            let mut level_time = world.level_time.lock().await;

            match action {
                Action::Set(preset) => {
                    let time_count = if let Some(p) = preset {
                        p.to_ticks()
                    } else {
                        TimeArgumentConsumer::find_arg(args, ARG_TIME)?
                    };
                    level_time.set_time(time_count.into());
                    level_time.send_time(world).await;
                    sender
                        .send_message(pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_TIME_SET_ABSOLUTE,
                            translation::bedrock::COMMANDS_TIME_SET,
                            TextComponent::text(clock_name.clone()),
                            TextComponent::text(time_count.to_string())
                        ))
                        .await;
                    Ok(time_count)
                }
                Action::Add => {
                    let time_count = TimeArgumentConsumer::find_arg(args, ARG_TIME)?;
                    level_time.add_time(time_count.into());
                    level_time.send_time(world).await;
                    let total_ticks = level_time.time_of_day;
                    sender
                        .send_message(pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_TIME_SET_ABSOLUTE,
                            translation::bedrock::COMMANDS_TIME_ADDED,
                            TextComponent::text(clock_name.clone()),
                            TextComponent::text(total_ticks.to_string())
                        ))
                        .await;
                    Ok(wrap_time(total_ticks))
                }
                Action::Pause => {
                    level_time.set_paused(true);
                    level_time.send_time(world).await;
                    sender
                        .send_message(pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_TIME_PAUSE,
                            translation::bedrock::COMMANDS_TIME_STOP,
                            TextComponent::text(clock_name.clone())
                        ))
                        .await;
                    Ok(1)
                }
                Action::Resume => {
                    level_time.set_paused(false);
                    level_time.send_time(world).await;
                    sender
                        .send_message(pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_TIME_RESUME,
                            translation::bedrock::COMMANDS_TIME_SET,
                            TextComponent::text(clock_name.clone())
                        ))
                        .await;
                    Ok(1)
                }
                Action::Rate => {
                    let rate_res = BoundedNumArgumentConsumer::<f32>::find_arg(args, ARG_RATE)?;
                    let rate = match rate_res {
                        Ok(val) => val,
                        Err(err) => return Err(err.into()),
                    };
                    level_time.set_rate(rate);
                    level_time.send_time(world).await;
                    sender
                        .send_message(pumpkin_macros::translate_cross!(
                            translation::java::COMMANDS_TIME_RATE,
                            translation::bedrock::COMMANDS_TIME_SET,
                            TextComponent::text(clock_name.clone()),
                            TextComponent::text(rate.to_string())
                        ))
                        .await;
                    Ok(1)
                }
            }
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    let set_node = literal("set")
        .then(literal("day").execute(ActionExecutor(Action::Set(Some(PresetTime::Day)))))
        .then(literal("noon").execute(ActionExecutor(Action::Set(Some(PresetTime::Noon)))))
        .then(literal("night").execute(ActionExecutor(Action::Set(Some(PresetTime::Night)))))
        .then(literal("midnight").execute(ActionExecutor(Action::Set(Some(PresetTime::Midnight)))))
        .then(
            argument(ARG_TIME, TimeArgumentConsumer::new())
                .execute(ActionExecutor(Action::Set(None))),
        );

    let add_node = literal("add").then(
        argument(ARG_TIME, TimeArgumentConsumer::min(i32::MIN))
            .execute(ActionExecutor(Action::Add)),
    );

    let pause_node = literal("pause").execute(ActionExecutor(Action::Pause));
    let resume_node = literal("resume").execute(ActionExecutor(Action::Resume));

    let rate_node = literal("rate").then(
        argument(
            ARG_RATE,
            BoundedNumArgumentConsumer::<f32>::new()
                .min(1.0e-5)
                .max(1000.0),
        )
        .execute(ActionExecutor(Action::Rate)),
    );

    let query_node = literal("query")
        .then(literal("time").execute(QueryExecutor(QueryMode::Time)))
        .then(literal("gametime").execute(QueryExecutor(QueryMode::GameTime)))
        .then(literal("daytime").execute(QueryExecutor(QueryMode::DayTime)))
        .then(literal("day").execute(QueryExecutor(QueryMode::Day)));

    let of_clock_node = literal("of").then(
        argument(ARG_CLOCK, ResourceLocationArgumentConsumer)
            .then(
                literal("set")
                    .then(
                        literal("day").execute(ActionExecutor(Action::Set(Some(PresetTime::Day)))),
                    )
                    .then(
                        literal("noon")
                            .execute(ActionExecutor(Action::Set(Some(PresetTime::Noon)))),
                    )
                    .then(
                        literal("night")
                            .execute(ActionExecutor(Action::Set(Some(PresetTime::Night)))),
                    )
                    .then(
                        literal("midnight")
                            .execute(ActionExecutor(Action::Set(Some(PresetTime::Midnight)))),
                    )
                    .then(
                        argument(ARG_TIME, TimeArgumentConsumer::new())
                            .execute(ActionExecutor(Action::Set(None))),
                    ),
            )
            .then(
                literal("add").then(
                    argument(ARG_TIME, TimeArgumentConsumer::min(i32::MIN))
                        .execute(ActionExecutor(Action::Add)),
                ),
            )
            .then(literal("pause").execute(ActionExecutor(Action::Pause)))
            .then(literal("resume").execute(ActionExecutor(Action::Resume)))
            .then(
                literal("rate").then(
                    argument(
                        ARG_RATE,
                        BoundedNumArgumentConsumer::<f32>::new()
                            .min(1.0e-5)
                            .max(1000.0),
                    )
                    .execute(ActionExecutor(Action::Rate)),
                ),
            )
            .then(
                literal("query")
                    .then(literal("time").execute(QueryExecutor(QueryMode::Time)))
                    .then(literal("gametime").execute(QueryExecutor(QueryMode::GameTime)))
                    .then(literal("daytime").execute(QueryExecutor(QueryMode::DayTime)))
                    .then(literal("day").execute(QueryExecutor(QueryMode::Day))),
            ),
    );

    CommandTree::new(NAMES, DESCRIPTION)
        .then(set_node)
        .then(add_node)
        .then(pause_node)
        .then(resume_node)
        .then(rate_node)
        .then(query_node)
        .then(of_clock_node)
}
