use crate::command::argument_types::selector::entity_selector::{EntitySelector, EntitySelectorPredicate, Order, PositionFunction};
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use pumpkin_data::entity::EntityType;
use pumpkin_data::translation;
use pumpkin_util::math::bounds::{DoubleBounds, FloatDegreeBounds, IntBounds};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::text::TextComponent;
use crate::command::errors::command_syntax_error::CommandSyntaxError;

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
    reader: &'a mut StringReader<'a>,
    allow_selectors: bool,
    max_selected: i32,
    includes_entities: bool,
    is_world_limited: bool,
    distance: Option<DoubleBounds>,
    experience_level: Option<IntBounds>,
    pos: Vector3<Option<f64>>,
    delta: Vector3<Option<f64>>,
    rotation: Vector2<Option<FloatDegreeBounds>>,
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
    entity_type: Option<&'static EntityType>,
    entity_type_is_inverse: bool,
    // TODO: Add score and advancement predicate
    uses_selectors: bool,
    start_position: usize,
}

impl<'a> EntitySelectorParser<'a> {
    /// Constructs a new [`EntitySelectorParser`].
    ///
    /// # Arguments
    ///
    /// * `reader`: The [`StringReader`] to use while parsing the entity selector.
    /// * `allow_selectors`: Whether to allow selector variables (like `@s` or `@p).
    pub fn new(reader: &'a mut StringReader<'a>, allow_selectors: bool) -> Self {
        Self {
            reader, allow_selectors,
            max_selected: 0,
            includes_entities: false,
            is_world_limited: false,
            distance: None,
            experience_level: None,
            pos: Default::default(),
            delta: Default::default(),
            rotation: Default::default(),
            predicates: vec![],
            order: Order::Arbitrary,
            is_current_entity: false,
            player_name: None,
            entity_uuid: None,
            has_name_equals: false,
            has_name_not_equals: false,
            is_limited: false,
            is_sorted: false,
            has_gamemode_equals: false,
            has_gamemode_not_equals: false,
            entity_type: None,
            entity_type_is_inverse: false,
            uses_selectors: false,
            start_position: 0,
        }
    }

    fn selector(mut self) -> EntitySelector {

        // We finalize our predicates.
        if let Some(x) = self.rotation.x {
            self.predicates.push(EntitySelectorPredicate::Rotation(x, |e| e.yaw.load()))
        }
        if let Some(y) = self.rotation.y {
            self.predicates.push(EntitySelectorPredicate::Rotation(y, |e| e.yaw.load()))
        }
        if let Some(level) = self.experience_level {
            self.predicates.push(EntitySelectorPredicate::ExperienceLevel(level));
        }

        let bounding_box = if self.delta.x.is_none() && self.delta.y.is_none() && self.delta.z.is_none() {
            if let Some(distance) = self.distance && let Some(max) = distance.max() {
                let max = max as f64;
                Some(
                    BoundingBox::new(Vector3::new(-max, -max, -max), Vector3::new(max + 1.0, max + 1.0, max + 1.0))
                )
            } else {
                None
            }
        } else {
            Some(
                Self::create_bounding_box(
                    Vector3::new(
                        self.delta.x.unwrap_or(0.0),
                        self.delta.y.unwrap_or(0.0),
                        self.delta.z.unwrap_or(0.0),
                    )
                )
            )
        };

        let position_function = if self.pos.x.is_none() && self.pos.y.is_none() && self.pos.z.is_none() {
            PositionFunction::Identity
        } else {
            PositionFunction::OverrideWithParser(self.pos)
        };

        EntitySelector {
            max_selected: self.max_selected,
            includes_entities: self.includes_entities,
            predicates: self.predicates,
            distance: self.distance,
            position_function,
            bounding_box,
            order: self.order,
            is_current_entity: self.is_current_entity,
            player_name: self.player_name,
            entity_uuid: self.entity_uuid,
            entity_type: self.entity_type,
            uses_selector_variable: false,
            is_world_limited: false,
        }
    }

    fn create_bounding_box(pos: Vector3<f64>) -> BoundingBox {
        BoundingBox::new(
            Vector3::new(
                pos.x.min(0.0), pos.y.min(0.0), pos.z.min(0.0)
            ),
            Vector3::new(
                pos.x.max(0.0) + 1.0, pos.y.max(0.0) + 1.0, pos.z.max(0.0) + 1.0
            )
        )
    }

    /// Limits the parsed selector's reach to players.
    pub fn limit_to_players(&mut self) {
        self.entity_type = Some(&EntityType::PLAYER);
    }

    /// Tries to parse the selector from the provided [`StringReader`].
    pub fn parse(mut self) -> Result<EntitySelector, CommandSyntaxError> {
        self.start_position = self.reader.cursor();
        // TODO: suggestions
        if self.reader.peek().is_some_and(|c| c == '@') {
            if self.allow_selectors {
                let error = Err(SELECTORS_NOT_ALLOWED_ERROR_TYPE.create(self.reader));
                self.reader.skip();
                self.parse_selector()?;
                return error
            } else {
                self.parse_name_or_uuid()?;
            }
        }
        Ok(self.selector())
    }

    fn parse_selector(&mut self) -> Result<(), CommandSyntaxError> {
        self.uses_selectors = true;
        // TODO: suggestions
        if !self.reader.can_read_char() {
            return Err(MISSING_SELECTOR_TYPE_ERROR_TYPE.create(self.reader))
        }
        let i = self.reader.cursor();
        let char = self.reader.read().unwrap();
        let mut add_alive_predicate = false;
        match char {
            'a' => {
                self.max_selected = i32::MAX;
                self.includes_entities = false;
                self.order = Order::Arbitrary;
                self.limit_to_players();
            }
            'e' => {
                self.max_selected = i32::MAX;
                self.includes_entities = false;
                self.order = Order::Arbitrary;
                add_alive_predicate = true;
            }
            'n' => {
                self.max_selected = 1;
                self.includes_entities = false;
                self.order = Order::Nearest;
                add_alive_predicate = true;
            }
            'p' => {
                self.max_selected = 1;
                self.includes_entities = false;
                self.order = Order::Nearest;
                self.limit_to_players();
            }
            'r' => {
                self.max_selected = 1;
                self.includes_entities = false;
                self.order = Order::Random;
                self.limit_to_players();
            }
            's' => {
                self.max_selected = 1;
                self.includes_entities = true;
                self.is_current_entity = true;
            }
            _ => {
                self.reader.set_cursor(i);
                let mut selector = "@".to_string();
                selector.push(char);
                return Err(UNKNOWN_SELECTOR_TYPE_ERROR_TYPE.create(self.reader, TextComponent::text(selector)));
            }
        }
        if add_alive_predicate {
            self.predicates.push(EntitySelectorPredicate::IsAlive)
        }
        // TODO: suggestions
        if self.reader.peek().is_some_and(|c| c == '[') {
            self.reader.skip();
            //
            self.parse_options()?;
        }
        Ok(())
    }

    fn parse_name_or_uuid(&mut self) -> Result<(), CommandSyntaxError> {
        todo!()
    }

    fn parse_options(&mut self) -> Result<(), CommandSyntaxError> {
        todo!()
    }
}
