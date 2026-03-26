use crate::command::argument_types::selector::entity_selector::{
    EntitySelectorPredicate, Order,
};
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use pumpkin_data::entity::EntityType;
use pumpkin_data::translation;
use pumpkin_util::math::bounds::{FloatDegreeBounds, IntBounds};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

pub const INVALID_NAME_OR_UUID_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_INVALID);

pub const UNKNOWN_SELECTOR_TYPE_ERROR_TYPE: CommandErrorType<1> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_SELECTOR_UNKNOWN);

pub const SELECTORS_NOT_ALLOWED_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_SELECTOR_NOT_ALLOWED);

pub const MISSING_SELECTOR_TYPE_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_SELECTOR_MISSING);

pub const EXPECTED_END_OF_OPTIONS_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_OPTIONS_UNTERMINATED);

pub const EXPECTED_OPTION_VALUE_ERROR_TYPE: CommandErrorType<1> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_OPTIONS_VALUELESS);

/// A struct to parse an [`EntitySelector`].
#[derive(Debug)]
struct EntitySelectorParser<'a> {
    reader: StringReader<'a>,
    allow_selectors: bool,
    max_selected: i32,
    includes_entities: bool,
    is_world_limited: bool,
    distance: Option<IntBounds>,
    pos: Vector3<Option<f64>>,
    delta: Vector3<Option<f64>>,
    rotation: Vector2<FloatDegreeBounds>,
    predicates: Vec<EntitySelectorPredicate>,
    order: Order,
    is_current_entity: bool,
    player_name: Option<String>,
    entity_uuid: Option<Uuid>,

    has_name_equals: bool,
    has_name_not_equals: bool,
    is_limited: bool,
    is_sorted: bool,
    has_gamemode_equals: bool,
    has_gamemode_not_equals: bool,
    // TODO: Add team predicate
    entity_type: &'static EntityType,
    entity_type_is_inverse: bool,
    // TODO: Add score and advancement predicate
    uses_selectors: bool,
    start_position: usize,
}
