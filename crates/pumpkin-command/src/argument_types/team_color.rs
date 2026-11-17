use crate::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::errors::error_types::CommandErrorType;
use crate::string_reader::StringReader;
use crate::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_data::translation::java::ARGUMENT_COLOR_INVALID;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

pub const INVALID_VALUE_ERROR_TYPE: CommandErrorType<1> =
    CommandErrorType::new(ARGUMENT_COLOR_INVALID, ARGUMENT_COLOR_INVALID);

pub struct TeamColorArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for TeamColorArgumentType {
    type Item = NamedColor;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let id = reader.read_unquoted_string();

        (&*id).try_into().map_or_else(
            |()| Err(INVALID_VALUE_ERROR_TYPE.create(reader, TextComponent::text(id))),
            Ok,
        )
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::Color
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        builder
            .filter_and_suggest(&[
                "black",
                "dark_blue",
                "dark_green",
                "dark_aqua",
                "dark_red",
                "dark_purple",
                "gold",
                "gray",
                "dark_gray",
                "blue",
                "green",
                "aqua",
                "red",
                "light_purple",
                "yellow",
                "white",
            ])
            .build()
    }

    fn examples(&self) -> Vec<String> {
        examples!("red", "blue", "yellow")
    }
}

impl_copy_get!(TeamColorArgumentType, NamedColor);
