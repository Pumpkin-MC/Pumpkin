use pumpkin_nbt::tag::NbtTag;

use crate::command::{
    argument_types::argument_type::{ArgumentType, JavaClientArgumentType},
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    snbt::SnbtParser,
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};

/// Parses any type of NBT tag from SNBT.
pub struct NbtTagArgumentType;

impl ArgumentType for NbtTagArgumentType {
    type Item = NbtTag;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        SnbtParser::parse_for_commands(reader)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::NbtTag
    }

    fn list_suggestions<'a>(
        &'a self,
        _context: &'a CommandContext,
        builder: SuggestionsBuilder,
    ) -> std::pin::Pin<Box<dyn Future<Output = Suggestions> + Send + 'a>> {
        Box::pin(async move { SnbtParser::parse_for_suggestions(builder) })
    }

    fn examples(&self) -> Vec<String> {
        examples!(
            "5", "7b", "1.6", "\"hi\"", "'bye'", "[2, 3]", "[L; 4]", "{x: 3}"
        )
    }
}
