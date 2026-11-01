use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::{
    CommandSource, context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError, string_reader::StringReader,
};
use pumpkin_command::argument_types::FromStringReader;
use pumpkin_util::identifier::Identifier;

/// Parses a function name: `name`, `namespace:name` or a tag such as `#tag`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FunctionArgumentType;

impl ArgumentType<CommandSource> for FunctionArgumentType {
    type Item = String;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        if reader.peek() == Some('#') {
            reader.skip();
            Ok(format!("#{}", Identifier::from_reader(reader)?))
        } else {
            Ok(Identifier::from_reader(reader)?.to_string())
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::translation;

    #[test]
    fn normalizes_function_names_and_tags() {
        for (input, expected) in [
            ("ping", "minecraft:ping"),
            ("#mcgrp", "#minecraft:mcgrp"),
            (":ping", "minecraft:ping"),
            ("#:mcgrp", "#minecraft:mcgrp"),
            ("nores:tick", "nores:tick"),
            ("#minecraft:tick", "#minecraft:tick"),
            ("nores:folder/tick", "nores:folder/tick"),
        ] {
            let mut reader = StringReader::new(input);
            assert_eq!(FunctionArgumentType.parse(&mut reader).unwrap(), expected);
            assert_eq!(reader.remaining_length(), 0);
        }
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
    fn rejects_invalid_identifiers_with_the_standard_error() {
        for input in [
            "bad/namespace:ping",
            "minecraft:bad:ping",
            "#bad/namespace:ping",
            "#minecraft:bad:ping",
        ] {
            let mut reader = StringReader::new(input);
            let error = FunctionArgumentType.parse(&mut reader).unwrap_err();
            assert_eq!(
                serde_json::to_value(error.message).unwrap()["translate"],
                translation::java::ARGUMENT_ID_INVALID
            );
            assert_eq!(reader.cursor(), usize::from(input.starts_with('#')));
        }
    }
}
