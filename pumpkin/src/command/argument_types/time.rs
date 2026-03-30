use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

pub const INVALID_UNIT_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_TIME_INVALID_UNIT);
pub const TICK_COUNT_TOO_LOW_ERROR_TYPE: CommandErrorType<2> =
    CommandErrorType::new(translation::ARGUMENT_TIME_TICK_COUNT_TOO_LOW);

/// Represents an argument type parsing a time value, which can
/// have one of these suffixes:
/// - `t` or *no suffix*: ticks
/// - `s`: real-life seconds (20 ticks)
/// - `d`: Minecraft days (24,000 ticks)
///
/// The provided `i32` by this argument type represents the value in *ticks*.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TimeArgumentType {
    /// The minimum amount of time, in ticks, that this type accepts.
    pub min: i32,
}

impl TimeArgumentType {
    /// Returns a [`TimeArgumentType`] that accepts any value of time.
    #[must_use]
    pub const fn any() -> Self {
        Self::new(0)
    }

    /// Returns a [`TimeArgumentType`] that accepts any value of time lasting for at least `min` ticks.
    #[must_use]
    pub const fn new(min: i32) -> Self {
        Self { min }
    }
}

impl ArgumentType for TimeArgumentType {
    type Item = i32;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let value = reader.read_float()?;
        let unit = reader.read_unquoted_string()?;
        // Find our unit's translation to ticks.
        let ticks_per_unit = match unit.as_str() {
            "t" | "" => 1,
            "s" => 20,
            "d" => 24000,
            _ => return Err(INVALID_UNIT_ERROR_TYPE.create(reader)),
        };
        let ticks = (ticks_per_unit as f32 * value).round() as i32;
        if ticks < self.min {
            Err(TICK_COUNT_TOO_LOW_ERROR_TYPE.create(
                reader,
                TextComponent::text(self.min.to_string()),
                TextComponent::text(ticks.to_string()),
            ))
        } else {
            Ok(ticks)
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::Time { min: self.min }
    }

    fn examples(&self) -> Vec<String> {
        examples!("12d", "14s", "789", "450t")
    }
}

#[cfg(test)]
mod test {
    use crate::command::{
        argument_types::{argument_type::ArgumentType, time, time::TimeArgumentType},
        string_reader::StringReader,
    };

    #[test]
    fn parse_ticks() {
        // No unit
        let mut reader = StringReader::new("15");

        assert_parse_ok_reset!(&mut reader, TimeArgumentType::any(), 15);
        assert_parse_ok_reset!(&mut reader, TimeArgumentType::new(10), 15);
        assert_parse_err_reset!(
            &mut reader,
            TimeArgumentType::new(20),
            &time::TICK_COUNT_TOO_LOW_ERROR_TYPE
        );

        // Unit
        let mut reader = StringReader::new("95t");

        assert_parse_ok_reset!(&mut reader, TimeArgumentType::any(), 95);
        assert_parse_ok_reset!(&mut reader, TimeArgumentType::new(50), 95);
        assert_parse_err_reset!(
            &mut reader,
            TimeArgumentType::new(150),
            &time::TICK_COUNT_TOO_LOW_ERROR_TYPE
        );
    }

    #[test]
    fn parse_other_units() {
        // Seconds
        let mut reader = StringReader::new("6s");
        assert_parse_ok_reset!(&mut reader, TimeArgumentType::any(), 120);

        // The argument type rounds decimals after multiplication
        // to the nearest integer.
        let mut reader = StringReader::new("4.5s");
        assert_parse_ok_reset!(&mut reader, TimeArgumentType::any(), 90);
        let mut reader = StringReader::new("4.633s");
        assert_parse_ok_reset!(&mut reader, TimeArgumentType::any(), 93);

        let mut reader = StringReader::new("-0.2s");
        assert_parse_err_reset!(
            &mut reader,
            TimeArgumentType::any(),
            &time::TICK_COUNT_TOO_LOW_ERROR_TYPE
        );

        // Minecraft days
        let mut reader = StringReader::new("9d");
        assert_parse_ok_reset!(&mut reader, TimeArgumentType::any(), 24000 * 9);
        let mut reader = StringReader::new("7.1234d");
        assert_parse_ok_reset!(&mut reader, TimeArgumentType::any(), 170962);

        // Invalid units
        let mut reader = StringReader::new("14m");
        assert_parse_err_reset!(
            &mut reader,
            TimeArgumentType::any(),
            &time::INVALID_UNIT_ERROR_TYPE
        );
        let mut reader = StringReader::new("1w");
        assert_parse_err_reset!(
            &mut reader,
            TimeArgumentType::any(),
            &time::INVALID_UNIT_ERROR_TYPE
        );
    }
}
