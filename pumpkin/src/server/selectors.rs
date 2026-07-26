use super::Server;
use crate::command::CommandSender;
use crate::command::args::entities::{
    EntityFilter, EntityFilterSort, EntitySelectorType, TargetSelector, ValueCondition,
};
use crate::entity::EntityBase;
use crate::entity::player::Player;
use pumpkin_data::entity::EntityType;
use rand::seq::SliceRandom;
use std::collections::HashSet;
use std::sync::Arc;

impl Server {
    #[allow(clippy::too_many_lines, clippy::option_if_let_else)]
    pub fn select_players(
        &self,
        target_selector: &TargetSelector,
        source: Option<&CommandSender>,
    ) -> Vec<Arc<Player>> {
        let mut players = match &target_selector.selector_type {
            EntitySelectorType::Source => source
                .and_then(CommandSender::as_player)
                .map_or_else(Vec::new, |player| vec![player]),
            EntitySelectorType::NearestPlayer
            | EntitySelectorType::NearestEntity
            | EntitySelectorType::RandomPlayer
            | EntitySelectorType::AllPlayers
            | EntitySelectorType::AllEntities => self.get_all_players(),
            EntitySelectorType::NamedPlayer(name) => self
                .get_player_by_name(name)
                .map_or_else(Vec::new, |player| vec![player]),
            EntitySelectorType::Uuid(uuid) => self
                .get_player_by_uuid(*uuid)
                .map_or_else(Vec::new, |player| vec![player]),
        };

        let player_type = EntityType::from_name("player").expect("entity type player must exist");
        let type_included = target_selector
            .conditions
            .iter()
            .filter_map(|f| {
                if let EntityFilter::Type(ValueCondition::Equals(entity_type)) = f {
                    Some(*entity_type)
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();
        let type_excluded = target_selector
            .conditions
            .iter()
            .filter_map(|f| {
                if let EntityFilter::Type(ValueCondition::NotEquals(entity_type)) = f {
                    Some(*entity_type)
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();

        players.retain(|_| {
            (type_excluded.is_empty() || !type_excluded.contains(player_type))
                && (type_included.is_empty() || type_included.contains(player_type))
        });

        let limit = target_selector.get_limit();
        if limit == 0 {
            return Vec::new();
        }

        match target_selector
            .get_sort()
            .unwrap_or(EntityFilterSort::Arbitrary)
        {
            EntityFilterSort::Arbitrary => players.into_iter().take(limit).collect(),
            EntityFilterSort::Random => {
                players.shuffle(&mut rand::rng());
                players.into_iter().take(limit).collect()
            }
            EntityFilterSort::Nearest | EntityFilterSort::Furthest => {
                let center = source.and_then(CommandSender::position).unwrap_or_default();
                let nearest_first = target_selector
                    .get_sort()
                    .is_none_or(|sort| sort == EntityFilterSort::Nearest);
                players.sort_by(|a, b| {
                    let a_distance = a.get_entity().pos.load().squared_distance_to_vec(&center);
                    let b_distance = b.get_entity().pos.load().squared_distance_to_vec(&center);
                    if nearest_first {
                        a_distance
                            .partial_cmp(&b_distance)
                            .unwrap_or(core::cmp::Ordering::Equal)
                    } else {
                        b_distance
                            .partial_cmp(&a_distance)
                            .unwrap_or(core::cmp::Ordering::Equal)
                    }
                });
                players.into_iter().take(limit).collect()
            }
        }
    }

    #[allow(clippy::too_many_lines, clippy::option_if_let_else)]
    pub fn select_entities(
        &self,
        target_selector: &TargetSelector,
        source: Option<&CommandSender>,
    ) -> Vec<Arc<dyn EntityBase>> {
        let all_entities_and_players = || {
            let mut entities = Vec::new();
            for world in self.worlds.load().iter() {
                entities.extend(world.entities.load().iter().cloned());
                entities.extend(
                    world
                        .players
                        .load()
                        .iter()
                        .cloned()
                        .map(|player| player as Arc<dyn EntityBase>),
                );
            }
            entities
        };
        let all_players_as_entities = || {
            self.get_all_players()
                .into_iter()
                .map(|player| player as Arc<dyn EntityBase>)
                .collect::<Vec<_>>()
        };

        let mut entities = match &target_selector.selector_type {
            EntitySelectorType::Source => source
                .and_then(CommandSender::as_player)
                .map_or_else(Vec::new, |player| vec![player as Arc<dyn EntityBase>]),
            EntitySelectorType::NearestPlayer
            | EntitySelectorType::RandomPlayer
            | EntitySelectorType::AllPlayers => all_players_as_entities(),
            EntitySelectorType::NearestEntity | EntitySelectorType::AllEntities => {
                all_entities_and_players()
            }
            EntitySelectorType::NamedPlayer(name) => self
                .get_player_by_name(name)
                .map_or_else(Vec::new, |player| vec![player as Arc<dyn EntityBase>]),
            EntitySelectorType::Uuid(uuid) => self
                .get_player_by_uuid(*uuid)
                .map_or_else(Vec::new, |player| vec![player as Arc<dyn EntityBase>]),
        };

        let type_included = target_selector
            .conditions
            .iter()
            .filter_map(|f| {
                if let EntityFilter::Type(ValueCondition::Equals(entity_type)) = f {
                    Some(*entity_type)
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();
        let type_excluded = target_selector
            .conditions
            .iter()
            .filter_map(|f| {
                if let EntityFilter::Type(ValueCondition::NotEquals(entity_type)) = f {
                    Some(*entity_type)
                } else {
                    None
                }
            })
            .collect::<HashSet<_>>();
        entities.retain(|entity| {
            // Filter by entity type
            (type_excluded.is_empty() || !type_excluded.contains(&entity.get_entity().entity_type))
                && (type_included.is_empty()
                    || type_included.contains(&entity.get_entity().entity_type))
        });

        let limit = target_selector.get_limit();
        if limit == 0 {
            return vec![];
        }

        match target_selector
            .get_sort()
            .unwrap_or(EntityFilterSort::Arbitrary)
        {
            EntityFilterSort::Arbitrary => entities.into_iter().take(limit).collect(),
            EntityFilterSort::Random => {
                entities.shuffle(&mut rand::rng());
                entities.into_iter().take(limit).collect()
            }
            EntityFilterSort::Nearest | EntityFilterSort::Furthest => {
                let center = source.and_then(CommandSender::position).unwrap_or_default();
                let nearest_first = target_selector
                    .get_sort()
                    .is_none_or(|sort| sort == EntityFilterSort::Nearest);
                entities.sort_by(|a, b| {
                    let a_distance = a.get_entity().pos.load().squared_distance_to_vec(&center);
                    let b_distance = b.get_entity().pos.load().squared_distance_to_vec(&center);
                    if nearest_first {
                        a_distance
                            .partial_cmp(&b_distance)
                            .unwrap_or(core::cmp::Ordering::Equal)
                    } else {
                        b_distance
                            .partial_cmp(&a_distance)
                            .unwrap_or(core::cmp::Ordering::Equal)
                    }
                });
                entities.into_iter().take(limit).collect()
            }
        }
    }
}
