use crate::command::{
    argument_types::{argument_type::{ArgumentType, JavaClientArgumentType}, core::within_or_err},
    errors::{command_syntax_error::CommandSyntaxError, error_types},
    string_reader::StringReader,
};

/// Represents an argument type parsing an [`i64`].
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LongArgumentType {
    pub min: i64,
    pub max: i64,
}

impl ArgumentType for LongArgumentType {
    type Item = i64;

    fn parse(&self, reader: &mut StringReader) -> Result<i64, CommandSyntaxError> {
        let reader_start = reader.cursor();
        let result = reader.read_long()?;
        within_or_err(
            reader,
            reader_start,
            result,
            self.min,
            self.max,
            &error_types::LONG_TOO_LOW,
            &error_types::LONG_TOO_HIGH,
        )
    }

    fn examples(&self) -> Vec<String> {
        examples!("0", "123", "-123")
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        JavaClientArgumentType::Long {
            min: (self.min != i64::MIN).then_some(self.min),
            max: (self.min != i64::MAX).then_some(self.max)
        }
    }
}

impl LongArgumentType {
    /// Constructs a new [`LongArgumentType`] with no minimum or maximum bounds.
    #[must_use]
    pub const fn any() -> Self {
        Self {
            min: i64::MIN,
            max: i64::MAX,
        }
    }

    /// Constructs a new [`LongArgumentType`] with *only* the specified minimum bound.
    #[must_use]
    pub const fn with_min(min: i64) -> Self {
        Self { min, max: i64::MAX }
    }

    /// Constructs a new [`LongArgumentType`] with *only* the specified maximum bound.
    #[must_use]
    pub const fn with_max(max: i64) -> Self {
        Self { min: i64::MIN, max }
    }

    /// Constructs a new [`LongArgumentType`] with the given bounds.
    #[must_use]
    pub const fn new(min: i64, max: i64) -> Self {
        Self { min, max }
    }
}

#[cfg(test)]
mod test {
    use crate::command::{
        argument_types::{argument_type::ArgumentType, core::long::LongArgumentType},
        errors::error_types,
        string_reader::StringReader,
    };

    #[test]
    fn parse_test() {
        let mut reader = StringReader::new("123");

        assert_parse_ok_reset!(&mut reader, LongArgumentType::any(), 123);

        assert_parse_ok_reset!(&mut reader, LongArgumentType::with_min(120), 123);
        assert_parse_err_reset!(
            &mut reader,
            LongArgumentType::with_min(130),
            &error_types::LONG_TOO_LOW
        );

        assert_parse_ok_reset!(&mut reader, LongArgumentType::with_max(200), 123);
        assert_parse_err_reset!(
            &mut reader,
            LongArgumentType::with_max(100),
            &error_types::LONG_TOO_HIGH
        );

        assert_parse_ok_reset!(&mut reader, LongArgumentType::new(100, 125), 123);
        assert_parse_err_reset!(
            &mut reader,
            LongArgumentType::new(100, 120),
            &error_types::LONG_TOO_HIGH
        );
        assert_parse_err_reset!(
            &mut reader,
            LongArgumentType::new(125, 150),
            &error_types::LONG_TOO_LOW
        );

        // 5_000_000_000_000_000_000 fits into an i64.
        reader = StringReader::new("5000000000000000000");
        assert_parse_ok_reset!(
            &mut reader,
            LongArgumentType::any(),
            5_000_000_000_000_000_000
        );

        // 10_000_000_000_000_000_000 does not fit into an i64.
        reader = StringReader::new("10000000000000000000");
        assert_parse_err_reset!(
            &mut reader,
            LongArgumentType::any(),
            &error_types::READER_INVALID_LONG
        );
    }
}
