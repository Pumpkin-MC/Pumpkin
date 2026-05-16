use pumpkin_data::translation::java::ARGUMENT_UUID_INVALID;
use pumpkin_util::uuid::parse_uuid;
use uuid::Uuid;

use crate::command::{
    argument_types::argument_type::{ArgumentType, JavaClientArgumentType},
    errors::{command_syntax_error::CommandSyntaxError, error_types::CommandErrorType},
    string_reader::StringReader,
};

pub const INVALID_UUID_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(ARGUMENT_UUID_INVALID, ARGUMENT_UUID_INVALID);

/// Represents an argument type parsing a [`Uuid`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct UuidArgumentType;

impl ArgumentType for UuidArgumentType {
    type Item = Uuid;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let start = reader.cursor();

        while matches!(reader.peek(), Some('A'..='F' | 'a'..='f' | '0'..='9' | '-')) {
            reader.skip();
        }

        parse_uuid(&reader.string()[start..reader.cursor()]).map_or_else(
            || {
                reader.set_cursor(start);
                Err(INVALID_UUID_ERROR_TYPE.create(reader))
            },
            Ok,
        )
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::Uuid
    }

    fn examples(&self) -> Vec<String> {
        examples!("3d569d3a-93ef-44a0-9f1c-f69db9d37a56")
    }
}

impl_copy_get!(UuidArgumentType, Uuid);
