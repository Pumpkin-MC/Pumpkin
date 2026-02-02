// These are akin to the translatable built-in exceptions in Minecraft.

pub const READER_EXPECTED_START_QUOTE: SimpleCommandErrorType =
    SimpleCommandErrorType::new("parsing.quote.expected.start");
pub const READER_EXPECTED_END_QUOTE: SimpleCommandErrorType =
    SimpleCommandErrorType::new("parsing.quote.expected.end");
pub const READER_INVALID_ESCAPE: DynamicCommandErrorType<1> =
    DynamicCommandErrorType::new("parsing.quote.escape");
pub const READER_INVALID_BOOL: DynamicCommandErrorType<1> =
    DynamicCommandErrorType::new("parsing.bool.invalid");
pub const READER_EXPECTED_BOOL: SimpleCommandErrorType =
    SimpleCommandErrorType::new("parsing.bool.expected");
pub const READER_INVALID_INT: DynamicCommandErrorType<1> =
    DynamicCommandErrorType::new("parsing.int.invalid");
pub const READER_EXPECTED_INT: SimpleCommandErrorType =
    SimpleCommandErrorType::new("parsing.int.expected");
pub const READER_INVALID_LONG: DynamicCommandErrorType<1> =
    DynamicCommandErrorType::new("parsing.long.invalid");
pub const READER_EXPECTED_LONG: SimpleCommandErrorType =
    SimpleCommandErrorType::new("parsing.long.expected");
pub const READER_INVALID_DOUBLE: DynamicCommandErrorType<1> =
    DynamicCommandErrorType::new("parsing.double.invalid");
pub const READER_EXPECTED_DOUBLE: SimpleCommandErrorType =
    SimpleCommandErrorType::new("parsing.double.expected");
pub const READER_INVALID_FLOAT: DynamicCommandErrorType<1> =
    DynamicCommandErrorType::new("parsing.float.invalid");
pub const READER_EXPECTED_FLOAT: SimpleCommandErrorType =
    SimpleCommandErrorType::new("parsing.float.expected");
pub const READER_EXPECTED_SYMBOL: DynamicCommandErrorType<1> =
    DynamicCommandErrorType::new("parsing.expected");

pub const LITERAL_INCORRECT: DynamicCommandErrorType<1> =
    DynamicCommandErrorType::new("argument.literal.incorrect");

pub const DOUBLE_TOO_LOW: DynamicCommandErrorType<2> =
    DynamicCommandErrorType::new("argument.double.low");
pub const DOUBLE_TOO_HIGH: DynamicCommandErrorType<2> =
    DynamicCommandErrorType::new("argument.double.big");
pub const FLOAT_TOO_LOW: DynamicCommandErrorType<2> =
    DynamicCommandErrorType::new("argument.float.low");
pub const FLOAT_TOO_HIGH: DynamicCommandErrorType<2> =
    DynamicCommandErrorType::new("argument.float.big");
pub const INTEGER_TOO_LOW: DynamicCommandErrorType<2> =
    DynamicCommandErrorType::new("argument.integer.low");
pub const INTEGER_TOO_HIGH: DynamicCommandErrorType<2> =
    DynamicCommandErrorType::new("argument.integer.big");
pub const LONG_TOO_LOW: DynamicCommandErrorType<2> =
    DynamicCommandErrorType::new("argument.long.low");
pub const LONG_TOO_HIGH: DynamicCommandErrorType<2> =
    DynamicCommandErrorType::new("argument.long.big");

pub const DISPATCHER_UNKNOWN_COMMAND: SimpleCommandErrorType =
    SimpleCommandErrorType::new("command.unknown.command");
pub const DISPATCHER_UNKNOWN_ARGUMENT: SimpleCommandErrorType =
    SimpleCommandErrorType::new("command.unknown.argument");
pub const DISPATCHER_EXPECTED_ARGUMENT_SEPARATOR: SimpleCommandErrorType =
    SimpleCommandErrorType::new("command.expected.separator");
pub const DISPATCHER_PARSE_EXCEPTION: DynamicCommandErrorType<1> =
    DynamicCommandErrorType::new("command.exception");

use crate::command::errors::command_syntax_error::{CommandSyntaxError, ContextProvider};
use pumpkin_util::text::TextComponent;

/// A command error type that does not require any translation arguments.
#[derive(Debug, Clone, Copy)]
pub struct SimpleCommandErrorType {
    pub translation_key: &'static str,
}

/// A command error that requires no translation arguments.
impl SimpleCommandErrorType {
    /// Creates a non-dynamic simple command error type,
    /// from only a translation string.
    #[must_use]
    pub const fn new(translation_key: &'static str) -> Self {
        Self { translation_key }
    }

    #[must_use]
    pub fn instance_without_context(&self) -> CommandSyntaxError {
        CommandSyntaxError::new_without_context(TextComponent::translate(self.translation_key, []))
    }

    #[must_use]
    pub fn instance_with_context<C>(&self, context_provider: &C) -> CommandSyntaxError
    where
        C: ContextProvider,
    {
        CommandSyntaxError::new_with_context(
            TextComponent::translate(self.translation_key, []),
            context_provider,
        )
    }
}

/// A command error that requires **exactly** [`N`] translation arguments.
#[derive(Debug, Clone, Copy)]
pub struct DynamicCommandErrorType<const N: usize> {
    pub translation_key: &'static str,
}

impl<const N: usize> DynamicCommandErrorType<N> {
    /// Creates a dynamic error type from a given translation string.
    #[must_use]
    pub const fn new(translation_key: &'static str) -> Self {
        Self { translation_key }
    }

    #[must_use]
    pub fn instance_without_context(&self, args: &[TextComponent; N]) -> CommandSyntaxError {
        CommandSyntaxError::new_without_context(TextComponent::translate(
            self.translation_key,
            args,
        ))
    }

    pub fn instance_with_context<C>(
        &self,
        context_provider: &C,
        args: &[TextComponent; N],
    ) -> CommandSyntaxError
    where
        C: ContextProvider,
    {
        CommandSyntaxError::new_with_context(
            TextComponent::translate(self.translation_key, args),
            context_provider,
        )
    }
}
