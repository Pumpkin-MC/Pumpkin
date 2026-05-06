use crate::command::argument_types::FromStringReader;
use crate::command::argument_types::entity_selector::parser::{EntitySelectorParser, Flags};
use crate::command::argument_types::entity_selector::{EntitySelectorPredicate, Order};
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use pumpkin_data::translation;
use pumpkin_util::GameMode;
use pumpkin_util::math::bounds::{DoubleBounds, FloatDegreeBounds, IntBounds};
use pumpkin_util::text::TextComponent;
use std::str::FromStr;

pub const UNKNOWN_OPTION_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_ENTITY_OPTIONS_UNKNOWN,
    translation::java::ARGUMENT_ENTITY_OPTIONS_UNKNOWN,
);
pub const INAPPLICABLE_OPTION_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_ENTITY_OPTIONS_INAPPLICABLE,
    translation::java::ARGUMENT_ENTITY_OPTIONS_INAPPLICABLE,
);
pub const DISTANCE_NEGATIVE_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENT_ENTITY_OPTIONS_DISTANCE_NEGATIVE,
    translation::java::ARGUMENT_ENTITY_OPTIONS_DISTANCE_NEGATIVE,
);
pub const LEVEL_NEGATIVE_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENT_ENTITY_OPTIONS_LEVEL_NEGATIVE,
    translation::java::ARGUMENT_ENTITY_OPTIONS_LEVEL_NEGATIVE,
);
pub const LIMIT_TOO_SMALL_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENT_ENTITY_OPTIONS_LIMIT_TOOSMALL,
    translation::java::ARGUMENT_ENTITY_OPTIONS_LIMIT_TOOSMALL,
);
pub const SORT_UNKNOWN_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_ENTITY_OPTIONS_SORT_IRREVERSIBLE,
    translation::java::ARGUMENT_ENTITY_OPTIONS_SORT_IRREVERSIBLE,
);
pub const GAMEMODE_INVALID_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_ENTITY_OPTIONS_MODE_INVALID,
    translation::java::ARGUMENT_ENTITY_OPTIONS_MODE_INVALID,
);

/// Options to customize an [`EntitySelectorParser`].
///
/// These can be used in commands while specifying entity selectors.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum EntitySelectorOption {
    Name,
    Distance,
    Level,
    X,
    Y,
    Z,
    Dx,
    Dy,
    Dz,
    XRotation,
    YRotation,
    Limit,
    Sort,
    Gamemode,
    Team,
    Type,
    Tag,
    Nbt,
    Scores,
    Advancements,
    Predicate,
}

/// Implements parsing for a coordinate option.
macro_rules! coordinate_option_impl {
    ($parser:ident, $vector:ident, $axis:ident) => {{
        $parser.set_flag(Flags::WORLD_LIMITED, true);
        $parser.$vector.$axis = Some($parser.reader.read_double()?);
        Ok(())
    }};
}

/// Implements parsing for a rotation option.
macro_rules! rotation_option_impl {
    ($parser:ident, $axis:ident) => {{
        $parser.rotation.$axis = Some(FloatDegreeBounds::from_reader($parser.reader)?);
        Ok(())
    }};
}

pub struct InvalidEntitySelectorOptionError;

impl FromStr for EntitySelectorOption {
    type Err = InvalidEntitySelectorOptionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(Self::Name),
            "distance" => Ok(Self::Distance),
            "level" => Ok(Self::Level),
            "x" => Ok(Self::X),
            "y" => Ok(Self::Y),
            "z" => Ok(Self::Z),
            "dx" => Ok(Self::Dx),
            "dy" => Ok(Self::Dy),
            "dz" => Ok(Self::Dz),
            "x_rotation" => Ok(Self::XRotation),
            "y_rotation" => Ok(Self::YRotation),
            "limit" => Ok(Self::Limit),
            "sort" => Ok(Self::Sort),
            "gamemode" => Ok(Self::Gamemode),
            "team" => Ok(Self::Team),
            "type" => Ok(Self::Type),
            "tag" => Ok(Self::Tag),
            "nbt" => Ok(Self::Nbt),
            "scores" => Ok(Self::Scores),
            "advancements" => Ok(Self::Advancements),
            "predicate" => Ok(Self::Predicate),
            _ => Err(InvalidEntitySelectorOptionError),
        }
    }
}

impl EntitySelectorOption {
    /// Returns the name required to specify this option.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Distance => "distance",
            Self::Level => "level",
            Self::X => "x",
            Self::Y => "y",
            Self::Z => "z",
            Self::Dx => "dx",
            Self::Dy => "dy",
            Self::Dz => "dz",
            Self::XRotation => "x_rotation",
            Self::YRotation => "y_rotation",
            Self::Limit => "limit",
            Self::Sort => "sort",
            Self::Gamemode => "gamemode",
            Self::Team => "team",
            Self::Type => "type",
            Self::Tag => "tag",
            Self::Nbt => "nbt",
            Self::Scores => "scores",
            Self::Advancements => "advancements",
            Self::Predicate => "predicate",
        }
    }

    /// Returns the name required to specify this option as a [`TextComponent`].
    pub fn name_component(self) -> TextComponent {
        TextComponent::text(self.name())
    }

    fn inapplicable_error(self, reader: &StringReader) -> CommandSyntaxError {
        INAPPLICABLE_OPTION_ERROR_TYPE.create(reader, self.name_component())
    }

    /// Modifies the provided [`EntitySelectorParser`].
    ///
    /// Any required fields will be parsed by this method using [`StringReader`]
    /// methods, and any required predicates will be added.
    /// Any found errors will be returned.
    pub fn modify_parser(
        self,
        parser: &mut EntitySelectorParser,
    ) -> Result<(), CommandSyntaxError> {
        let i = parser.reader.cursor();
        match self {
            Self::Distance => {
                let bounds = DoubleBounds::from_reader(parser.reader)?;
                if bounds.min().is_none_or(|n| n < 0.0) && bounds.max().is_none_or(|n| n < 0.0) {
                    parser.reader.set_cursor(i);
                    Err(DISTANCE_NEGATIVE_ERROR_TYPE.create(parser.reader))
                } else {
                    parser.distance = Some(bounds);
                    parser.set_flag(Flags::WORLD_LIMITED, true);
                    Ok(())
                }
            }
            Self::Level => {
                let bounds = IntBounds::from_reader(parser.reader)?;
                if bounds.min().is_none_or(|n| n < 0) && bounds.max().is_none_or(|n| n < 0) {
                    parser.reader.set_cursor(i);
                    Err(LEVEL_NEGATIVE_ERROR_TYPE.create(parser.reader))
                } else {
                    parser.experience_level = Some(bounds);
                    parser.set_includes_entities(false);
                    Ok(())
                }
            }
            Self::X => coordinate_option_impl!(parser, pos, x),
            Self::Y => coordinate_option_impl!(parser, pos, y),
            Self::Z => coordinate_option_impl!(parser, pos, z),
            Self::Dx => coordinate_option_impl!(parser, delta, x),
            Self::Dy => coordinate_option_impl!(parser, delta, y),
            Self::Dz => coordinate_option_impl!(parser, delta, z),
            Self::XRotation => rotation_option_impl!(parser, x),
            Self::YRotation => rotation_option_impl!(parser, y),
            Self::Limit => {
                let limit = parser.reader.read_int()?;
                if limit < 1 {
                    parser.reader.set_cursor(i);
                    Err(LIMIT_TOO_SMALL_ERROR_TYPE.create(parser.reader))
                } else {
                    parser.max_selected = limit;
                    parser.set_flag(Flags::LIMIT_SET, true);
                    Ok(())
                }
            }
            Self::Sort => {
                let string = parser.reader.read_unquoted_string();
                parser.order = match string.as_str() {
                    "nearest" => Ok(Order::Nearest),
                    "furthest" => Ok(Order::Furthest),
                    "random" => Ok(Order::Random),
                    "arbitrary" => Ok(Order::Arbitrary),
                    _ => {
                        parser.reader.set_cursor(i);
                        Err(SORT_UNKNOWN_ERROR_TYPE
                            .create(parser.reader, TextComponent::text(string)))
                    }
                }?;
                parser.set_flag(Flags::SORT_SET, true);
                Ok(())
            }
            Self::Gamemode => {
                let invert = parser.consume_inverted_start();
                if parser.has_flag(Flags::GAMEMODE_NOT_EQUALS_SET) && !invert {
                    return Err(self.inapplicable_error(parser.reader));
                }
                let string = parser.reader.read_unquoted_string();
                if let Ok(gamemode) = GameMode::from_str(&string) {
                    parser.set_includes_entities(false);
                    parser.add_predicate(EntitySelectorPredicate::GameMode(gamemode, invert));
                    parser.set_flag(
                        if invert {
                            Flags::GAMEMODE_NOT_EQUALS_SET
                        } else {
                            Flags::GAMEMODE_EQUALS_SET
                        },
                        true,
                    );
                    Ok(())
                } else {
                    Err(GAMEMODE_INVALID_ERROR_TYPE
                        .create(parser.reader, TextComponent::text(string)))
                }
            }
            _ => {
                tracing::warn!("Unimplemented entity selector option: {:?}", self);
                Err(UNKNOWN_OPTION_ERROR_TYPE.create_without_context(self.name_component()))
            }
        }
    }

    /// Returns whether this option can be used by the provided [`EntitySelectorParser`].
    pub const fn can_use(self, parser: &EntitySelectorParser) -> bool {
        match self {
            Self::Name => !parser.has_flag(Flags::NAME_EQUALS_SET),
            Self::Distance => parser.distance.is_none(),
            Self::Level => parser.experience_level.is_none(),
            Self::X => parser.pos.x.is_none(),
            Self::Y => parser.pos.y.is_none(),
            Self::Z => parser.pos.z.is_none(),
            Self::Dx => parser.delta.x.is_none(),
            Self::Dy => parser.delta.y.is_none(),
            Self::Dz => parser.delta.z.is_none(),
            Self::XRotation => parser.rotation.x.is_none(),
            Self::YRotation => parser.rotation.y.is_none(),
            Self::Limit => !parser.is_current_entity && !parser.has_flag(Flags::LIMIT_SET),
            Self::Sort => !parser.is_current_entity && !parser.has_flag(Flags::SORT_SET),
            Self::Gamemode => !parser.has_flag(Flags::GAMEMODE_EQUALS_SET),
            Self::Team => !parser.has_flag(Flags::TEAM_EQUALS_SET),
            Self::Type => parser.entity_type.is_none(),
            Self::Scores => !parser.has_flag(Flags::SCORES_SET),
            Self::Advancements => !parser.has_flag(Flags::ADVANCEMENTS_SET),
            Self::Tag | Self::Nbt | Self::Predicate => true,
        }
    }
}
