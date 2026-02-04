use crate::command::{
    argument_types::argument_type::ArgumentType, errors::command_syntax_error::CommandSyntaxError,
    string_reader::StringReader,
};

pub enum StringArgumentType {
    /// Accepts a single unquoted word.
    SingleWord,

    /// Accepts a quoted or unquoted string.
    QuotablePhrase,

    /// Takes the remaining text from the [`StringReader`] and returns that.
    GreedyPhrase,
}

impl ArgumentType<String> for StringArgumentType {
    fn parse(&self, reader: &mut StringReader) -> Result<String, CommandSyntaxError> {
        match self {
            Self::SingleWord => reader.read_unquoted_string(),
            Self::QuotablePhrase => reader.read_string(),
            Self::GreedyPhrase => {
                let text = reader.remaining_part().to_owned();
                reader.set_cursor(reader.total_length());
                Ok(text)
            }
        }
    }

    fn examples(&self) -> &'static [&'static str] {
        match self {
            Self::SingleWord => &["word", "words_with_underscores"],
            Self::QuotablePhrase => &["\"quoted phrase\"", "word", "\"\""],
            Self::GreedyPhrase => &["word", "words with spaces", "\"and symbols\""],
        }
    }
}
