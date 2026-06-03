use pumpkin_nbt::tag::NbtTag;

use crate::command::{
    argument_types::argument_type::{ArgumentType, JavaClientArgumentType},
    context::command_context::CommandContext,
    errors::{command_syntax_error::CommandSyntaxError, error_types::CommandErrorType},
    snbt::SnbtParser,
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};

use pumpkin_data::translation::java::ARGUMENT_NBT_EXPECTED_COMPOUND;
use pumpkin_nbt::compound::NbtCompound;

/// Parses any type of NBT tag from SNBT.
pub struct NbtTagArgumentType;

impl ArgumentType for NbtTagArgumentType {
    type Item = NbtTag;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        SnbtParser::parse_for_commands(reader)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
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

impl NbtTagArgumentType {
    /// Returns the parsed [`NbtTag`] from the name of the argument.
    pub fn get<'a>(
        context: &'a CommandContext,
        name: &'_ str,
    ) -> Result<&'a NbtTag, CommandSyntaxError> {
        context.get_argument(name)
    }
}

pub const EXPECTED_COMPOUND_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    ARGUMENT_NBT_EXPECTED_COMPOUND,
    ARGUMENT_NBT_EXPECTED_COMPOUND,
);

/// Parses **only** compound NBT tags from SNBT.
pub struct NbtCompoundArgumentType;

impl ArgumentType for NbtCompoundArgumentType {
    type Item = NbtCompound;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        SnbtParser::parse_for_commands(reader).and_then(|tag| {
            if let NbtTag::Compound(compound) = tag {
                Ok(compound)
            } else {
                Err(EXPECTED_COMPOUND_ERROR_TYPE.create(reader))
            }
        })
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::NbtCompound
    }

    fn examples(&self) -> Vec<String> {
        examples!("{}", "{x: 3}")
    }
}

impl NbtCompoundArgumentType {
    /// Returns the parsed [`NbtCompound`] from the name of the argument.
    pub fn get<'a>(
        context: &'a CommandContext,
        name: &'_ str,
    ) -> Result<&'a NbtCompound, CommandSyntaxError> {
        context.get_argument(name)
    }
}
