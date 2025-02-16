use async_trait::async_trait;
use pumpkin_util::text::TextComponent;

use crate::command::{
    args::{time::TimeArgumentConsumer, ConsumedArgs, FindArg},
    tree::builder::{argument, literal},
    tree::CommandTree,
    CommandError, CommandExecutor, CommandSender,
};

const NAMES: [&str; 1] = ["weather"];
const DESCRIPTION: &str = "Changes the weather.";
const ARG_DURATION: &str = "duration";

struct WeatherExecutor {
    mode: WeatherMode,
}

enum WeatherMode {
    Clear,
    Rain,
    Thunder,
}

#[async_trait]
impl CommandExecutor for WeatherExecutor {
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
                    argument(ARG_DURATION, TimeArgumentConsumer).execute(WeatherExecutor {
                        mode: WeatherMode::Clear,
                    }),
                )
                .execute(WeatherExecutor {
                    mode: WeatherMode::Clear,
                }),
        )
        .then(
            literal("rain")
                .then(
                    argument(ARG_DURATION, TimeArgumentConsumer).execute(WeatherExecutor {
                        mode: WeatherMode::Rain,
                    }),
                )
                .execute(WeatherExecutor {
                    mode: WeatherMode::Rain,
                }),
        )
        .then(
            literal("thunder")
                .then(
                    argument(ARG_DURATION, TimeArgumentConsumer).execute(WeatherExecutor {
                        mode: WeatherMode::Thunder,
                    }),
                )
                .execute(WeatherExecutor {
                    mode: WeatherMode::Thunder,
                }),
        )
}
