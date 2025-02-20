use async_trait::async_trait;
use pumpkin_util::text::color::{Color, NamedColor};
use pumpkin_util::text::TextComponent;

use crate::command::args::{time::TimeArgumentConsumer, FindArg};
use crate::command::tree::builder::{argument, literal};
use crate::command::{
    tree::CommandTree, CommandError, CommandExecutor, CommandSender, ConsumedArgs,
};

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
    fn to_ticks(self) -> i32 {
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

struct TimeQueryExecutor(QueryMode);

#[async_trait]
impl CommandExecutor for TimeQueryExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        _args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let mode = self.0;
        // TODO: Maybe ask player for world, or get the current world
        let worlds = server.worlds.read().await;
        let world = worlds
            .first()
            .expect("There should always be at least one world");
        let level_time = world.level_time.lock().await;

        let msg = match mode {
            QueryMode::DayTime => {
                let curr_time = level_time.query_daytime();
                TextComponent::translate(
                    "commands.time.query",
                    [TextComponent::text(curr_time.to_string())],
                )
            }
            QueryMode::GameTime => {
                let curr_time = level_time.query_gametime();
                TextComponent::translate(
                    "commands.time.query",
                    [TextComponent::text(curr_time.to_string())],
                )
            }
            QueryMode::Day => {
                let curr_time = level_time.query_day();
                TextComponent::translate(
                    "commands.time.query",
                    [TextComponent::text(curr_time.to_string())],
                )
            }
        };

        sender.send_message(msg).await;
        Ok(())
    }
}

struct TimeChangeExecutor(Mode);

#[async_trait]
impl CommandExecutor for TimeChangeExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let time_count = if let Mode::Set(Some(preset)) = &self.0 {
            preset.to_ticks()
        } else if let Ok(ticks) = TimeArgumentConsumer::find_arg(args, ARG_TIME) {
            ticks
        } else {
            sender
                .send_message(
                    TextComponent::text("Invalid time specified.")
                        .color(Color::Named(NamedColor::Red)),
                )
                .await;
            return Ok(());
        };

        let mode = self.0;
        // TODO: Maybe ask player for world, or get the current world
        let worlds = server.worlds.read().await;
        let world = worlds
            .first()
            .expect("There should always be at least one world");
        let mut level_time = world.level_time.lock().await;

        let msg = match mode {
            Mode::Add => {
                // add
                level_time.add_time(time_count.into());
                level_time.send_time(world).await;
                let curr_time = level_time.query_daytime();
                TextComponent::translate(
                    "commands.time.add",
                    [TextComponent::text(curr_time.to_string())],
                )
            }
            Mode::Set(_) => {
                // set
                level_time.set_time(time_count.into());
                level_time.send_time(world).await;
                TextComponent::translate(
                    "commands.time.set",
                    [TextComponent::text(time_count.to_string())],
                )
            }
        };

        sender.send_message(msg).await;
        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("add").then(
                argument(ARG_TIME, TimeArgumentConsumer).execute(TimeChangeExecutor(Mode::Add)),
            ),
        )
        .then(
            literal("query")
                .then(literal("daytime").execute(TimeQueryExecutor(QueryMode::DayTime)))
                .then(literal("gametime").execute(TimeQueryExecutor(QueryMode::GameTime)))
                .then(literal("day").execute(TimeQueryExecutor(QueryMode::Day))),
        )
        .then(
            literal("set")
                .then(literal("day").execute(TimeChangeExecutor(Mode::Set(Some(PresetTime::Day)))))
                .then(
                    literal("noon").execute(TimeChangeExecutor(Mode::Set(Some(PresetTime::Noon)))),
                )
                .then(
                    literal("night")
                        .execute(TimeChangeExecutor(Mode::Set(Some(PresetTime::Night)))),
                )
                .then(
                    literal("midnight")
                        .execute(TimeChangeExecutor(Mode::Set(Some(PresetTime::Midnight)))),
                )
                .then(
                    argument(ARG_TIME, TimeArgumentConsumer)
                        .execute(TimeChangeExecutor(Mode::Set(None))),
                ),
        )
}
