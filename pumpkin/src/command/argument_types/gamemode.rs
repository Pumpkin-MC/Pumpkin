use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_data::translation;
use pumpkin_util::GameMode;
use pumpkin_util::text::TextComponent;
use std::pin::Pin;
use std::str::FromStr;

pub const INVALID_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_GAMEMODE_INVALID,
    translation::java::ARGUMENT_GAMEMODE_INVALID,
);

pub struct GameModeArgumentType;

impl ArgumentType for GameModeArgumentType {
    type Item = GameMode;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let string = reader.read_unquoted_string();
        GameMode::from_str(&string)
            .map_err(|_| INVALID_ERROR_TYPE.create(reader, TextComponent::text(string)))
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::Gamemode
    }

    fn list_suggestions<'a>(
        &'a self,
        _context: &'a CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send + 'a>> {
        Box::pin(async move {
            for gamemode in GameMode::VALUES {
                builder = builder.filter_and_suggest_one(gamemode.name());
            }
            builder.build()
        })
    }

    fn examples(&self) -> Vec<String> {
        examples!("survival", "creative")
    }
}

impl_copy_get!(GameModeArgumentType, GameMode);
