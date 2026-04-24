use pumpkin_util::identifier::Identifier;

use crate::command::{
    argument_types::{
        FromStringReader,
        argument_type::{ArgumentType, JavaClientArgumentType},
    },
    errors::command_syntax_error::CommandSyntaxError,
    string_reader::StringReader,
};

/// An argument type that parses a generic [`Identifier`] with a namespace and path.
pub struct IdentifierArgumentType;

impl ArgumentType for IdentifierArgumentType {
    type Item = Identifier;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        Identifier::from_reader(reader)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::ResourceLocation
    }

    fn examples(&self) -> Vec<String> {
        examples!("foo", "foo:bar")
    }
}
