use crate::command::{
    argument_types::argument_type::{ArgumentType, JavaClientArgumentType},
    errors::command_syntax_error::CommandSyntaxError,
    string_reader::StringReader,
};

pub struct ResourceLocationArgumentType;

impl ArgumentType for ResourceLocationArgumentType {
    type Item = String;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        reader.read_unquoted_string()
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::ResourceLocation
    }

    fn examples(&self) -> Vec<String> {
        examples!(
            "minecraft:stone",
            "pumpkin:test/path",
            "namespace:identifier"
        )
    }
}
