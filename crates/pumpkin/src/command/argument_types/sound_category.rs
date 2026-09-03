use pumpkin_data::sound::SoundCategory;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};

pub const ERROR_INVALID_SOURCE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_ENUM_INVALID,
    translation::java::ARGUMENT_ENUM_INVALID,
);

const CATEGORIES: [&str; 10] = [
    "master", "music", "record", "weather", "block", "hostile", "neutral", "player", "ambient",
    "voice",
];

#[derive(Clone, Copy)]
pub struct SoundCategoryArgumentType;

impl ArgumentType for SoundCategoryArgumentType {
    type Item = SoundCategory;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let start = reader.cursor();
        let s = reader.read_unquoted_string();
        match s.to_lowercase().as_str() {
            "master" => Ok(SoundCategory::Master),
            "music" => Ok(SoundCategory::Music),
            "record" => Ok(SoundCategory::Records),
            "weather" => Ok(SoundCategory::Weather),
            "block" => Ok(SoundCategory::Blocks),
            "hostile" => Ok(SoundCategory::Hostile),
            "neutral" => Ok(SoundCategory::Neutral),
            "player" => Ok(SoundCategory::Players),
            "ambient" => Ok(SoundCategory::Ambient),
            "voice" => Ok(SoundCategory::Voice),
            _ => {
                reader.set_cursor(start);
                Err(ERROR_INVALID_SOURCE.create(reader, TextComponent::text(s)))
            }
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::String(
            pumpkin_protocol::java::client::play::StringProtoArgBehavior::SingleWord,
        )
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        builder.filter_and_suggest(&CATEGORIES).build()
    }
}

impl SoundCategoryArgumentType {
    pub fn get(context: &CommandContext, name: &str) -> Result<SoundCategory, CommandSyntaxError> {
        context.get_argument::<SoundCategory>(name).copied()
    }
}
