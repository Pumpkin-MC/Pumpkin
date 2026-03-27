mod option;
pub mod parser;

use crate::command::argument_types::entity;
use crate::command::argument_types::entity::ENTITY_SELECTOR_PERMISSION;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};
use crate::world::World;
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::bounds::{DoubleBounds, FloatDegreeBounds, IntBounds};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use rand::seq::SliceRandom;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use uuid::Uuid;

/// Represents a structure that can target entities.
///
/// Not all selectors represented by this object start with `@`; this is also used for
/// selectors like *plain player names* and *bare UUIDs*.
pub struct EntitySelector {
    /// The maximum number of possible entities that can be selected.
    pub max_selected: i32,
    /// Whether this selector includes all entities rather than just players.
    pub includes_entities: bool,
    /// A list of predicates that must be satisfied by an entity to be part
    /// of this selector. Used for things like checking a game mode.
    pub predicates: Vec<EntitySelectorPredicate>,
    /// The distance range that this selector allows.
    pub distance: Option<DoubleBounds>,
    /// A function corresponding to how positions are provided for this selector, provided
    /// an initial [`Vector3<f64>`].
    pub position_function: PositionFunction,
    /// The limiting bounding box for this selector.
    pub bounding_box: Option<BoundingBox>,
    /// The sorting order of this selector.
    pub order: Order,
    /// Whether the selector represents the currently executing entity. Only true if `@s` is used.
    pub is_current_entity: bool,
    /// The limiting player name of this selector.
    pub player_name: Option<String>,
    /// The limiting UUID of this selector.
    pub entity_uuid: Option<Uuid>,
    /// The limiting entity type or tag for this selector.
    pub entity_type: Option<&'static EntityType>,
    /// Whether this selector uses a selector variable (like `@p`).
    pub uses_selector_variable: bool,
    /// Whether this selector limits entities to a certain world.
    pub is_world_limited: bool,
}

impl EntitySelector {
    /// Returns an [`Err`] if a [`CommandSource`] does not have permission to use
    /// this entity selector.
    pub async fn check_permissions(
        &self,
        source: &CommandSource,
    ) -> Result<(), CommandSyntaxError> {
        if self.uses_selector_variable && !source.has_permission(ENTITY_SELECTOR_PERMISSION).await {
            Err(entity::SELECTORS_NOT_ALLOWED_ERROR_TYPE.create_without_context())
        } else {
            Ok(())
        }
    }

    const fn result_limit(&self) -> usize {
        if matches!(self.order, Order::Arbitrary) {
            self.max_selected as usize
        } else {
            usize::MAX
        }
    }

    /// Tries to find a single entity represented by this selector.
    pub async fn find_single_entity(
        &self,
        source: &CommandSource,
    ) -> Result<Arc<dyn EntityBase>, CommandSyntaxError> {
        let list = self.find_entities(source).await?;
        match list.as_slice() {
            [] => Err(entity::NO_ENTITIES_ERROR_TYPE.create_without_context()),
            [entity] => Ok(entity.clone()),
            _ => Err(entity::NOT_SINGLE_ENTITY_ERROR_TYPE.create_without_context()),
        }
    }

    /// Tries to find any entities represented by this selector.
    pub async fn find_entities(
        &self,
        source: &CommandSource,
    ) -> Result<Vec<Arc<dyn EntityBase>>, CommandSyntaxError> {
        self.check_permissions(source).await?;
        if self.includes_entities {
            self.find_players(source)
                .await
                .map(|v| v.into_iter().map(|p| p as Arc<dyn EntityBase>).collect())
        } else if let Some(name) = self.player_name.as_ref() {
            // Try to get the player by name.
            let player = source
                .server
                .as_ref()
                .and_then(|s| s.get_player_by_name(name));
            Ok(player.map_or_else(Vec::new, |p| vec![p as Arc<dyn EntityBase>]))
        } else if let Some(uuid) = self.entity_uuid.as_ref() {
            // Try to get an entity by UUID.
            for world in source.server().worlds.load().iter() {
                if let Some(entity) = world.get_entity_by_uuid(*uuid) {
                    return Ok(vec![entity]);
                }
            }
            Ok(Vec::new())
        } else {
            let origin = self.position_function.apply(source.position);
            let bounding_box = self.absolute_bounding_box(origin);
            let predicate = self.predicate(origin, bounding_box);
            if self.is_current_entity {
                Ok(source
                    .entity
                    .as_ref()
                    .filter(|p| predicate.test(p.as_ref()))
                    .map_or_else(Vec::new, |p| vec![p.clone()]))
            } else {
                let mut list = Vec::new();
                if self.is_world_limited {
                    self.add_entities(&mut list, source.world().as_ref(), bounding_box, &predicate);
                } else {
                    for world in source.server().worlds.load().iter() {
                        self.add_entities(&mut list, world, bounding_box, &predicate);
                    }
                }

                Ok(self.sort_and_limit(origin, list))
            }
        }
    }

    fn add_entities(
        &self,
        list: &mut Vec<Arc<dyn EntityBase>>,
        world: &World,
        bounding_box: Option<BoundingBox>,
        predicate: &EntitySelectorPredicate,
    ) {
        let limit = self.result_limit();
        world.get_entities_and_add(list, limit, bounding_box, |e| predicate.test(e));
    }

    /// Tries to find a player represented by this selector.
    pub async fn find_player(
        &self,
        source: &CommandSource,
    ) -> Result<Arc<Player>, CommandSyntaxError> {
        let list = self.find_players(source).await?;
        if list.len() == 1 {
            Ok(list.first().unwrap().clone())
        } else {
            Err(entity::NO_PLAYERS_ERROR_TYPE.create_without_context())
        }
    }

    /// Tries to find any players represented by this selector.
    pub async fn find_players(
        &self,
        source: &CommandSource,
    ) -> Result<Vec<Arc<Player>>, CommandSyntaxError> {
        self.check_permissions(source).await?;
        if let Some(name) = self.player_name.as_ref() {
            // Try to get the player by name.
            let player = source
                .server
                .as_ref()
                .and_then(|s| s.get_player_by_name(name));
            Ok(player.map_or_else(Vec::new, |p| vec![p]))
        } else if let Some(uuid) = self.entity_uuid.as_ref() {
            // Try to get an entity by UUID.
            for world in source.server().worlds.load().iter() {
                if let Some(player) = world.get_player_by_uuid(*uuid) {
                    return Ok(vec![player]);
                }
            }
            Ok(Vec::new())
        } else {
            let origin = self.position_function.apply(source.position);
            let bounding_box = self.absolute_bounding_box(origin);
            let predicate = self.predicate(origin, bounding_box);
            if self.is_current_entity {
                Ok(source
                    .entity
                    .as_ref()
                    .and_then(|e| {
                        source
                            .server()
                            .get_player_by_uuid(e.get_entity().entity_uuid)
                    })
                    .filter(|p| predicate.test(p.as_ref()))
                    .map_or_else(Vec::new, |p| vec![p]))
            } else {
                let limit = self.result_limit();
                let mut list = Vec::new();
                if limit > 0 {
                    if self.is_world_limited {
                        Self::add_players_from_world(source.world(), &mut list, &predicate, limit);
                    } else {
                        for world in source.server().worlds.load().iter() {
                            Self::add_players_from_world(
                                world.as_ref(),
                                &mut list,
                                &predicate,
                                limit,
                            );
                        }
                    }
                }

                Ok(self.sort_and_limit(origin, list))
            }
        }
    }

    fn add_players_from_world(
        world: &World,
        list: &mut Vec<Arc<Player>>,
        predicate: &EntitySelectorPredicate,
        limit: usize,
    ) {
        for player in world.players.load().iter() {
            if predicate.test(player.as_ref()) {
                list.push(player.clone());
                if list.len() >= limit {
                    return;
                }
            }
        }
    }

    #[must_use]
    pub fn absolute_bounding_box(&self, pos: Vector3<f64>) -> Option<BoundingBox> {
        self.bounding_box.map(|b| b.shift(pos))
    }

    /// Returns a [`EntitySelectorPredicate`] to test against an entity.
    #[must_use]
    fn predicate(
        &self,
        pos: Vector3<f64>,
        bounding_box: Option<BoundingBox>,
    ) -> EntitySelectorPredicate {
        let mut list = self.predicates.clone();

        if let Some(bounding_box) = bounding_box {
            list.push(EntitySelectorPredicate::BoundingBox(bounding_box));
        }
        if let Some(distance_bounds) = self.distance {
            list.push(EntitySelectorPredicate::Distance(distance_bounds, pos));
        }

        EntitySelectorPredicate::new_all_of(list)
    }

    /// Sorts the provided entities according to this selector's order ([`Order::Arbitrary`] by default)
    /// and limits the number of entries depending on this selector's limit.
    fn sort_and_limit<T: EntityBase + ?Sized>(
        &self,
        origin: Vector3<f64>,
        entities: Vec<Arc<T>>,
    ) -> Vec<Arc<T>> {
        self.order.sort_and_limit(
            entities.len().min(self.max_selected as usize),
            origin,
            entities,
        )
    }
}

/// A function that may or may not manipulate a provided position to be used by a parser.
pub enum PositionFunction {
    /// A function that does not affect a position and simply returns it.
    Identity,
    /// A function that may override one or more coordinates of a position depending
    /// on the position of the parse.
    ///
    /// If a position coordinate of the parser is set, the provided position's
    /// corresponding coordinate is replaced, and the new position is returned.
    OverrideWithParser(Vector3<Option<f64>>),
}

impl PositionFunction {
    fn apply(&self, pos: Vector3<f64>) -> Vector3<f64> {
        match self {
            Self::Identity => pos,
            Self::OverrideWithParser(function_pos) => Vector3::new(
                function_pos.x.unwrap_or(pos.x),
                function_pos.y.unwrap_or(pos.y),
                function_pos.z.unwrap_or(pos.z),
            ),
        }
    }
}

/// An order to choose entities that could be selected by an entity selector.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Order {
    Nearest,
    Furthest,
    Random,
    Arbitrary,
}

impl Order {
    fn sort_with_comparator_usage_function<T: EntityBase + ?Sized>(
        mut entities: Vec<Arc<T>>,
        f: impl Fn(&Arc<T>, &Arc<T>) -> std::cmp::Ordering,
    ) -> Vec<Arc<T>> {
        entities.sort_by(f);
        entities
    }

    fn distance_comparator<T: EntityBase + ?Sized>(
        origin: &Vector3<f64>,
        a: &Arc<T>,
        b: &Arc<T>,
    ) -> std::cmp::Ordering {
        a.get_entity()
            .pos
            .load()
            .squared_distance_to_vec(origin)
            .total_cmp(&b.get_entity().pos.load().squared_distance_to_vec(origin))
    }

    #[must_use]
    pub fn sort_and_limit<T: EntityBase + ?Sized>(
        &self,
        limit: usize,
        origin: Vector3<f64>,
        mut entities: Vec<Arc<T>>,
    ) -> Vec<Arc<T>> {
        match self {
            Self::Nearest => Self::sort_with_comparator_usage_function(entities, |a, b| {
                Self::distance_comparator(&origin, a, b)
            }),
            Self::Furthest => Self::sort_with_comparator_usage_function(entities, |a, b| {
                Self::distance_comparator(&origin, b, a)
            }),
            Self::Random => {
                let mut rng = rand::rng();
                entities.shuffle(&mut rng);
                entities
            }
            Self::Arbitrary => entities,
        }
        .into_iter()
        .take(limit)
        .collect()
    }
}

/// A predicate for an entity selector.
#[derive(Debug, Clone)]
pub enum EntitySelectorPredicate {
    /// A predicate to check whether an entity is alive.
    IsAlive,
    /// A predicate to check the experience level of an entity, if any.
    ExperienceLevel(IntBounds),
    /// A predicate to check the rotation coordinate of an entity.
    Rotation(FloatDegreeBounds, fn(&Entity) -> f32),
    /// A predicate to check whether an entity intersects a bounding box.
    BoundingBox(BoundingBox),
    /// A predicate to check whether an entity is within a specified range from some position.
    Distance(DoubleBounds, Vector3<f64>),

    /// Used to combine sub-predicates.
    AllOf(Vec<Self>),
}

impl EntitySelectorPredicate {
    #[must_use]
    pub const fn new_all_of(predicates: Vec<Self>) -> Self {
        Self::AllOf(predicates)
    }

    pub fn test(&self, entity: &dyn EntityBase) -> bool {
        match self {
            Self::IsAlive => entity.get_entity().is_alive(),
            Self::ExperienceLevel(bounds) => entity
                .get_player()
                .is_some_and(|p| bounds.matches(p.experience_level.load(Ordering::Relaxed))),
            Self::Rotation(bounds, f) => {
                let min = wrap_degrees(bounds.min().unwrap_or(0.0f32));
                let max = wrap_degrees(bounds.min().unwrap_or(360.0f32));
                let degrees = wrap_degrees(f(entity.get_entity()));
                if min > max {
                    degrees >= min || degrees <= max
                } else {
                    degrees >= min && degrees <= max
                }
            }
            Self::BoundingBox(bounding_box) => entity
                .get_entity()
                .bounding_box
                .load()
                .intersects(bounding_box),
            Self::Distance(bounds, pos) => {
                bounds.matches_square(entity.get_entity().pos.load().squared_distance_to_vec(pos))
            }
            Self::AllOf(predicates) => predicates.iter().all(|predicate| predicate.test(entity)),
        }
    }
}
