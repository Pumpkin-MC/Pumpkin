use std::str::FromStr;
use std::sync::Arc;

use crate::command::CommandSender;
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::entity::EntityBase;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_data::entity::EntityType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};
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

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        let negate = s.starts_with('!');
        if negate {
            s = &s[1..];
        }
        let mut parts = s.splitn(2, '=');
        let key = parts.next().ok_or("Missing key in entity filter")?;
        let value = parts.next().ok_or("Missing value in entity filter")?;

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
            _ => todo!(),
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

    pub fn get_sort(&self) -> Option<EntityFilterSort> {
        self.conditions.iter().find_map(|f| {
            if let EntityFilter::Sort(sort) = f {
                Some(*sort)
            } else {
                None
            }
        })
    }

    pub fn get_limit(&self) -> usize {
        self.conditions
            .iter()
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
        if arg.contains('[') {
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
            if body[1].as_bytes()[body[1].len() - 1] != b']' {
                return Err("Target selector must end with ]".to_string());
            }
            let conditions: Vec<_> = body[1][..body[1].len() - 1]
                .split(',')
                .map(str::trim)
                .collect();
            let conditions = conditions
                .iter()
                .filter_map(|s| EntityFilter::from_str(s).ok())
                .collect::<Vec<_>>();
            selector.conditions.extend(conditions);
            Ok(selector)
        } else if let Ok(uuid) = Uuid::parse_str(arg) {
            return Ok(Self {
                selector_type: EntitySelectorType::Uuid(uuid),
                conditions: Vec::new(),
                player_only: false,
            });
        } else {
            return Ok(Self {
                selector_type: EntitySelectorType::NamedPlayer(arg.to_string()),
                conditions: Vec::new(),
                player_only: true,
            });
        }
    }
}

/// todo: implement (currently just calls [`super::arg_player::PlayerArgumentConsumer`])
///
/// For selecting zero, one or multiple entities, eg. using @s, a player name, @a or @e
pub struct EntitiesArgumentConsumer;

impl GetClientSideArgParser for EntitiesArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        // todo: investigate why this does not accept target selectors
        ArgumentType::Entity { flags: 0 }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for EntitiesArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        src: &CommandSender,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;
        let entity_selector = match s.parse::<TargetSelector>() {
            Ok(selector) => selector,
            Err(e) => {
                log::debug!("Failed to parse target selector '{s}': {e}");
                return None;
            }
        };
        // todo: command context
        let entities = server.select_entities(&entity_selector, Some(src)).await;

        Some(Arg::Entities(entities))
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        Ok(None)
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
