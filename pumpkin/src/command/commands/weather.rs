use async_trait::async_trait;
use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandError, CommandExecutor, CommandSender,
    args::{ConsumedArgs, FindArg, time::TimeArgumentConsumer},
    tree::CommandTree,
    tree::builder::{argument, literal},
};

const NAMES: [&str; 1] = ["weather"];
const DESCRIPTION: &str = "Changes the weather.";
const ARG_DURATION: &str = "duration";

struct Executor {
    mode: WeatherMode,
}

enum WeatherMode {
    Clear,
    Rain,
    Thunder,
}

#[async_trait]
impl CommandExecutor for Executor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let world = sender
            .world()
            .await
            .ok_or(CommandError::InvalidRequirement)?;
        let duration = TimeArgumentConsumer::find_arg(args, ARG_DURATION).unwrap_or(6000);
        let mut weather = world.weather.lock().await;

        match self.mode {
            WeatherMode::Clear => {
                weather
                    .set_weather_parameters(&world, duration, 0, false, false)
                    .await;
                sender
                    .send_message(TextComponent::translate("commands.weather.set.clear", []))
                    .await;
            }
            WeatherMode::Rain => {
                weather
                    .set_weather_parameters(&world, 0, duration, true, false)
                    .await;
                sender
                    .send_message(TextComponent::translate("commands.weather.set.rain", []))
                    .await;
            }
            WeatherMode::Thunder => {
                weather
                    .set_weather_parameters(&world, 0, duration, true, true)
                    .await;
                sender
                    .send_message(TextComponent::translate("commands.weather.set.thunder", []))
                    .await;
            }
        }

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("clear")
                .then(
                    argument(ARG_DURATION, TimeArgumentConsumer).execute(Executor {
                        mode: WeatherMode::Clear,
                    }),
                )
                .execute(Executor {
                    mode: WeatherMode::Clear,
                }),
        )
        .then(
            literal("rain")
                .then(
                    argument(ARG_DURATION, TimeArgumentConsumer).execute(Executor {
                        mode: WeatherMode::Rain,
                    }),
                )
                .execute(Executor {
                    mode: WeatherMode::Rain,
                }),
        )
        .then(
            literal("thunder")
                .then(
                    argument(ARG_DURATION, TimeArgumentConsumer).execute(Executor {
                        mode: WeatherMode::Thunder,
                    }),
                )
                .execute(Executor {
                    mode: WeatherMode::Thunder,
                }),
        )
}
