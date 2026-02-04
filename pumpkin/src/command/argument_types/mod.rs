// Helper methods for assertion with a `StringReader`:

/// Asserts that the result read by reader `$reader` with the argument
/// type `$argument_type` used to parse is equal to `Ok($value)`.
/// Also resets the reader's cursor back to the start.
#[cfg(test)]
macro_rules! assert_parse_ok_reset {
    ($argument_type: expr, $reader: ident, $value: expr) => {
        assert_eq!($argument_type.parse(&mut $reader), Ok($value));
        $reader.set_cursor(0)
    };
}

/// Asserts that the result read by reader `$reader` with the argument
/// type `$argument_type` used to parse is an `Err`.
/// Also resets the reader's cursor back to the start.
#[cfg(test)]
macro_rules! assert_parse_err_reset {
    ($argument_type: expr, $reader: ident) => {
        assert!($argument_type.parse(&mut $reader).is_err());
        $reader.set_cursor(0)
    };
}

pub mod argument_type;
pub mod core;
