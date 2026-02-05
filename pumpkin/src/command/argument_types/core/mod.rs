use pumpkin_util::text::TextComponent;

use crate::command::{
    errors::{command_syntax_error::CommandSyntaxError, error_types::CommandErrorType},
    string_reader::StringReader,
};

pub mod bool;
pub mod double;
pub mod float;
pub mod integer;
pub mod long;
pub mod string;

/// Helper method for parsing with a reader and returning an [`Err`] if outside the range.
#[inline]
pub fn within_or_err<T>(
    reader: &mut StringReader,
    reader_start: usize,
    value: T,
    min: T,
    max: T,
    too_low_error_type: &'static CommandErrorType<2>,
    too_high_error_type: &'static CommandErrorType<2>,
) -> Result<T, CommandSyntaxError>
where
    T: PartialOrd + ToString + Copy,
{
    if value < min {
        reader.set_cursor(reader_start);
        Err(too_low_error_type.create(
            reader,
            TextComponent::text(value.to_string()),
            TextComponent::text(min.to_string()),
        ))
    } else if value > max {
        reader.set_cursor(reader_start);
        Err(too_high_error_type.create(
            reader,
            TextComponent::text(value.to_string()),
            TextComponent::text(max.to_string()),
        ))
    } else {
        Ok(value)
    }
}
