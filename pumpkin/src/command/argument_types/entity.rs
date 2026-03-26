use crate::command::args::entities::TargetSelector;
use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use pumpkin_data::translation;

/// A [`CommandErrorType`] to tell that using an entity selector variable is not allowed.
pub const SELECTORS_NOT_ALLOWED_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_SELECTOR_NOT_ALLOWED);

/// A [`CommandErrorType`] to tell that no entities could be found.
pub const NO_ENTITIES_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_NOTFOUND_ENTITY);

/// A [`CommandErrorType`] to tell that no players could be found.
pub const NO_PLAYERS_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_NOTFOUND_PLAYER);

/// A [`CommandErrorType`] to tell that only players are allowed for an entity selector.
pub const ONLY_PLAYERS_ALLOWED_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_PLAYER_ENTITIES);

/// A [`CommandErrorType`] to tell that only 1 entity is allowed for an entity selector.
pub const NOT_SINGLE_ENTITY_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_ENTITY_TOOMANY);

/// A [`CommandErrorType`] to tell that only 1 player is allowed for an entity selector.
pub const NOT_SINGLE_PLAYER_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_PLAYER_TOOMANY);

/// Represents an argument type parsing a [`TargetSelector`]. This argument type is used
/// to target entities.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EntityArgumentType {
    single: bool,
    players_only: bool,
}

impl ArgumentType for EntityArgumentType {
    type Item = TargetSelector;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        self.parse_with_allow_selectors(reader, true)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::Entity {
            flags: (self.single as u8 * JavaClientArgumentType::ENTITY_FLAG_ONLY_SINGLE)
                | (self.players_only as u8 * JavaClientArgumentType::ENTITY_FLAG_PLAYERS_ONLY),
        }
    }

    fn examples(&self) -> Vec<String> {
        examples!(
            "Herobrine",
            "98765",
            "@a",
            "@p[limit=2]",
            "@e[type=creeper]",
            "5e5677dc-bb96-4669-a4ab-60468b574e8e"
        )
    }
}

impl EntityArgumentType {
    fn parse_with_allow_selectors(
        self,
        reader: &mut StringReader,
        allow_selectors: bool,
    ) -> Result<<Self as ArgumentType>::Item, CommandSyntaxError> {
        todo!()
    }
}
