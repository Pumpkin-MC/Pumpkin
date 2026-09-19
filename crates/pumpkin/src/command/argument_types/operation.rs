use crate::command::{
    CommandSource,
    argument_types::argument_type::{ArgumentType, JavaClientArgumentType},
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};

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

/// The scoreboard operators, in the order vanilla lists them.
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
    /// The operators sorted so that two character operators are matched before
    /// their one character variants.
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

    #[must_use]
    pub const fn symbol(self) -> &'static str {
        match self {
            Self::Assign => "=",
            Self::Plus => "+=",
            Self::Minus => "-=",
            Self::Multiply => "*=",
            Self::Divide => "/=",
            Self::Modulo => "%=",
            Self::Swap => "><",
            Self::Min => "<",
            Self::Max => ">",
        }
    }

    /// Applies the operator to the target value using the source value. [`Self::Swap`]
    /// changes both scores and is handled by the command instead.
    #[must_use]
    pub fn apply(self, target: i32, source: i32) -> i32 {
        match self {
            Self::Assign | Self::Swap => source,
            Self::Plus => target.wrapping_add(source),
            Self::Minus => target.wrapping_sub(source),
            Self::Multiply => target.wrapping_mul(source),
            // Vanilla uses floor division and modulo for negative operands and
            // leaves the target unchanged when the divisor is zero.
            Self::Divide => {
                if source == 0 {
                    target
                } else {
                    floor_div(target, source)
                }
            }
            Self::Modulo => {
                if source == 0 {
                    target
                } else {
                    floor_mod(target, source)
                }
            }
            Self::Min => target.min(source),
            Self::Max => target.max(source),
        }
    }
}

/// Parses a scoreboard operation such as `=`, `+=` or `><`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct OperationArgumentType;

impl ArgumentType<CommandSource> for OperationArgumentType {
    type Item = ScoreboardOperation;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let remaining = reader.remaining_part();
        for (operation, symbol) in ScoreboardOperation::ALL {
            if remaining.starts_with(symbol) {
                reader.set_cursor(reader.cursor() + symbol.len());
                return Ok(operation);
            }
        }
        Err(INVALID_OPERATION_ERROR.create(reader))
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
            builder = builder.suggest(symbol);
        }
        builder.build()
    }

    fn examples(&self) -> Vec<String> {
        examples!("=", "+=", "><")
    }
}

const INVALID_OPERATION_ERROR: crate::command::errors::error_types::CommandErrorType<0> =
    crate::command::errors::error_types::CommandErrorType::new(
        "commands.scoreboard.players.operation.invalidOperation",
        "commands.scoreboard.players.operation.invalidOperation",
    );

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_operator_applies_its_math() {
        assert_eq!(ScoreboardOperation::Assign.apply(7, 3), 3);
        assert_eq!(ScoreboardOperation::Plus.apply(7, 3), 10);
        assert_eq!(ScoreboardOperation::Minus.apply(7, 3), 4);
        assert_eq!(ScoreboardOperation::Multiply.apply(7, 3), 21);
        assert_eq!(ScoreboardOperation::Divide.apply(7, 2), 3);
        assert_eq!(ScoreboardOperation::Divide.apply(-5, 3), -2);
        assert_eq!(ScoreboardOperation::Divide.apply(7, 0), 7);
        assert_eq!(ScoreboardOperation::Modulo.apply(7, 2), 1);
        assert_eq!(ScoreboardOperation::Modulo.apply(-5, 4), 3);
        assert_eq!(ScoreboardOperation::Modulo.apply(7, 0), 7);
        assert_eq!(ScoreboardOperation::Swap.apply(7, 3), 3);
        assert_eq!(ScoreboardOperation::Min.apply(7, 3), 3);
        assert_eq!(ScoreboardOperation::Max.apply(7, 3), 7);
    }

    #[test]
    fn symbols_match_the_operations() {
        for (operation, symbol) in ScoreboardOperation::ALL {
            assert_eq!(operation.symbol(), symbol);
        }
    }

    #[test]
    fn two_character_operators_win_over_their_prefixes() {
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
            let mut reader = StringReader::new(text);
            assert_eq!(OperationArgumentType.parse(&mut reader).unwrap(), expected);
            assert_eq!(reader.remaining_length(), 0, "did not consume {text}");
        }
    }

    #[test]
    fn extreme_operands_do_not_panic() {
        assert_eq!(ScoreboardOperation::Divide.apply(i32::MIN, -1), i32::MIN);
        assert_eq!(ScoreboardOperation::Modulo.apply(i32::MIN, -1), 0);
        assert_eq!(ScoreboardOperation::Divide.apply(i32::MAX, 1), i32::MAX);
    }

    #[test]
    fn rejects_unknown_operators() {
        let mut reader = StringReader::new("+");
        assert!(OperationArgumentType.parse(&mut reader).is_err());
    }
}
