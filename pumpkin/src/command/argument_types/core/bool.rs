use crate::command::{
    argument_types::argument_type::ArgumentType, errors::command_syntax_error::CommandSyntaxError,
    string_reader::StringReader,
};

/// Represents an argument type parsing a [`bool`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BoolArgumentType;

impl ArgumentType<bool> for BoolArgumentType {
    fn parse(&self, reader: &mut StringReader) -> Result<bool, CommandSyntaxError> {
        reader.read_bool()
    }

    fn examples(&self) -> &'static [&'static str] {
        &["true", "false"]
    }
}

#[cfg(test)]
mod test {
    use crate::command::{
        argument_types::{argument_type::ArgumentType, core::bool::BoolArgumentType},
        string_reader::StringReader,
    };

    #[test]
    fn parse_test() {
        let mut reader = StringReader::new("true");
        assert_parse_ok_reset!(BoolArgumentType, reader, true);

        reader = StringReader::new("false");
        assert_parse_ok_reset!(BoolArgumentType, reader, false);

        reader = StringReader::new("1");
        assert_parse_err_reset!(BoolArgumentType, reader);
    }
}
