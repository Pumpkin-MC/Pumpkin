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

pub mod argument_type;
pub mod core;
