use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::argument_types::entity_selector::EntitySelector;
use crate::command::argument_types::entity_selector::parser::EntitySelectorParser;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use pumpkin_data::translation;

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

pub const ENTITY_SELECTOR_PERMISSION: &str = "minecraft:command.selector";

/// Represents an argument type parsing a [`TargetSelector`]. This argument type is used
/// to target entities.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum EntityArgumentType {
    Entity,
    Entities,
    Player,
    Players,
}

impl EntityArgumentType {
    const fn is_single(self) -> bool {
        matches!(self, Self::Entity | Self::Player)
    }

    const fn is_players_only(self) -> bool {
        matches!(self, Self::Player | Self::Players)
    }
}

impl ArgumentType for EntityArgumentType {
    type Item = EntitySelector;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        self.parse_with_allow_selectors(reader, true)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::Entity {
            flags: (self.is_single() as u8 * JavaClientArgumentType::ENTITY_FLAG_ONLY_SINGLE)
                | (self.is_players_only() as u8 * JavaClientArgumentType::ENTITY_FLAG_PLAYERS_ONLY),
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
        let selector = {
            let parser = EntitySelectorParser::new(reader, allow_selectors);
            parser.parse()?
        };
        if selector.max_selected > 1 && self.is_single() {
            reader.set_cursor(0);
            Err(if self.is_players_only() {
                NOT_SINGLE_PLAYER_ERROR_TYPE.create(reader)
            } else {
                NOT_SINGLE_ENTITY_ERROR_TYPE.create(reader)
            })
        } else if selector.includes_entities
            && self.is_players_only()
            && !selector.is_current_entity
        {
            reader.set_cursor(0);
            Err(ONLY_PLAYERS_ALLOWED_ERROR_TYPE.create(reader))
        } else {
            Ok(selector)
        }
    }
}
