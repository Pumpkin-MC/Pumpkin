use std::str::FromStr;
use std::sync::Arc;

use crate::command::CommandSender;
use crate::command::args::ConsumeResult;
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::entity::EntityBase;
use crate::server::Server;
use pumpkin_data::entity::EntityType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};
use pumpkin_util::GameMode;
use uuid::Uuid;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

#[allow(dead_code)]
pub enum EntitySelectorType {
    Source,
    NearestPlayer,
    NearestEntity,
    RandomPlayer,
    AllPlayers,
    AllEntities,
    NamedPlayer(String),
    Uuid(Uuid),
}

// todo tags
#[allow(dead_code)]
pub enum ValueCondition<T> {
    Equals(T),
    NotEquals(T),
}

#[allow(dead_code)]
pub enum ComparableValueCondition<T> {
    Equals(T),
    NotEquals(T),
    GreaterThan(T),
    LessThan(T),
    GreaterThanOrEquals(T),
    LessThanOrEquals(T),
    Between(T, T),
}

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq)]
pub enum EntityFilterSort {
    Arbitrary,
    Nearest,
    Furthest,
    Random,
}

#[allow(dead_code)]
pub enum EntityFilter {
    X(ComparableValueCondition<f64>),
    Y(ComparableValueCondition<f64>),
    Z(ComparableValueCondition<f64>),
    Distance(ComparableValueCondition<f64>),
    Dx(ComparableValueCondition<f64>),
    Dy(ComparableValueCondition<f64>),
    Dz(ComparableValueCondition<f64>),
    XRotation(ComparableValueCondition<f64>),
    YRotation(ComparableValueCondition<f64>),
    Score(ComparableValueCondition<i32>),
    Tag(ValueCondition<String>),
    Team(ValueCondition<String>),
    Name(ValueCondition<String>),
    Type(ValueCondition<&'static EntityType>),
    Nbt(NbtCompound),
    Gamemode(ValueCondition<GameMode>),
    Limit(usize),
    Sort(EntityFilterSort),
}

impl FromStr for EntityFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, '=');
        let key = parts.next().ok_or("Missing key in entity filter")?;
        let mut value = parts.next().ok_or("Missing value in entity filter")?;
        let negate = value.starts_with('!');
        if negate {
            value = &value[1..];
        }

        match key {
            "type" => {
                let entity_type =
                    EntityType::from_name(value).ok_or(format!("Invalid entity type {value}"))?;
                Ok(Self::Type(if negate {
                    ValueCondition::NotEquals(entity_type)
                } else {
                    ValueCondition::Equals(entity_type)
                }))
            }
            "limit" => {
                let limit = value
                    .parse::<usize>()
                    .map_err(|_| "Invalid limit value".to_string())?;
                if negate {
                    return Err("Negation of limit is not allowed".to_string());
                }
                Ok(Self::Limit(limit))
            }
            "sort" => {
                let sort = match value {
                    "arbitrary" => EntityFilterSort::Arbitrary,
                    "nearest" => EntityFilterSort::Nearest,
                    "furthest" => EntityFilterSort::Furthest,
                    "random" => EntityFilterSort::Random,
                    _ => return Err(format!("Invalid sort type {value}")),
                };
                if negate {
                    return Err("Negation of sort is not allowed".to_string());
                }
                Ok(Self::Sort(sort))
            }
            _ => todo!("{key}"),
        }
    }
}

/// <https://minecraft.wiki/w/Target_selectors>
#[allow(dead_code)]
pub struct TargetSelector {
    pub selector_type: EntitySelectorType,
    pub conditions: Vec<EntityFilter>,
    pub player_only: bool,
}

impl TargetSelector {
    /// Creates a new target selector with the specified type and default conditions.
    fn new(selector_type: EntitySelectorType) -> Self {
        let mut filter = Vec::new();
        match selector_type {
            EntitySelectorType::Source => filter.push(EntityFilter::Limit(1)),
            EntitySelectorType::NearestPlayer | EntitySelectorType::NearestEntity => {
                filter.push(EntityFilter::Sort(EntityFilterSort::Nearest));
                filter.push(EntityFilter::Limit(1));
            }
            EntitySelectorType::RandomPlayer => {
                filter.push(EntityFilter::Sort(EntityFilterSort::Random));
                filter.push(EntityFilter::Limit(1));
            }
            EntitySelectorType::NamedPlayer(_) | EntitySelectorType::Uuid(_) => {
                // Named or UUID selectors should only return one entity
                filter.push(EntityFilter::Limit(1));
            }
            _ => {}
        }
        Self {
            player_only: matches!(
                selector_type,
                EntitySelectorType::AllPlayers
                    | EntitySelectorType::NearestPlayer
                    | EntitySelectorType::RandomPlayer
                    | EntitySelectorType::NamedPlayer(_)
            ),
            selector_type,
            conditions: filter,
        }
    }

    #[must_use]
    pub fn get_sort(&self) -> Option<EntityFilterSort> {
        self.conditions.iter().rev().find_map(|f| {
            if let EntityFilter::Sort(sort) = f {
                Some(*sort)
            } else {
                None
            }
        })
    }

    #[must_use]
    pub fn get_limit(&self) -> usize {
        self.conditions
            .iter()
            .rev()
            .find_map(|f| {
                if let EntityFilter::Limit(limit) = f {
                    Some(*limit)
                } else {
                    None
                }
            })
            .unwrap_or(usize::MAX)
    }
}

impl FromStr for TargetSelector {
    type Err = String;

    fn from_str(arg: &str) -> Result<Self, Self::Err> {
        if arg.starts_with('@') {
            let body: Vec<_> = arg.splitn(2, '[').collect();
            let r#type = match body[0] {
                "@a" => EntitySelectorType::AllPlayers,
                "@e" => EntitySelectorType::AllEntities,
                "@s" => EntitySelectorType::Source,
                "@p" => EntitySelectorType::NearestPlayer,
                "@r" => EntitySelectorType::RandomPlayer,
                "@n" => EntitySelectorType::NearestEntity,
                _ => return Err(format!("Invalid target selector type {}", body[0])),
            };
            let mut selector = Self::new(r#type);
            if body.len() < 2 {
                // No conditions specified, return the selector with default conditions
                return Ok(selector);
            }
            // parse conditions
            if body[1].as_bytes()[body[1].len() - 1] != b']' {
                return Err("Target selector must end with ]".to_string());
            }
            let conditions: Vec<_> = body[1][..body[1].len() - 1]
                .split(',')
                .map(str::trim)
                .collect();
            for s in conditions {
                selector.conditions.push(EntityFilter::from_str(s)?);
            }
            Ok(selector)
        } else if let Ok(uuid) = Uuid::parse_str(arg) {
            Ok(Self::new(EntitySelectorType::Uuid(uuid)))
        } else {
            Ok(Self::new(EntitySelectorType::NamedPlayer(arg.to_string())))
        }
    }
}

/// todo: implement (currently just calls [`super::arg_player::PlayerArgumentConsumer`])
///
/// For selecting zero, one or multiple entities, eg. using @s, a player name, @a or @e
pub struct EntitiesArgumentConsumer;

impl GetClientSideArgParser for EntitiesArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        // todo: investigate why this does not accept target selectors
        ArgumentType::Entity { flags: 0 }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for EntitiesArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s_opt: Option<&'a str> = args.pop();

        let Some(s) = s_opt else {
            return Box::pin(async move { None });
        };

        let entity_selector = match s.parse::<TargetSelector>() {
            Ok(selector) => selector,
            Err(e) => {
                log::debug!("Failed to parse target selector '{s}': {e}");
                return Box::pin(async move { None }); // Return a Future resolving to None
            }
        };

        Box::pin(async move {
            // todo: command context
            // This is the required asynchronous operation.
            let entities = server.select_entities(&entity_selector, Some(sender)).await;

            Some(Arg::Entities(entities))
        })
    }
}

impl DefaultNameArgConsumer for EntitiesArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "targets"
    }
}

impl<'a> FindArg<'a> for EntitiesArgumentConsumer {
    type Data = &'a [Arc<dyn EntityBase>];

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Entities(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
