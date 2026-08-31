use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::time::TimeArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Changes the weather.";
const PERMISSION: &str = "minecraft:command.weather";

#[derive(Clone, Copy)]
enum WeatherMode {
    Clear,
    Rain,
    Thunder,
}

struct WeatherExecutor {
    mode: WeatherMode,
    has_duration: bool,
}

impl CommandExecutor for WeatherExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let duration = if self.has_duration {
            Some(TimeArgumentType::get(context, "duration")?)
        } else {
            None
        };

        let world = context.source.world();
        let message = {
            let mut weather = world
                .weather
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            match self.mode {
                WeatherMode::Clear => {
                    let processed_duration =
                        duration.unwrap_or_else(|| rand::random_range(12_000..=180_000));

                    weather.set_weather_parameters(world, processed_duration, 0, false, false);
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_WEATHER_SET_CLEAR,
                        translation::bedrock::COMMANDS_WEATHER_CLEAR,
                        [],
                    )
                }
                WeatherMode::Rain => {
                    let processed_duration =
                        duration.unwrap_or_else(|| rand::random_range(12_000..=24_000));

                    weather.set_weather_parameters(world, 0, processed_duration, true, false);
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_WEATHER_SET_RAIN,
                        translation::bedrock::COMMANDS_WEATHER_RAIN,
                        [],
                    )
                }
                WeatherMode::Thunder => {
                    let processed_duration =
                        duration.unwrap_or_else(|| rand::random_range(3600..=15_600));

                    weather.set_weather_parameters(world, 0, processed_duration, true, true);
                    TextComponent::translate_cross(
                        translation::java::COMMANDS_WEATHER_SET_THUNDER,
                        translation::bedrock::COMMANDS_WEATHER_THUNDER,
                        [],
                    )
                }
            }
        };

        context.source.send_feedback(message, true);

        Ok(1)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("weather", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                literal("clear")
                    .executes(WeatherExecutor {
                        mode: WeatherMode::Clear,
                        has_duration: false,
                    })
                    .then(argument("duration", TimeArgumentType::any()).executes(
                        WeatherExecutor {
                            mode: WeatherMode::Clear,
                            has_duration: true,
                        },
                    )),
            )
            .then(
                literal("rain")
                    .executes(WeatherExecutor {
                        mode: WeatherMode::Rain,
                        has_duration: false,
                    })
                    .then(argument("duration", TimeArgumentType::any()).executes(
                        WeatherExecutor {
                            mode: WeatherMode::Rain,
                            has_duration: true,
                        },
                    )),
            )
            .then(
                literal("thunder")
                    .executes(WeatherExecutor {
                        mode: WeatherMode::Thunder,
                        has_duration: false,
                    })
                    .then(argument("duration", TimeArgumentType::any()).executes(
                        WeatherExecutor {
                            mode: WeatherMode::Thunder,
                            has_duration: true,
                        },
                    )),
            ),
    );
}
