use crate::command::argument_types::entity;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use pumpkin_data::data_component_impl::EntityTypeOrTag;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;
use uuid::Uuid;

/// A permission allowing a [`CommandSource`] to use entity selectors.
const ENTITY_SELECTOR_PERMISSION: &str = "minecraft:command.selector";

/// Represents a structure that can target entities.
///
/// Not all selectors represented by this object start with `@`; this is also used for
/// selectors like *plain player names* and *bare UUIDs*.
pub struct EntitySelector {
    /// The maximum number of possible entities that can be selected.
    max_selected: i32,
    /// Whether this selector includes all entities rather than just players.
    includes_entities: bool,
    /// A list of predicates that must be satisfied by an entity to be part
    /// of this selector. Used for things like checking a game mode.
    predicates: Vec<EntitySelectorPredicate>,
    /// The distance range that this selector allows.
    range: Option<Range>,
    /// A function corresponding to how positions are provided for this selector, provided
    /// an initial [`Vector3<f64>`].
    position_function: PositionFunction,
    /// The limiting bounding box for this selector.
    bounding_box: Option<BoundingBox>,
    /// The sorting order of this selector.
    order: Order,
    /// Whether the selector represents the currently executing entity. Only true if `@s` is used.
    is_current_entity: bool,
    /// The limiting player name of this selector.
    player_name: Option<String>,
    /// The limiting UUID of this selector.
    entity_uuid: Option<Uuid>,
    /// The limiting entity type or tag for this selector.
    entity_type_or_tag: EntityTypeOrTag,
    /// Whether this selector uses a selector variable (like `@p`).
    uses_selector_variable: bool,
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
        todo!()
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
        todo!()
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
        if let Some(range) = self.range {
            list.push(EntitySelectorPredicate::Range(range, pos));
        }

        EntitySelectorPredicate::new_all_of(list)
    }
}

/// Represents a range of `f64`s, whose bounds' minima and maxima may be optional.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range {
    bounds: RangeBounds,
    squared_bounds: RangeBounds,
}

impl Range {
    /// Returns whether a number satisfies this range.
    #[must_use]
    pub fn matches(&self, number: f64) -> bool {
        self.bounds.min.is_none_or(|min| min <= number)
            && self.bounds.max.is_none_or(|max| max >= number)
    }

    /// Returns whether a number, in its squared form, satisfies this range.
    #[must_use]
    pub fn matches_square(&self, number: f64) -> bool {
        self.squared_bounds.min.is_none_or(|min| min <= number)
            && self.squared_bounds.max.is_none_or(|max| max >= number)
    }
}

/// Represents a single range bound of `f64`s, whose bounds may be optional.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RangeBounds {
    min: Option<f64>,
    max: Option<f64>,
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
    OverrideWithParser,
}

/// An order to choose entities that could be selected by an entity selector.
pub enum Order {
    Nearest,
    Furthest,
    Random,
    Arbitrary,
}

/// A predicate for an entity selector.
#[derive(Debug, Clone)]
pub enum EntitySelectorPredicate {
    // TODO: add the rest of the predicates
    BoundingBox(BoundingBox),
    Range(Range, Vector3<f64>),

    AllOf(Vec<Self>),
}

impl EntitySelectorPredicate {
    #[must_use]
    pub const fn new_all_of(predicates: Vec<Self>) -> Self {
        Self::AllOf(predicates)
    }

    pub fn test(&self, entity: &dyn EntityBase) -> bool {
        match self {
            Self::BoundingBox(bounding_box) => entity
                .get_entity()
                .bounding_box
                .load()
                .intersects(bounding_box),
            Self::Range(range, pos) => {
                range.matches_square(entity.get_entity().pos.load().squared_distance_to_vec(pos))
            }
            Self::AllOf(predicates) => {
                predicates.iter().all(|predicate| predicate.test(entity))
            }
        }
    }
}
