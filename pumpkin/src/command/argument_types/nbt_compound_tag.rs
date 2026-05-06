use pumpkin_data::translation::java::ARGUMENT_NBT_EXPECTED_COMPOUND;
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};

use crate::command::{
    argument_types::argument_type::{ArgumentType, JavaClientArgumentType},
    context::command_context::CommandContext,
    errors::{command_syntax_error::CommandSyntaxError, error_types::CommandErrorType},
    snbt::SnbtParser,
    string_reader::StringReader,
};

pub const EXPECTED_COMPOUND_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    ARGUMENT_NBT_EXPECTED_COMPOUND,
    ARGUMENT_NBT_EXPECTED_COMPOUND,
);

/// Parses **only** compound NBT tags from SNBT.
pub struct NbtCompoundTagArgumentType;

impl ArgumentType for NbtCompoundTagArgumentType {
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

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::NbtCompoundTag
    }

    fn examples(&self) -> Vec<String> {
        examples!("{}", "{x: 3}")
    }
}

impl NbtCompoundTagArgumentType {
    /// Returns the parsed [`NbtCompound`] from the name of the argument.
    pub fn get<'a>(
        context: &'a CommandContext,
        name: &'_ str,
    ) -> Result<&'a NbtCompound, CommandSyntaxError> {
        context.get_argument(name)
    }
}
