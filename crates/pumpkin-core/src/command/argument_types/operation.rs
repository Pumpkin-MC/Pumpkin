use crate::command::{
    CommandSource,
    argument_types::argument_type::{ArgumentType, JavaClientArgumentType},
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    errors::error_types::CommandErrorType,
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};
use pumpkin_data::translation;

/// Floor division that wraps on `i32::MIN / -1` instead of panicking.
const fn floor_div(target: i32, source: i32) -> i32 {
    let quotient = target.wrapping_div(source);
    let remainder = target.wrapping_rem(source);
    if (target ^ source) < 0 && remainder != 0 {
        quotient.wrapping_sub(1)
    } else {
        quotient
    }
}

/// Floor modulo that wraps on `i32::MIN % -1` instead of panicking.
const fn floor_mod(target: i32, source: i32) -> i32 {
    let remainder = target.wrapping_rem(source);
    if remainder != 0 && (remainder ^ source) < 0 {
        remainder.wrapping_add(source)
    } else {
        remainder
    }
}

/// Supported scoreboard operations.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ScoreboardOperation {
    Assign,
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Swap,
    Min,
    Max,
}

impl ScoreboardOperation {
    const ALL: [(Self, &'static str); 9] = [
        (Self::Plus, "+="),
        (Self::Minus, "-="),
        (Self::Multiply, "*="),
        (Self::Divide, "/="),
        (Self::Modulo, "%="),
        (Self::Swap, "><"),
        (Self::Assign, "="),
        (Self::Min, "<"),
        (Self::Max, ">"),
    ];

    /// Applies the operator to the target value using the source value. [`Self::Swap`]
    /// changes both scores and is handled by the command instead.
    pub fn apply(self, target: i32, source: i32) -> Result<i32, CommandSyntaxError> {
        Ok(match self {
            Self::Assign | Self::Swap => source,
            Self::Plus => target.wrapping_add(source),
            Self::Minus => target.wrapping_sub(source),
            Self::Multiply => target.wrapping_mul(source),
            // Java uses floor division and modulo and reports a zero divisor as an error.
            Self::Divide => {
                if source == 0 {
                    return Err(DIVIDE_BY_ZERO_ERROR.create_without_context());
                }
                floor_div(target, source)
            }
            Self::Modulo => {
                if source == 0 {
                    return Err(DIVIDE_BY_ZERO_ERROR.create_without_context());
                }
                floor_mod(target, source)
            }
            Self::Min => target.min(source),
            Self::Max => target.max(source),
        })
    }
}

/// Parses a scoreboard operation such as `=`, `+=` or `><`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct OperationArgumentType;

impl ArgumentType<CommandSource> for OperationArgumentType {
    type Item = ScoreboardOperation;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        if !reader.can_read_char() {
            return Err(INVALID_OPERATION_ERROR.create(reader));
        }
        let start = reader.cursor();
        while reader.peek().is_some_and(|c| c != ' ') {
            reader.skip();
        }
        let token = &reader.string()[start..reader.cursor()];
        for (operation, symbol) in ScoreboardOperation::ALL {
            if token == symbol {
                return Ok(operation);
            }
        }
        Err(INVALID_OPERATION_ERROR.create_without_context())
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::Operation
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        let mut builder = builder;
        for (_, symbol) in ScoreboardOperation::ALL {
            builder = builder.filter_and_suggest_one(symbol);
        }
        builder.build()
    }

    fn examples(&self) -> Vec<String> {
        examples!("=", "+=", "><")
    }
}

const INVALID_OPERATION_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENTS_OPERATION_INVALID,
    translation::java::ARGUMENTS_OPERATION_INVALID,
);

const DIVIDE_BY_ZERO_ERROR: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENTS_OPERATION_DIV0,
    translation::java::ARGUMENTS_OPERATION_DIV0,
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_operator_applies_its_math() {
        assert_eq!(ScoreboardOperation::Assign.apply(7, 3), Ok(3));
        assert_eq!(ScoreboardOperation::Plus.apply(7, 3), Ok(10));
        assert_eq!(ScoreboardOperation::Minus.apply(7, 3), Ok(4));
        assert_eq!(ScoreboardOperation::Multiply.apply(7, 3), Ok(21));
        assert_eq!(ScoreboardOperation::Divide.apply(7, 2), Ok(3));
        assert_eq!(ScoreboardOperation::Divide.apply(-5, 3), Ok(-2));
        assert_eq!(ScoreboardOperation::Divide.apply(5, -3), Ok(-2));
        assert_eq!(ScoreboardOperation::Modulo.apply(7, 2), Ok(1));
        assert_eq!(ScoreboardOperation::Modulo.apply(-5, 4), Ok(3));
        assert_eq!(ScoreboardOperation::Modulo.apply(5, -4), Ok(-3));
        assert_eq!(ScoreboardOperation::Swap.apply(7, 3), Ok(3));
        assert_eq!(ScoreboardOperation::Min.apply(7, 3), Ok(3));
        assert_eq!(ScoreboardOperation::Max.apply(7, 3), Ok(7));
    }

    #[test]
    fn parses_complete_operator_tokens() {
        for (expected, text) in [
            (ScoreboardOperation::Plus, "+="),
            (ScoreboardOperation::Minus, "-="),
            (ScoreboardOperation::Multiply, "*="),
            (ScoreboardOperation::Divide, "/="),
            (ScoreboardOperation::Modulo, "%="),
            (ScoreboardOperation::Swap, "><"),
            (ScoreboardOperation::Assign, "="),
            (ScoreboardOperation::Min, "<"),
            (ScoreboardOperation::Max, ">"),
        ] {
            let input = format!("{text} source");
            let mut reader = StringReader::new(&input);
            assert_eq!(OperationArgumentType.parse(&mut reader).unwrap(), expected);
            assert_eq!(reader.remaining_part(), " source", "did not consume {text}");
        }
    }

    #[test]
    fn extreme_operands_do_not_panic() {
        assert_eq!(
            ScoreboardOperation::Divide.apply(i32::MIN, -1),
            Ok(i32::MIN)
        );
        assert_eq!(ScoreboardOperation::Modulo.apply(i32::MIN, -1), Ok(0));
        assert_eq!(ScoreboardOperation::Divide.apply(i32::MAX, 1), Ok(i32::MAX));
    }

    #[test]
    fn rejects_unknown_operators() {
        for token in ["", "+", "<=", ">=", "==", "+=extra", "><>"] {
            let mut reader = StringReader::new(token);
            let error = OperationArgumentType.parse(&mut reader).unwrap_err();
            assert!(error.is(&INVALID_OPERATION_ERROR));
            assert_eq!(error.message.to_pretty_console(), "Invalid operation");
        }
    }

    #[test]
    fn division_and_modulo_by_zero_fail() {
        for operation in [ScoreboardOperation::Divide, ScoreboardOperation::Modulo] {
            let error = operation.apply(7, 0).unwrap_err();
            assert!(error.is(&DIVIDE_BY_ZERO_ERROR));
            assert_eq!(error.message.to_pretty_console(), "Cannot divide by zero");
        }
    }
}
