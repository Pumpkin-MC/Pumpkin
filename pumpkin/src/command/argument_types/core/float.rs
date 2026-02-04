use crate::command::{
    argument_types::{argument_type::ArgumentType, core::within_or_err},
    errors::{command_syntax_error::CommandSyntaxError, error_types},
    string_reader::StringReader,
};

/// Represents an argument type parsing an [`f32`].
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FloatArgumentType {
    pub min: f32,
    pub max: f32,
}

impl ArgumentType<f32> for FloatArgumentType {
    fn parse(&self, reader: &mut StringReader) -> Result<f32, CommandSyntaxError> {
        let reader_start = reader.cursor();
        let result = reader.read_float()?;
        within_or_err(
            reader,
            reader_start,
            result,
            self.min,
            self.max,
            error_types::FLOAT_TOO_LOW,
            error_types::FLOAT_TOO_HIGH,
        )
    }

    fn examples(&self) -> &'static [&'static str] {
        &["0", "1.2", ".5", "-1", "-.5", "-1234.56"]
    }
}

impl FloatArgumentType {
    /// Constructs a new [`FloatArgumentType`] with no minimum or maximum bounds.
    #[must_use]
    pub const fn any() -> Self {
        Self {
            min: f32::MIN,
            max: f32::MAX,
        }
    }

    /// Constructs a new [`FloatArgumentType`] with *only* the specified minimum bound.
    #[must_use]
    pub const fn with_min(min: f32) -> Self {
        Self { min, max: f32::MAX }
    }

    /// Constructs a new [`FloatArgumentType`] with *only* the specified maximum bound.
    #[must_use]
    pub const fn with_max(max: f32) -> Self {
        Self { min: f32::MIN, max }
    }

    /// Constructs a new [`FloatArgumentType`] with the given bounds.
    #[must_use]
    pub const fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }
}
