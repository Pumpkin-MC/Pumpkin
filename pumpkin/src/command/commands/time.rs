use pumpkin_util::text::TextComponent;

use crate::command::CommandResult;
use crate::command::args::{FindArg, time::TimeArgumentConsumer};
use crate::command::dispatcher::CommandError;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandSender, ConsumedArgs, tree::CommandTree};

const NAMES: [&str; 1] = ["time"];
const DESCRIPTION: &str = "Query the world time.";
const ARG_TIME: &str = "time";

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
enum Mode {
    Add,
    Set(Option<PresetTime>),
}

#[derive(Clone, Copy)]
enum QueryMode {
    DayTime,
    GameTime,
    Day,
}

struct QueryExecutor(QueryMode);

impl CommandExecutor for QueryExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let mode = self.0;
            // TODO: Maybe ask player for world, or get the current world
            let worlds = server.worlds.load();
            let world = worlds
                .first()
                .expect("There should always be at least one world");
            let level_time = world.level_time.lock().await;

            let curr_time = match mode {
                QueryMode::DayTime => level_time.query_daytime(),
                QueryMode::GameTime => level_time.query_gametime(),
                QueryMode::Day => level_time.query_day(),
            };
            sender
                .send_message(TextComponent::translate(
                    "commands.time.query",
                    [TextComponent::text(curr_time.to_string())],
                ))
                .await;
            Ok(curr_time as i32)
        })
    }
}

struct ChangeExecutor(Mode);

impl CommandExecutor for ChangeExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let time_count = if let Mode::Set(Some(preset)) = &self.0 {
                preset.to_ticks()
            } else if let Ok(ticks) = TimeArgumentConsumer::find_arg(args, ARG_TIME) {
                ticks
            } else {
                return Err(CommandError::CommandFailed(TextComponent::text(
                    "Invalid time specified.",
                )));
            };

            let mode = self.0;
            // TODO: Maybe ask player for world, or get the current world
            let worlds = server.worlds.load();
            let world = worlds
                .first()
                .expect("There should always be at least one world");
            let mut level_time = world.level_time.lock().await;

            match mode {
                Mode::Add => {
                    // add
                    level_time.add_time(time_count.into());
                    level_time.send_time(world).await;
                    let curr_time = level_time.query_daytime();
                    sender
                        .send_message(TextComponent::translate(
                            "commands.time.set",
                            [TextComponent::text(curr_time.to_string())],
                        ))
                        .await;
                    Ok(curr_time as i32)
                }
                Mode::Set(_) => {
                    // set
                    level_time.set_time(time_count.into());
                    level_time.send_time(world).await;
                    sender
                        .send_message(TextComponent::translate(
                            "commands.time.set",
                            [TextComponent::text(time_count.to_string())],
                        ))
                        .await;
                    Ok(time_count)
                }
            }
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("add")
                .then(argument(ARG_TIME, TimeArgumentConsumer).execute(ChangeExecutor(Mode::Add))),
        )
        .then(
            literal("query")
                .then(literal("daytime").execute(QueryExecutor(QueryMode::DayTime)))
                .then(literal("gametime").execute(QueryExecutor(QueryMode::GameTime)))
                .then(literal("day").execute(QueryExecutor(QueryMode::Day))),
        )
        .then(
            literal("set")
                .then(literal("day").execute(ChangeExecutor(Mode::Set(Some(PresetTime::Day)))))
                .then(literal("noon").execute(ChangeExecutor(Mode::Set(Some(PresetTime::Noon)))))
                .then(literal("night").execute(ChangeExecutor(Mode::Set(Some(PresetTime::Night)))))
                .then(
                    literal("midnight")
                        .execute(ChangeExecutor(Mode::Set(Some(PresetTime::Midnight)))),
                )
                .then(
                    argument(ARG_TIME, TimeArgumentConsumer)
                        .execute(ChangeExecutor(Mode::Set(None))),
                ),
        )
}
