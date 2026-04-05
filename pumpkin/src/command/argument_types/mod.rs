use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::{
    CommandErrorType, READER_INVALID_DOUBLE, READER_INVALID_FLOAT, READER_INVALID_INT,
};
use crate::command::string_reader::StringReader;
use pumpkin_data::translation;
use pumpkin_util::math::bounds::{Bounds, DoubleBounds, FloatDegreeBounds, IntBounds};
use pumpkin_util::text::TextComponent;
use std::str::FromStr;

/// Creates a [`Vec<String>`] of examples from
/// the given string literals.
macro_rules! examples {
    ( $( $example:literal ),* ) => {
        vec! [
            $( $example.to_string(), )*
        ]
    };
}

// Helper methods for assertion with a `StringReader`:

/// Asserts that the result read by `reader` with the argument
/// type `$argument_type` used to parse is equal to `Ok($value)`.
/// Also resets the reader's cursor back to the start.
#[cfg(test)]
macro_rules! assert_parse_ok_reset {
    ($reader: expr, $argument_type: expr, $value: expr) => {{
        assert_eq!($argument_type.parse(&mut $reader), Ok($value));
        $reader.set_cursor(0)
    }};
    ($reader: expr, $argument_type: expr) => {{
        assert!($argument_type.parse(&mut $reader).is_ok());
        $reader.set_cursor(0)
    }};
}

/// Asserts that the result read by `reader` with the argument
/// type `$argument_type` used to parse is an `Err` containing the type of error as `$error_type`.
/// Also resets the reader's cursor back to the start.
#[cfg(test)]
macro_rules! assert_parse_err_reset {
    ($reader: expr, $argument_type: expr, $error_type: expr) => {
        let error_type_dyn: &'static dyn crate::command::errors::error_types::AnyCommandErrorType =
            $error_type;
        assert_eq!(
            $argument_type.parse(&mut $reader).map_err(|e| e.error_type),
            Err(error_type_dyn)
        );
        $reader.set_cursor(0)
    };
}

const EMPTY_BOUNDS_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_RANGE_EMPTY);
const SWAPPED_BOUNDS_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::ARGUMENT_RANGE_SWAPPED);

/// A trait to try getting a value from a [`StringReader`].
pub trait FromStringReader: Sized {
    /// Tries to parse a value of this type from a [`StringReader`].
    ///
    /// If this fails, a [`CommandSyntaxError`] is returned with the erroring details.
    fn from_reader(reader: &mut StringReader) -> Result<Self, CommandSyntaxError>;
}

fn bounds_from_reader<T: Copy + FromStr + PartialOrd>(
    reader: &mut StringReader,
    error_type: &'static CommandErrorType<1>,
) -> Result<Bounds<T>, CommandSyntaxError> {
    if reader.can_read_char() {
        let i = reader.cursor();
        try_bounds_from_reader(reader, error_type).map_err(|e| {
            // On catching any error, set the reader's cursor position to be at where the range starts.
            if let Some(mut context) = e.context {
                context.cursor = i;
                CommandSyntaxError::create(e.error_type, e.message, &context)
            } else {
                CommandSyntaxError::create_without_context(e.error_type, e.message)
            }
        })
    } else {
        Err(EMPTY_BOUNDS_ERROR_TYPE.create(reader))
    }
}

fn try_bounds_from_reader<T: Copy + FromStr + PartialOrd>(
    reader: &mut StringReader,
    error_type: &'static CommandErrorType<1>,
) -> Result<Bounds<T>, CommandSyntaxError> {
    let min = read_number_from_reader(reader, error_type)?;
    let max = if reader.peek() == Some('.') && reader.peek_with_offset(1) == Some('.') {
        reader.skip();
        reader.skip();
        read_number_from_reader(reader, error_type)?
    } else {
        min
    };
    if min.is_none() && max.is_none() {
        Err(EMPTY_BOUNDS_ERROR_TYPE.create(reader))
    } else {
        Ok(Bounds::<T>::new(min, max))
    }
}

/// Tries to read a number of a type from a reader.
fn read_number_from_reader<T: FromStr>(
    reader: &mut StringReader,
    error_type: &'static CommandErrorType<1>,
) -> Result<Option<T>, CommandSyntaxError> {
    let i = reader.cursor();
    while has_allowed_peaked_character(reader) {
        reader.skip();
    }
    let string = &reader.string()[i..reader.cursor()].to_string();
    if string.is_empty() {
        Ok(None)
    } else {
        string.parse::<T>().map_or_else(
            |_| Err(error_type.create(reader, TextComponent::text(string.clone()))),
            |t| Ok(Some(t)),
        )
    }
}

fn has_allowed_peaked_character(reader: &StringReader) -> bool {
    let c = reader.peek();
    if matches!(c, Some('0'..='9' | '-')) {
        true
    } else {
        c.is_some_and(|c| {
            c == '.' && (!reader.can_read_chars(2) || reader.peek_with_offset(1) != Some('.'))
        })
    }
}

impl FromStringReader for IntBounds {
    fn from_reader(reader: &mut StringReader) -> Result<Self, CommandSyntaxError> {
        let i = reader.cursor();
        let bounds = bounds_from_reader(reader, &READER_INVALID_INT)?;
        if bounds.are_swapped() {
            reader.set_cursor(i);
            Err(SWAPPED_BOUNDS_ERROR_TYPE.create(reader))
        } else {
            Ok(Self::from_bounds(bounds))
        }
    }
}

impl FromStringReader for DoubleBounds {
    fn from_reader(reader: &mut StringReader) -> Result<Self, CommandSyntaxError> {
        let i = reader.cursor();
        let bounds = bounds_from_reader(reader, &READER_INVALID_DOUBLE)?;
        if bounds.are_swapped() {
            reader.set_cursor(i);
            Err(SWAPPED_BOUNDS_ERROR_TYPE.create(reader))
        } else {
            Ok(Self::from_bounds(bounds))
        }
    }
}

impl FromStringReader for FloatDegreeBounds {
    fn from_reader(reader: &mut StringReader) -> Result<Self, CommandSyntaxError> {
        let i = reader.cursor();
        let bounds = bounds_from_reader(reader, &READER_INVALID_FLOAT)?;
        if bounds.are_swapped() {
            reader.set_cursor(i);
            Err(SWAPPED_BOUNDS_ERROR_TYPE.create(reader))
        } else {
            Ok(Self::from_bounds(bounds))
        }
    }
}

pub mod argument_type;
pub mod coordinates;
pub mod core;
pub mod entity;
pub mod entity_selector;
pub mod range;
pub mod time;
