use pumpkin_data::translation::ARGUMENT_ID_INVALID;
use pumpkin_util::identifier::Identifier;

use crate::command::{
    argument_types::argument_type::ArgumentType,
    errors::{command_syntax_error::CommandSyntaxError, error_types::CommandErrorType},
    string_reader::StringReader,
};

pub const COMMAND_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(ARGUMENT_ID_INVALID);

pub struct IdentifierArgumentType;

impl ArgumentType for IdentifierArgumentType {
    type Item = Identifier;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let start = reader.cursor();
        while let Some(c) = reader.peek()
            && Identifier::is_valid_char(c)
        {
            reader.skip();
        }
        let end = reader.cursor();

        let slice = &reader.string()[start..end];
        Identifier::parse(slice).map_err(|_| COMMAND_ERROR_TYPE.create(reader))
    }

    fn examples(&self) -> Vec<String> {
        examples!("foo", "foo:bar", "123", "123-45:67/8_9")
    }
}
