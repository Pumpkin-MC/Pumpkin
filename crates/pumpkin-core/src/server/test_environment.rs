use std::collections::HashSet;
use std::io::Cursor;
use std::sync::Arc;

use pumpkin_data::dimension::Dimension;
use pumpkin_data::game_rules::{GameRule, GameRuleRegistry, GameRuleValue};
use pumpkin_gametest::{GameTestError, GameTestResult};
use pumpkin_nbt::Nbt;
use pumpkin_nbt::deserializer::NbtReadHelperJava;
use serde_json::Value;
use tracing::error;

use crate::command::{CommandSender, CommandSource};
use crate::server::Server;
use crate::world::World;

enum Environment {
    AllOf(Vec<Self>),
    ClockTime(i64),
    GameRules(Vec<(GameRule, GameRuleValue<i64, bool>)>),
    Weather(WeatherKind),
    Function {
        setup: Option<String>,
        teardown: Option<String>,
    },
}

enum SavedEnvironment {
    AllOf(Vec<Self>),
    ClockTime(i64),
    GameRules(Vec<(GameRule, GameRuleValue<i64, bool>)>),
    Weather(WeatherKind),
    Function(Option<String>),
}

#[derive(Clone, Copy)]
enum WeatherKind {
    Clear,
    Rain,
    Thunder,
}

impl WeatherKind {
    fn parse(value: &str) -> GameTestResult<Self> {
        match value {
            "clear" => Ok(Self::Clear),
            "rain" => Ok(Self::Rain),
            "thunder" => Ok(Self::Thunder),
            _ => Err(invalid(format!("Unknown test weather '{value}'"))),
        }
    }

    fn apply(self, world: &World) {
        let (clear_time, rain_time, raining, thundering) = match self {
            Self::Clear => (100_000, 0, false, false),
            Self::Rain => (0, 100_000, true, false),
            Self::Thunder => (0, 100_000, true, true),
        };
        world
            .weather
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_weather_parameters(world, clear_time, rain_time, raining, thundering);
    }

    fn current(world: &World) -> Self {
        if world.is_thundering() {
            Self::Thunder
        } else if world.is_raining() {
            Self::Rain
        } else {
            Self::Clear
        }
    }
}

fn invalid(message: impl Into<String>) -> GameTestError {
    GameTestError::World(message.into())
}

fn string_field<'a>(value: &'a Value, name: &str) -> GameTestResult<&'a str> {
    value
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| invalid(format!("Test environment is missing '{name}'")))
}

fn parse_game_rules(value: &Value) -> GameTestResult<Environment> {
    let rules = value
        .get("rules")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid("Test environment game_rules is missing rules"))?;
    let defaults = GameRuleRegistry::default();
    let mut parsed = Vec::with_capacity(rules.len());
    for (name, value) in rules {
        let short_name = name.strip_prefix("minecraft:").unwrap_or(name);
        let rule = GameRule::all()
            .iter()
            .find(|rule| rule.to_string() == short_name)
            .cloned()
            .ok_or_else(|| invalid(format!("Unknown test game rule '{name}'")))?;
        let selected =
            match defaults.get(&rule) {
                GameRuleValue::Bool(_) => {
                    let parsed = value
                        .as_bool()
                        .or_else(|| match value.as_i64() {
                            Some(0) => Some(false),
                            Some(1) => Some(true),
                            _ => None,
                        })
                        .or_else(|| value.as_str().and_then(|text| text.parse().ok()));
                    GameRuleValue::Bool(parsed.ok_or_else(|| {
                        invalid(format!("Invalid value for test game rule '{name}'"))
                    })?)
                }
                GameRuleValue::Int(_) => {
                    let parsed = value
                        .as_i64()
                        .or_else(|| value.as_str().and_then(|text| text.parse().ok()));
                    GameRuleValue::Int(parsed.ok_or_else(|| {
                        invalid(format!("Invalid value for test game rule '{name}'"))
                    })?)
                }
            };
        parsed.push((rule, selected));
    }
    Ok(Environment::GameRules(parsed))
}

fn resolve(
    server: &Server,
    value: &Value,
    seen: &mut HashSet<String>,
    depth: usize,
) -> GameTestResult<Environment> {
    if depth > 16 {
        return Err(invalid("Test environment nesting exceeds 16 levels"));
    }
    if let Some(id) = value.as_str() {
        let id = if id.contains(':') {
            id.to_owned()
        } else {
            format!("minecraft:{id}")
        };
        if !seen.insert(id.clone()) {
            return Err(invalid(format!("Cyclic test environment '{id}'")));
        }
        let resolved = if id == "minecraft:default" {
            Ok(Environment::AllOf(Vec::new()))
        } else {
            let bytes = server
                .datapack_manager
                .get_dynamic_registry_entry_data("minecraft:test_environment", &id)
                .ok_or_else(|| invalid(format!("Unknown test environment '{id}'")))?;
            let mut cursor = Cursor::new(bytes.as_ref());
            let mut reader = NbtReadHelperJava::new(&mut cursor);
            let nbt = Nbt::read_unnamed(&mut reader)
                .map_err(|error| invalid(format!("Invalid test environment '{id}': {error}")))?;
            let json = pumpkin_world::world_info::data_files::nbt_tag_to_json(
                &pumpkin_nbt::tag::NbtTag::Compound(nbt.root_tag),
            );
            resolve(server, &json, seen, depth + 1)
        };
        seen.remove(&id);
        return resolved;
    }
    let kind = string_field(value, "type")?;
    match kind {
        "minecraft:all_of" | "all_of" => {
            let definitions = value
                .get("definitions")
                .and_then(Value::as_array)
                .ok_or_else(|| invalid("Test environment all_of is missing definitions"))?;
            if definitions.len() > 64 {
                return Err(invalid("Test environment has too many definitions"));
            }
            definitions
                .iter()
                .map(|definition| resolve(server, definition, seen, depth + 1))
                .collect::<GameTestResult<Vec<_>>>()
                .map(Environment::AllOf)
        }
        "minecraft:clock_time" | "clock_time" => {
            let clock = string_field(value, "clock")?;
            if clock != "minecraft:overworld" && clock != "overworld" {
                return Err(invalid(format!("Unsupported test world clock '{clock}'")));
            }
            let time = value
                .get("time")
                .and_then(Value::as_i64)
                .filter(|time| *time >= 0)
                .ok_or_else(|| invalid("Test clock time must be non-negative"))?;
            Ok(Environment::ClockTime(time))
        }
        "minecraft:time_of_day" | "time_of_day" => {
            let time = value
                .get("time")
                .and_then(Value::as_i64)
                .filter(|time| *time >= 0)
                .ok_or_else(|| invalid("Test time of day must be non-negative"))?;
            Ok(Environment::ClockTime(time))
        }
        "minecraft:weather" | "weather" => Ok(Environment::Weather(WeatherKind::parse(
            string_field(value, "weather")?,
        )?)),
        "minecraft:function" | "function" => Ok(Environment::Function {
            setup: value
                .get("setup")
                .and_then(Value::as_str)
                .map(str::to_owned),
            teardown: value
                .get("teardown")
                .and_then(Value::as_str)
                .map(str::to_owned),
        }),
        "minecraft:game_rules" | "game_rules" => parse_game_rules(value),
        _ => Err(invalid(format!(
            "Unsupported test environment type '{kind}'"
        ))),
    }
}

fn run_function(server: &Arc<Server>, world: &Arc<World>, id: &str) {
    let source: CommandSource = CommandSender::Console
        .into_source(server)
        .with_world(world.clone())
        .with_silent();
    if let Err(error) = server
        .datapack_manager
        .execute_function(server, &source, id)
    {
        error!(target: "pumpkin::gametest", "Test batch function '{id}' failed: {error}");
    }
}

impl Environment {
    fn activate(self, server: &Arc<Server>, world: &Arc<World>) -> SavedEnvironment {
        match self {
            Self::AllOf(definitions) => SavedEnvironment::AllOf(
                definitions
                    .into_iter()
                    .map(|definition| definition.activate(server, world))
                    .collect(),
            ),
            Self::ClockTime(time) => {
                let clock_world = server.get_world_from_dimension(&Dimension::OVERWORLD);
                let previous = clock_world.get_time_of_day();
                clock_world.set_time_of_day(time);
                SavedEnvironment::ClockTime(previous)
            }
            Self::GameRules(rules) => {
                let previous = rules
                    .into_iter()
                    .map(|(rule, value)| {
                        let previous = world.get_game_rule(&rule);
                        world.set_game_rule(&rule, value);
                        (rule, previous)
                    })
                    .collect();
                SavedEnvironment::GameRules(previous)
            }
            Self::Weather(weather) => {
                let previous = WeatherKind::current(world);
                weather.apply(world);
                SavedEnvironment::Weather(previous)
            }
            Self::Function { setup, teardown } => {
                if let Some(id) = setup {
                    run_function(server, world, &id);
                }
                SavedEnvironment::Function(teardown)
            }
        }
    }
}

impl SavedEnvironment {
    fn teardown(self, server: &Arc<Server>, world: &Arc<World>) {
        match self {
            Self::AllOf(definitions) => {
                for definition in definitions.into_iter().rev() {
                    definition.teardown(server, world);
                }
            }
            Self::ClockTime(time) => server
                .get_world_from_dimension(&Dimension::OVERWORLD)
                .set_time_of_day(time),
            Self::GameRules(rules) => {
                for (rule, value) in rules {
                    world.set_game_rule(&rule, value);
                }
            }
            Self::Weather(previous) => {
                world
                    .weather
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .reset_weather_cycle(world);
                previous.apply(world);
            }
            Self::Function(teardown) => {
                if let Some(id) = teardown {
                    run_function(server, world, &id);
                }
            }
        }
    }
}

pub(super) fn activate(
    server: &Arc<Server>,
    world: &Arc<World>,
    value: &Value,
) -> GameTestResult<Box<dyn std::any::Any + Send>> {
    let definition = resolve(server, value, &mut HashSet::new(), 0)?;
    Ok(Box::new(definition.activate(server, world)))
}

pub(super) fn teardown(
    server: &Arc<Server>,
    world: &Arc<World>,
    saved: Box<dyn std::any::Any + Send>,
) {
    if let Ok(saved) = saved.downcast::<SavedEnvironment>() {
        saved.teardown(server, world);
    }
}
