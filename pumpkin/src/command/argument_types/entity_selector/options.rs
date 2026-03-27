use crate::command::argument_types::entity_selector::parser::EntitySelectorParser;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

pub const UNKNOWN_OPTION_ERROR_TYPE: CommandErrorType<1> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_OPTIONS_UNKNOWN);
pub const INAPPLICABLE_OPTION_ERROR_TYPE: CommandErrorType<1> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_OPTIONS_INAPPLICABLE);
pub const DISTANCE_NEGATIVE_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_OPTIONS_DISTANCE_NEGATIVE);
pub const LEVEL_NEGATIVE_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_OPTIONS_LEVEL_NEGATIVE);
pub const LIMIT_TOO_SMALL_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_OPTIONS_LIMIT_TOOSMALL);
pub const SORT_UNKNOWN_ERROR_TYPE: CommandErrorType<1> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_OPTIONS_SORT_IRREVERSIBLE);
pub const GAMEMODE_INVALID_ERROR_TYPE: CommandErrorType<1> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_OPTIONS_MODE_INVALID);
pub const ENTITY_TYPE_INVALID_ERROR_TYPE: CommandErrorType<1> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_OPTIONS_TYPE_INVALID);

/// Options to customize an [`EntitySelectorParser`].
///
/// These can be used in commands while specifying entity selectors.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum EntitySelectorOption {
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

    pub const fn description_translation_key(self) -> &'static str {
        match self {
            Self::Name => translation::ARGUMENT_ENTITY_OPTIONS_NAME_DESCRIPTION,
            Self::Distance => translation::ARGUMENT_ENTITY_OPTIONS_DISTANCE_DESCRIPTION,
            Self::Level => translation::ARGUMENT_ENTITY_OPTIONS_LEVEL_DESCRIPTION,
            Self::X => translation::ARGUMENT_ENTITY_OPTIONS_X_DESCRIPTION,
            Self::Y => translation::ARGUMENT_ENTITY_OPTIONS_Y_DESCRIPTION,
            Self::Z => translation::ARGUMENT_ENTITY_OPTIONS_Z_DESCRIPTION,
            Self::Dx => translation::ARGUMENT_ENTITY_OPTIONS_DX_DESCRIPTION,
            Self::Dy => translation::ARGUMENT_ENTITY_OPTIONS_DY_DESCRIPTION,
            Self::Dz => translation::ARGUMENT_ENTITY_OPTIONS_DZ_DESCRIPTION,
            Self::XRotation => translation::ARGUMENT_ENTITY_OPTIONS_X_ROTATION_DESCRIPTION,
            Self::YRotation => translation::ARGUMENT_ENTITY_OPTIONS_Y_ROTATION_DESCRIPTION,
            Self::Limit => translation::ARGUMENT_ENTITY_OPTIONS_LIMIT_DESCRIPTION,
            Self::Sort => translation::ARGUMENT_ENTITY_OPTIONS_SORT_DESCRIPTION,
            Self::Gamemode => translation::ARGUMENT_ENTITY_OPTIONS_GAMEMODE_DESCRIPTION,
            Self::Team => translation::ARGUMENT_ENTITY_OPTIONS_TEAM_DESCRIPTION,
            Self::Type => translation::ARGUMENT_ENTITY_OPTIONS_TYPE_DESCRIPTION,
            Self::Tag => translation::ARGUMENT_ENTITY_OPTIONS_TAG_DESCRIPTION,
            Self::Nbt => translation::ARGUMENT_ENTITY_OPTIONS_NBT_DESCRIPTION,
            Self::Scores => translation::ARGUMENT_ENTITY_OPTIONS_SCORES_DESCRIPTION,
            Self::Advancements => translation::ARGUMENT_ENTITY_OPTIONS_ADVANCEMENTS_DESCRIPTION,
            Self::Predicate => translation::ARGUMENT_ENTITY_OPTIONS_PREDICATE_DESCRIPTION,
        }
    }

    pub fn description(self) -> TextComponent {
        TextComponent::translate(self.description_translation_key(), [])
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
        match self {
            _ => {
                tracing::warn!("Unimplemented entity selector option: {:?}", self);
            }
        }
        Ok(())
    }

    /// Returns whether this option can be used by the provided [`EntitySelectorParser`].
    pub const fn can_use(self, parser: &EntitySelectorParser) -> bool {
        match self {
            Self::Name => !parser.has_name_equals,
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
            Self::Limit => !parser.is_current_entity && !parser.is_limited,
            Self::Sort => !parser.is_current_entity && !parser.is_sorted,
            Self::Gamemode => !parser.has_gamemode_equals,
            Self::Team => !parser.has_team_equals,
            Self::Type => parser.entity_type.is_none(),
            Self::Scores => !parser.has_scores,
            Self::Advancements => !parser.has_advancements,
            Self::Tag | Self::Nbt | Self::Predicate => true,
        }
    }
}
