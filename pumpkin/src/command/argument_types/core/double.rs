use crate::command::{
    argument_types::{argument_type::ArgumentType, core::within_or_err},
    errors::{command_syntax_error::CommandSyntaxError, error_types},
    string_reader::StringReader,
};

/// Represents an argument type parsing an [`f64`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DoubleArgumentType {
    pub min: f64,
    pub max: f64,
}

impl ArgumentType<f64> for DoubleArgumentType {
    fn parse(&self, reader: &mut StringReader) -> Result<f64, CommandSyntaxError> {
        let reader_start = reader.cursor();
        let result = reader.read_double()?;
        within_or_err(
            reader,
            reader_start,
            result,
            self.min,
            self.max,
            error_types::DOUBLE_TOO_LOW,
            error_types::DOUBLE_TOO_HIGH,
        )
    }

    fn examples(&self) -> &'static [&'static str] {
        &["0", "1.2", ".5", "-1", "-.5", "-1234.56"]
    }
}

impl DoubleArgumentType {
    /// Constructs a new [`DoubleArgumentType`] with no minimum or maximum bounds.
    #[must_use]
    pub const fn any() -> Self {
        Self {
            min: f64::MIN,
            max: f64::MAX,
        }
    }

    /// Constructs a new [`DoubleArgumentType`] with *only* the specified minimum bound.
    #[must_use]
    pub const fn with_min(min: f64) -> Self {
        Self { min, max: f64::MAX }
    }

    /// Constructs a new [`DoubleArgumentType`] with *only* the specified maximum bound.
    #[must_use]
    pub const fn with_max(max: f64) -> Self {
        Self { min: f64::MIN, max }
    }

    /// Constructs a new [`DoubleArgumentType`] with the given bounds.
    #[must_use]
    pub const fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }
}
