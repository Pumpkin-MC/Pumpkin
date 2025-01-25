use async_trait::async_trait;
use pumpkin_util::text::color::{Color, NamedColor};
use pumpkin_util::text::TextComponent;

use crate::command::args::arg_bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::FindArgDefaultName;
use crate::command::tree_builder::{argument_default_name, literal};
use crate::command::{
    tree::CommandTree, CommandError, CommandExecutor, CommandSender, ConsumedArgs,
};

const NAMES: [&str; 1] = ["time"];

const DESCRIPTION: &str = "Query the world time.";

// TODO: This should be either higher or not bounded
fn arg_number() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new()
        .name("time")
        .min(0)
        .max(24000)
}

#[derive(Clone, Copy)]
enum PresetTime {
    Day,
    Noon,
    Night,
    Midnight,
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
                    [TextComponent::text(curr_time.to_string())].into(),
                )
            }
            QueryMode::GameTime => {
                let curr_time = level_time.query_gametime();
                TextComponent::translate(
                    "commands.time.query",
                    [TextComponent::text(curr_time.to_string())].into(),
                )
            }
            QueryMode::Day => {
                let curr_time = level_time.query_day();
                TextComponent::translate(
                    "commands.time.query",
                    [TextComponent::text(curr_time.to_string())].into(),
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
            match preset {
                PresetTime::Day => 1000,
                PresetTime::Noon => 6000,
                PresetTime::Night => 13000,
                PresetTime::Midnight => 18000,
            }
        } else {
            match arg_number().find_arg_default_name(args) {
                Err(_) => 1,
                Ok(Ok(count)) => count,
                Ok(Err(())) => {
                    sender
                        .send_message(
                            TextComponent::text("Time is too large or too small.")
                                .color(Color::Named(NamedColor::Red)),
                        )
                        .await;
                    return Ok(());
                }
            }
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
                    [TextComponent::text(curr_time.to_string())].into(),
                )
            }
            Mode::Set(_) => {
                // set
                level_time.set_time(time_count.into());
                level_time.send_time(world).await;
                TextComponent::translate(
                    "commands.time.set",
                    [TextComponent::text(time_count.to_string())].into(),
                )
            }
        };

        sender.send_message(msg).await;
        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .with_child(
            literal("add").with_child(
                argument_default_name(arg_number()).execute(TimeChangeExecutor(Mode::Add)),
            ),
        )
        .with_child(
            literal("query")
                .with_child(literal("daytime").execute(TimeQueryExecutor(QueryMode::DayTime)))
                .with_child(literal("gametime").execute(TimeQueryExecutor(QueryMode::GameTime)))
                .with_child(literal("day").execute(TimeQueryExecutor(QueryMode::Day))),
        )
        .with_child(
            literal("set")
                .with_child(
                    literal("day").execute(TimeChangeExecutor(Mode::Set(Some(PresetTime::Day)))),
                )
                .with_child(
                    literal("noon").execute(TimeChangeExecutor(Mode::Set(Some(PresetTime::Noon)))),
                )
                .with_child(
                    literal("night")
                        .execute(TimeChangeExecutor(Mode::Set(Some(PresetTime::Night)))),
                )
                .with_child(
                    literal("midnight")
                        .execute(TimeChangeExecutor(Mode::Set(Some(PresetTime::Midnight)))),
                )
                .with_child(
                    argument_default_name(arg_number())
                        .execute(TimeChangeExecutor(Mode::Set(None))),
                ),
        )
}
