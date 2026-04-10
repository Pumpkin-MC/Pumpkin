use uuid::Uuid;
use pumpkin_data::translation;
use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::argument_types::entity::ONLY_PLAYERS_ALLOWED_ERROR_TYPE;
use crate::command::argument_types::entity_selector::EntitySelector;
use crate::command::argument_types::entity_selector::parser::EntitySelectorParser;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use crate::net::GameProfile;

pub const UNKNOWN_PLAYER_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(translation::ARGUMENT_PLAYER_UNKNOWN);

/// A result from the [`GameProfileArgumentType`], which can be resolved into
/// one or more [`GameProfile`]s, successfully or not.
pub enum GameProfileResult {
    Selector(EntitySelector),
    Name(String),
    Uuid(Uuid),
}

impl GameProfileResult {
    /// Resolves this result with the help of a [`CommandSource`].
    pub async fn resolve(&self, source: &CommandSource) -> Result<Vec<GameProfile>, CommandSyntaxError> {
        let players = match self {
            Self::Selector(selector) => selector.find_players(source).await,
            Self::Name(name) => source.server()
                    .get_player_by_name(name.as_str())
                    .map_or_else(
                        || Err(UNKNOWN_PLAYER_ERROR_TYPE.create_without_context()),
                        |p| Ok(vec![p])
                    ),
            Self::Uuid(uuid) => source.server()
                .get_player_by_uuid(*uuid)
                .map_or_else(
                    || Err(UNKNOWN_PLAYER_ERROR_TYPE.create_without_context()),
                    |p| Ok(vec![p])
                )
        }?;

        Ok(
            players.iter()
            .map(|p| &p.gameprofile)
            .cloned()
            .collect()
        )
    }
}

/// An argument type to parse one or more [`GameProfile`]s.
///
/// Use [`GameProfileArgumentType::get`] to automatically get a `Vec` of
/// [`GameProfile`]s for an argument by providing a [`CommandContext`] and the
/// argument's name.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct GameProfileArgumentType;

impl ArgumentType for GameProfileArgumentType {
    type Item = GameProfileResult;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        self.parse_with_allow_selectors(reader, true)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::GameProfile
    }

    fn examples(&self) -> Vec<String> {
        examples!(
            "Herobrine",
            "98765",
            "@a",
            "@p[limit=2]"
        )
    }
}

impl GameProfileArgumentType {
    fn parse_with_allow_selectors(
        self,
        reader: &mut StringReader,
        allow_selectors: bool,
    ) -> Result<<Self as ArgumentType>::Item, CommandSyntaxError> {
        if reader.peek() == Some('@') {
            // We read a selector variable.
            let parser = EntitySelectorParser::new(reader, allow_selectors);
            let selector = parser.parse()?;
            if selector.includes_entities {
                Err(ONLY_PLAYERS_ALLOWED_ERROR_TYPE.create(reader))
            } else {
                Ok(GameProfileResult::Selector(selector))
            }
        } else {
            // We read a UUID or player name.
            let i = reader.cursor();
            while reader.can_read_char() && reader.peek() != Some(' ') {
                reader.skip();
            }
            let string = &reader.string()[i..reader.cursor()];
            if let Ok(uuid) = Uuid::try_parse(string) {
                Ok(GameProfileResult::Uuid(uuid))
            } else {
                Ok(GameProfileResult::Name(string.to_owned()))
            }
        }
    }

    /// Tries to get any number of [`GameProfile`]s from a parsed argument of the provided [`CommandContext`].
    pub async fn get(context: &CommandContext<'_>, name: &str) -> Result<Vec<GameProfile>, CommandSyntaxError> {
        context.get_argument::<GameProfileResult>(name)?
            .resolve(context.source.as_ref()).await
    }
}
