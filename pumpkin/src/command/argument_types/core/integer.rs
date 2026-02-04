use crate::command::{
    argument_types::{argument_type::ArgumentType, core::within_or_err},
    errors::{command_syntax_error::CommandSyntaxError, error_types},
    string_reader::StringReader,
};

/// Represents an argument type parsing an [`i32`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct IntegerArgumentType {
    pub min: i32,
    pub max: i32,
}

impl ArgumentType<i32> for IntegerArgumentType {
    fn parse(&self, reader: &mut StringReader) -> Result<i32, CommandSyntaxError> {
        let reader_start = reader.cursor();
        let result = reader.read_int()?;
        within_or_err(
            reader,
            reader_start,
            result,
            self.min,
            self.max,
            error_types::INTEGER_TOO_LOW,
            error_types::INTEGER_TOO_HIGH,
        )
    }

    fn examples(&self) -> &'static [&'static str] {
        &["0", "123", "-123"]
    }
}

impl IntegerArgumentType {
    /// Constructs a new [`IntegerArgumentType`] with no minimum or maximum bounds.
    #[must_use]
    pub const fn any() -> Self {
        Self {
            min: i32::MIN,
            max: i32::MAX,
        }
    }

    /// Constructs a new [`IntegerArgumentType`] with *only* the specified minimum bound.
    #[must_use]
    pub const fn with_min(min: i32) -> Self {
        Self { min, max: i32::MAX }
    }

    /// Constructs a new [`IntegerArgumentType`] with *only* the specified maximum bound.
    #[must_use]
    pub const fn with_max(max: i32) -> Self {
        Self { min: i32::MIN, max }
    }

    /// Constructs a new [`IntegerArgumentType`] with the given bounds.
    #[must_use]
    pub const fn new(min: i32, max: i32) -> Self {
        Self { min, max }
    }
}

#[cfg(test)]
mod test {
    use crate::command::{
        argument_types::{argument_type::ArgumentType, core::integer::IntegerArgumentType},
        string_reader::StringReader,
    };

    #[test]
    fn parse_test() {
        let mut reader = StringReader::new("123");

        assert_parse_ok_reset!(IntegerArgumentType::any(), reader, 123);

        assert_parse_ok_reset!(IntegerArgumentType::with_min(120), reader, 123);
        assert_parse_err_reset!(IntegerArgumentType::with_min(130), reader);

        assert_parse_ok_reset!(IntegerArgumentType::with_max(200), reader, 123);
        assert_parse_err_reset!(IntegerArgumentType::with_max(100), reader);

        assert_parse_ok_reset!(IntegerArgumentType::new(100, 125), reader, 123);
        assert_parse_err_reset!(IntegerArgumentType::new(100, 120), reader);
        assert_parse_err_reset!(IntegerArgumentType::new(125, 150), reader);

        // 500_000_000 fits into an i32.
        reader = StringReader::new("500000000");
        assert_parse_ok_reset!(IntegerArgumentType::any(), reader, 500_000_000);

        // 5_000_000_000 does not fit into an i32.
        reader = StringReader::new("5000000000");
        assert_parse_err_reset!(IntegerArgumentType::any(), reader);
    }
}
