use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender,
    args::{ConsumedArgs, FindArg, time::TimeArgumentConsumer},
    tree::{
        CommandTree,
        builder::{argument, literal},
    },
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

impl CommandExecutor for Executor {
    fn execute(
        &self,
        sender: &CommandSender,
        server: &crate::server::Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let duration = TimeArgumentConsumer::find_arg(args, ARG_DURATION).ok();
        let world = {
            let guard = server.worlds.load();

            guard
                .first()
                .cloned()
                .ok_or(CommandError::InvalidRequirement)?
        };
        let message = {
            let mut weather = world
                .weather
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            match self.mode {
                WeatherMode::Clear => {
                    let processed_duration =
                        duration.unwrap_or_else(|| rand::random_range(12_000..=180_000));

                    weather.set_weather_parameters(&world, processed_duration, 0, false, false);
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_WEATHER_SET_CLEAR,
                        translation::bedrock::COMMANDS_WEATHER_CLEAR,
                        [],
                    )
                }
                WeatherMode::Rain => {
                    let processed_duration =
                        duration.unwrap_or_else(|| rand::random_range(12_000..=24_000));

                    weather.set_weather_parameters(&world, 0, processed_duration, true, false);
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_WEATHER_SET_RAIN,
                        translation::bedrock::COMMANDS_WEATHER_RAIN,
                        [],
                    )
                }
                WeatherMode::Thunder => {
                    let processed_duration =
                        duration.unwrap_or_else(|| rand::random_range(3_600..=15_600));

                    weather.set_weather_parameters(&world, 0, processed_duration, true, true);
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_WEATHER_SET_THUNDER,
                        translation::bedrock::COMMANDS_WEATHER_THUNDER,
                        [],
                    )
                }
            }
        };

        sender.send_message(message);

        // Vanilla returns -1 when duration is not specified
        Ok(duration.unwrap_or(-1))
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("clear")
                .then(
                    argument(ARG_DURATION, TimeArgumentConsumer::new()).execute(Executor {
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
                    argument(ARG_DURATION, TimeArgumentConsumer::new()).execute(Executor {
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
                    argument(ARG_DURATION, TimeArgumentConsumer::new()).execute(Executor {
                        mode: WeatherMode::Thunder,
                    }),
                )
                .execute(Executor {
                    mode: WeatherMode::Thunder,
                }),
        )
}
