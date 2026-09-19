use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::{
    CommandSource, context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError, string_reader::StringReader,
};
use pumpkin_data::translation;

/// Parses a function name: `name`, `namespace:name` or a tag such as `#tag`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FunctionArgumentType;

impl ArgumentType<CommandSource> for FunctionArgumentType {
    type Item = String;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let length = reader
            .remaining_part()
            .find(char::is_whitespace)
            .unwrap_or_else(|| reader.remaining_part().len());
        if length == 0 {
            return Err(INVALID_FUNCTION_ERROR.create(reader));
        }
        let name = reader.remaining_part()[..length].to_string();
        reader.set_cursor(reader.cursor() + length);
        Ok(name)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::Function
    }

    fn examples(&self) -> Vec<String> {
        examples!("foo", "foo:bar", "#tag")
    }
}

impl FunctionArgumentType {
    /// Returns a [`CommandContext`]'s parsed function name as a string slice.
    pub fn get<'a>(context: &'a CommandContext, name: &str) -> Result<&'a str, CommandSyntaxError> {
        Ok(context.get_argument::<String>(name)?.as_str())
    }
}

const INVALID_FUNCTION_ERROR: crate::command::errors::error_types::CommandErrorType<0> =
    crate::command::errors::error_types::CommandErrorType::new(
        translation::java::ARGUMENTS_FUNCTION_UNKNOWN,
        translation::java::ARGUMENTS_FUNCTION_UNKNOWN,
    );

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_namespaced_names_and_tags() {
        assert_eq!(parse("nores:tick"), "nores:tick".to_string());
        assert_eq!(parse("#minecraft:tick"), "#minecraft:tick".to_string());
    }

    #[test]
    fn stops_at_whitespace() {
        let mut reader = StringReader::new("nores:tick extra");
        assert_eq!(
            FunctionArgumentType.parse(&mut reader).unwrap(),
            "nores:tick".to_string()
        );
        assert_eq!(reader.remaining_part(), " extra");
    }

    #[test]
    fn rejects_an_empty_name() {
        let mut reader = StringReader::new(" nores:tick");
        assert!(FunctionArgumentType.parse(&mut reader).is_err());
    }

    fn parse(text: &str) -> String {
        let mut reader = StringReader::new(text);
        FunctionArgumentType.parse(&mut reader).unwrap()
    }
}
