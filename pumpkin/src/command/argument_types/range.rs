use pumpkin_util::math::bounds::{DoubleBounds, IntBounds};

use crate::command::{argument_types::{FromStringReader, argument_type::{ArgumentType, JavaClientArgumentType}}, errors::command_syntax_error::CommandSyntaxError, string_reader::StringReader};

/// Parses an inclusive range of `i32`s that can be represented in the following ways:
/// - `value`: Only includes the integer `value`.
/// - `min..`: All integers above or equal to `min`.
/// - `..max`: All integers below or equal to `max`.
/// - `min..max`: All integers that are between `min` and `max`, inclusive. Mathematically representable as `[min, max]`.
pub struct IntRangeArgumentType;

impl ArgumentType for IntRangeArgumentType {
    type Item = IntBounds;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        IntBounds::from_reader(reader)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::IntRange
    }

    fn examples(&self) -> Vec<String> {
        examples!(
            "8",
            "2..",
            "..-3",
            "-4..5"
        )
    }
}

/// Parses an inclusive range of `f64`s that can be represented in the following ways:
/// - `value`: Only includes the number `value`.
/// - `min..`: All numbers above or equal to `min`.
/// - `..max`: All numbers below or equal to `max`.
/// - `min..max`: All numbers that are between `min` and `max`, inclusive. Mathematically representable as `[min, max]`.
pub struct FloatRangeArgumentType;

impl ArgumentType for FloatRangeArgumentType {
    type Item = DoubleBounds;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        DoubleBounds::from_reader(reader)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::FloatRange
    }

    fn examples(&self) -> Vec<String> {
        examples!(
            "8",
            "8.0",
            "5..",
            "..-3.8",
            "-4.2..5",
            "0.1..0.2"
        )
    }
}

#[cfg(test)]
mod test {
    use pumpkin_util::math::bounds::{DoubleBounds, IntBounds};

    use crate::command::{argument_types::{SWAPPED_BOUNDS_ERROR_TYPE, argument_type::ArgumentType, range::{FloatRangeArgumentType, IntRangeArgumentType}}, string_reader::StringReader};

    #[test]
    fn parse_int_ranges() {
        let mut reader = StringReader::new("-7");
        assert_parse_ok_reset!(&mut reader, IntRangeArgumentType, IntBounds::new(-7, -7));

        let mut reader = StringReader::new("3..");
        assert_parse_ok_reset!(&mut reader, IntRangeArgumentType, IntBounds::new_at_least(3));

        let mut reader = StringReader::new("..4");
        assert_parse_ok_reset!(&mut reader, IntRangeArgumentType, IntBounds::new_at_most(4));

        let mut reader = StringReader::new("3..4");
        assert_parse_ok_reset!(&mut reader, IntRangeArgumentType, IntBounds::new(3, 4));

        let mut reader = StringReader::new("-0..0");
        assert_parse_ok_reset!(&mut reader, IntRangeArgumentType, IntBounds::new(0, 0));

        let mut reader = StringReader::new("2..1");
        assert_parse_err_reset!(&mut reader, IntRangeArgumentType, &SWAPPED_BOUNDS_ERROR_TYPE);
    }

    #[test]
    fn parse_float_ranges() {
        let mut reader = StringReader::new("-1");
        assert_parse_ok_reset!(&mut reader, FloatRangeArgumentType, DoubleBounds::new(-1.0, -1.0));

        let mut reader = StringReader::new("-1.0");
        assert_parse_ok_reset!(&mut reader, FloatRangeArgumentType, DoubleBounds::new(-1.0, -1.0));

        let mut reader = StringReader::new("0.");
        assert_parse_ok_reset!(&mut reader, FloatRangeArgumentType, DoubleBounds::new(0.0, 0.0));

        let mut reader = StringReader::new("3..");
        assert_parse_ok_reset!(&mut reader, FloatRangeArgumentType, DoubleBounds::new_at_least(3.0));

        let mut reader = StringReader::new("..4");
        assert_parse_ok_reset!(&mut reader, FloatRangeArgumentType, DoubleBounds::new_at_most(4.0));

        let mut reader = StringReader::new("3..4");
        assert_parse_ok_reset!(&mut reader, FloatRangeArgumentType, DoubleBounds::new(3.0, 4.0));

        let mut reader = StringReader::new("0.9");
        assert_parse_ok_reset!(&mut reader, FloatRangeArgumentType, DoubleBounds::new(0.9, 0.9));

        let mut reader = StringReader::new("0..9");
        assert_parse_ok_reset!(&mut reader, FloatRangeArgumentType, DoubleBounds::new(0.0, 9.0));

        let mut reader = StringReader::new("0...9");
        assert_parse_ok_reset!(&mut reader, FloatRangeArgumentType, DoubleBounds::new(0.0, 0.9));

        let mut reader = StringReader::new("2..1");
        assert_parse_err_reset!(&mut reader, FloatRangeArgumentType, &SWAPPED_BOUNDS_ERROR_TYPE);
    }
}