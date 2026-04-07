use crate::command::argument_types::entity_selector::option::{
    EntitySelectorOption, INAPPLICABLE_OPTION_ERROR_TYPE, UNKNOWN_OPTION_ERROR_TYPE,
};
use crate::command::argument_types::entity_selector::{
    EntitySelector, EntitySelectorPredicate, Order, PositionFunction, RotationType,
};
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use bitflags::bitflags;
use pumpkin_data::entity::EntityType;
use pumpkin_data::translation;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::bounds::{DoubleBounds, FloatDegreeBounds, IntBounds};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
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

bitflags! {
    /// A list of bit flags to set entity selector parser properties.
    /// These are supposed to be set by entity selector options.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Flags: u32 {
        /// Whether the execution of the selector is limited to the current world (dimension).
        const WORLD_LIMITED = 1 << 0;
        /// Whether the `name` option has been set.
        const NAME_EQUALS_SET = 1 << 1;
        /// Whether the `!name` option has been set.
        const NAME_NOT_EQUALS_SET = 1 << 2;
        /// Whether the `gamemode` option has been set.
        const GAMEMODE_EQUALS_SET = 1 << 3;
        /// Whether the `!gamemode` option has been set.
        const GAMEMODE_NOT_EQUALS_SET = 1 << 4;
        /// Whether the `team` option has been set.
        const TEAM_EQUALS_SET = 1 << 5;
        /// Whether the `!team` option has been set.
        const TEAM_NOT_EQUALS_SET = 1 << 6;
        /// Whether the `limit` option has been set.
        const LIMIT_SET = 1 << 7;
        /// Whether the `sort` option has been set.
        const SORT_SET = 1 << 8;
        /// Whether the `type` (entity type) option is inverted.
        const ENTITY_TYPE_INVERTED = 1 << 9;
        /// Whether the `scores` option has been set.
        const SCORES_SET = 1 << 10;
        /// Whether the `advancements` option has been set.
        const ADVANCEMENTS_SET = 1 << 11;
    }
}

/// A struct to parse an [`EntitySelector`].
///
/// * `'b` is the lifetime of the mutable reference to the [`StringReader`].
/// * `'a` is the lifetime of the [`StringReader`].
#[derive(Debug)]
pub struct EntitySelectorParser<'b, 'a> {
    pub reader: &'b mut StringReader<'a>,
    pub(crate) max_selected: i32,
    pub(crate) distance: Option<DoubleBounds>,
    pub(crate) experience_level: Option<IntBounds>,
    pub(crate) pos: Vector3<Option<f64>>,
    pub(crate) delta: Vector3<Option<f64>>,
    pub(crate) rotation: Vector2<Option<FloatDegreeBounds>>,
    predicates: Vec<EntitySelectorPredicate>,
    pub(crate) order: Order,
    player_name: Option<String>,
    entity_uuid: Option<Uuid>,
    pub(crate) entity_type: Option<&'static EntityType>,
    start_position: usize,

    allows_selector_variable: bool,
    uses_selector_variable: bool,
    includes_entities: bool,
    pub(crate) is_current_entity: bool,

    flags: Flags,
}

impl<'b, 'a> EntitySelectorParser<'b, 'a> {
    /// Constructs a new [`EntitySelectorParser`].
    ///
    /// # Arguments
    ///
    /// * `reader`: The [`StringReader`] to use while parsing the entity selector.
    /// * `allow_selectors`: Whether to allow selector variables (like `@s` or `@p`).
    pub fn new(reader: &'b mut StringReader<'a>, allow_selectors: bool) -> Self {
        Self {
            reader,
            max_selected: 0,
            distance: None,
            experience_level: None,
            pos: Vector3::default(),
            delta: Vector3::default(),
            rotation: Vector2::default(),
            predicates: vec![],
            order: Order::Arbitrary,
            player_name: None,
            entity_uuid: None,
            entity_type: None,
            start_position: 0,
            allows_selector_variable: allow_selectors,
            uses_selector_variable: false,
            includes_entities: false,
            is_current_entity: false,
            flags: Flags::empty(),
        }
    }

    fn selector(mut self) -> EntitySelector {
        // We finalize our predicates.
        if let Some(x) = self.rotation.x {
            self.predicates
                .push(EntitySelectorPredicate::Rotation(x, RotationType::Yaw));
        }
        if let Some(y) = self.rotation.y {
            self.predicates
                .push(EntitySelectorPredicate::Rotation(y, RotationType::Pitch));
        }
        if let Some(level) = self.experience_level {
            self.predicates
                .push(EntitySelectorPredicate::ExperienceLevel(level));
        }

        let bounding_box =
            if self.delta.x.is_none() && self.delta.y.is_none() && self.delta.z.is_none() {
                if let Some(distance) = self.distance
                    && let Some(max) = distance.max()
                {
                    Some(BoundingBox::new(
                        Vector3::new(-max, -max, -max),
                        Vector3::new(max + 1.0, max + 1.0, max + 1.0),
                    ))
                } else {
                    None
                }
            } else {
                Some(Self::create_bounding_box(Vector3::new(
                    self.delta.x.unwrap_or(0.0),
                    self.delta.y.unwrap_or(0.0),
                    self.delta.z.unwrap_or(0.0),
                )))
            };

        let position_function =
            if self.pos.x.is_none() && self.pos.y.is_none() && self.pos.z.is_none() {
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
            uses_selector_variable: self.uses_selector_variable,
            is_world_limited: self.flags.contains(Flags::WORLD_LIMITED),
        }
    }

    fn create_bounding_box(pos: Vector3<f64>) -> BoundingBox {
        BoundingBox::new(
            Vector3::new(pos.x.min(0.0), pos.y.min(0.0), pos.z.min(0.0)),
            Vector3::new(
                pos.x.max(0.0) + 1.0,
                pos.y.max(0.0) + 1.0,
                pos.z.max(0.0) + 1.0,
            ),
        )
    }

    /// Limits the parsed selector's reach to players.
    pub const fn limit_to_players(&mut self) {
        self.entity_type = Some(&EntityType::PLAYER);
    }

    /// Tries to parse the selector from the provided [`StringReader`].
    pub fn parse(mut self) -> Result<EntitySelector, CommandSyntaxError> {
        self.start_position = self.reader.cursor();
        if self.reader.peek() == Some('@') {
            if !self.allows_selector_variable {
                return Err(SELECTORS_NOT_ALLOWED_ERROR_TYPE.create(self.reader));
            }
            self.reader.skip();
            self.parse_selector()?;
        } else {
            self.parse_name_or_uuid()?;
        }
        Ok(self.selector())
    }

    fn parse_selector(&mut self) -> Result<(), CommandSyntaxError> {
        self.uses_selector_variable = true;
        if !self.reader.can_read_char() {
            return Err(MISSING_SELECTOR_TYPE_ERROR_TYPE.create(self.reader));
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
                self.includes_entities = true;
                self.order = Order::Arbitrary;
                add_alive_predicate = true;
            }
            'n' => {
                self.max_selected = 1;
                self.includes_entities = true;
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
                return Err(UNKNOWN_SELECTOR_TYPE_ERROR_TYPE
                    .create(self.reader, TextComponent::text(selector)));
            }
        }
        if add_alive_predicate {
            self.predicates.push(EntitySelectorPredicate::IsAlive);
        }
        if self.reader.peek() == Some('[') {
            self.reader.skip();
            //
            self.parse_options()?;
        }
        Ok(())
    }

    fn parse_name_or_uuid(&mut self) -> Result<(), CommandSyntaxError> {
        let i = self.reader.cursor();
        let string = self.reader.read_string()?;
        if let Ok(uuid) = string.parse() {
            // The string is a UUID.
            self.entity_uuid = Some(uuid);
            self.includes_entities = true;
        } else {
            // Check for a player name.
            if string.is_empty() || string.len() > 16 {
                self.reader.set_cursor(i);
                return Err(INVALID_NAME_OR_UUID_ERROR_TYPE.create(self.reader));
            }
            self.includes_entities = false;
            self.player_name = Some(string);
        }
        self.max_selected = 1;

        Ok(())
    }

    fn parse_options(&mut self) -> Result<(), CommandSyntaxError> {
        self.reader.skip_whitespace();
        while self.reader.can_read_char() && self.reader.peek() != Some(']') {
            self.reader.skip_whitespace();
            let i = self.reader.cursor();
            let string = self.reader.read_string()?;
            // Try to get the option.
            let option = string.parse::<EntitySelectorOption>();
            if let Ok(option) = option {
                if !option.can_use(self) {
                    return Err(INAPPLICABLE_OPTION_ERROR_TYPE
                        .create(self.reader, TextComponent::text(string)));
                }
                // Now, we start parsing the option.
                self.reader.skip_whitespace();
                if self.reader.peek() != Some('=') {
                    self.reader.set_cursor(i);
                    return Err(EXPECTED_OPTION_VALUE_ERROR_TYPE
                        .create(self.reader, TextComponent::text(string)));
                }
                self.reader.skip();
                self.reader.skip_whitespace();
                option.modify_parser(self)?;
                self.reader.skip_whitespace();
                if let Some(peeked) = self.reader.peek() {
                    if peeked != ',' {
                        if peeked != ']' {
                            return Err(EXPECTED_END_OF_OPTIONS_ERROR_TYPE.create(self.reader));
                        }
                        break;
                    }
                    self.reader.skip();
                }
            } else {
                self.reader.set_cursor(i);
                return Err(
                    UNKNOWN_OPTION_ERROR_TYPE.create(self.reader, TextComponent::text(string))
                );
            }
        }
        if self.reader.can_read_char() {
            self.reader.skip();
            Ok(())
        } else {
            Err(EXPECTED_END_OF_OPTIONS_ERROR_TYPE.create(self.reader))
        }
    }

    /// Adds a single predicate to this parser.
    pub fn add_predicate(&mut self, predicate: EntitySelectorPredicate) {
        self.predicates.push(predicate);
    }

    /// Returns whether this parser's current cursor state tells that the
    /// currently-parsed entity selector option is inverted.
    ///
    /// This method also skips whitespace when required.
    pub fn consume_inverted_start(&mut self) -> bool {
        self.reader.skip_whitespace();
        if self.reader.peek() == Some('!') {
            self.reader.skip();
            self.reader.skip_whitespace();
            true
        } else {
            false
        }
    }

    /// Returns whether this parser's current cursor state tells that the
    /// currently-parsed entity selector option is a tag.
    ///
    /// This method also skips whitespace when required.
    pub fn consume_tag_start(&mut self) -> bool {
        self.reader.skip_whitespace();
        if self.reader.peek() == Some('#') {
            self.reader.skip();
            self.reader.skip_whitespace();
            true
        } else {
            false
        }
    }

    /// Sets a flag of this parser for options.
    pub fn set_flag(&mut self, flag: Flags, value: bool) {
        self.flags.set(flag, value);
    }

    /// Returns whether a flag is set for this parser for options.
    #[must_use]
    pub const fn has_flag(&self, flag: Flags) -> bool {
        self.flags.contains(flag)
    }

    /// Sets this parse to not include non-player entities.
    pub const fn set_includes_entities(&mut self, value: bool) {
        self.includes_entities = value;
    }
}
