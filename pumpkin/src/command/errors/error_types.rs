// These are akin to the translatable built-in exceptions in Minecraft.

pub const READER_EXPECTED_START_QUOTE: CommandErrorType<0> =
    CommandErrorType::new("parsing.quote.expected.start");
pub const READER_EXPECTED_END_QUOTE: CommandErrorType<0> =
    CommandErrorType::new("parsing.quote.expected.end");
pub const READER_INVALID_ESCAPE: CommandErrorType<1> =
    CommandErrorType::new("parsing.quote.escape");
pub const READER_INVALID_BOOL: CommandErrorType<1> = CommandErrorType::new("parsing.bool.invalid");
pub const READER_EXPECTED_BOOL: CommandErrorType<0> =
    CommandErrorType::new("parsing.bool.expected");
pub const READER_INVALID_INT: CommandErrorType<1> = CommandErrorType::new("parsing.int.invalid");
pub const READER_EXPECTED_INT: CommandErrorType<0> = CommandErrorType::new("parsing.int.expected");
pub const READER_INVALID_LONG: CommandErrorType<1> = CommandErrorType::new("parsing.long.invalid");
pub const READER_EXPECTED_LONG: CommandErrorType<0> =
    CommandErrorType::new("parsing.long.expected");
pub const READER_INVALID_DOUBLE: CommandErrorType<1> =
    CommandErrorType::new("parsing.double.invalid");
pub const READER_EXPECTED_DOUBLE: CommandErrorType<0> =
    CommandErrorType::new("parsing.double.expected");
pub const READER_INVALID_FLOAT: CommandErrorType<1> =
    CommandErrorType::new("parsing.float.invalid");
pub const READER_EXPECTED_FLOAT: CommandErrorType<0> =
    CommandErrorType::new("parsing.float.expected");
pub const READER_EXPECTED_SYMBOL: CommandErrorType<1> = CommandErrorType::new("parsing.expected");

pub const LITERAL_INCORRECT: CommandErrorType<1> =
    CommandErrorType::new("argument.literal.incorrect");

pub const DOUBLE_TOO_LOW: CommandErrorType<2> = CommandErrorType::new("argument.double.low");
pub const DOUBLE_TOO_HIGH: CommandErrorType<2> = CommandErrorType::new("argument.double.big");
pub const FLOAT_TOO_LOW: CommandErrorType<2> = CommandErrorType::new("argument.float.low");
pub const FLOAT_TOO_HIGH: CommandErrorType<2> = CommandErrorType::new("argument.float.big");
pub const INTEGER_TOO_LOW: CommandErrorType<2> = CommandErrorType::new("argument.integer.low");
pub const INTEGER_TOO_HIGH: CommandErrorType<2> = CommandErrorType::new("argument.integer.big");
pub const LONG_TOO_LOW: CommandErrorType<2> = CommandErrorType::new("argument.long.low");
pub const LONG_TOO_HIGH: CommandErrorType<2> = CommandErrorType::new("argument.long.big");

pub const DISPATCHER_UNKNOWN_COMMAND: CommandErrorType<0> =
    CommandErrorType::new("command.unknown.command");
pub const DISPATCHER_UNKNOWN_ARGUMENT: CommandErrorType<0> =
    CommandErrorType::new("command.unknown.argument");
pub const DISPATCHER_EXPECTED_ARGUMENT_SEPARATOR: CommandErrorType<0> =
    CommandErrorType::new("command.expected.separator");
pub const DISPATCHER_PARSE_EXCEPTION: CommandErrorType<1> =
    CommandErrorType::new("command.exception");

use crate::command::errors::{
    command_syntax_error::{CommandSyntaxError, ContextProvider},
    error_types::sealed::Sealed,
};
use pumpkin_util::text::TextComponent;

/// Represents text that is generated at
/// compile time and cannot change at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompileTimeText {
    /// This is first translated, then arguments are substituted (which are not constant)
    /// when it needs to be displayed.
    TranslationKey(&'static str),

    /// Directly shows up to the user without any translation done.
    Literal(&'static str),
}

/// A command error that requires **exactly** `N` translation arguments.
/// This takes a translation key. If you want the non-translatable version,
/// use [`LiteralCommandErrorType`].
///
/// **Comparison with Brigadier**:
/// - [`CommandErrorType<0>`] = `SimpleCommandExceptionType`
/// - [`CommandErrorType<1>`] = `DynamicCommandExceptionType`
/// - [`CommandErrorType<2>`] = `Dynamic2CommandExceptionType`
/// - [`CommandErrorType<3>`] = `Dynamic3CommandExceptionType`
/// - [`CommandErrorType<4>`] = `Dynamic4CommandExceptionType`
/// - `CommandErrorType<N>` = `DynamicNCommandExceptionType`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandErrorType<const N: usize> {
    pub translation_key: &'static str,
}

impl<const N: usize> CommandErrorType<N> {
    /// Creates an error type from a given translation string.
    #[must_use]
    pub const fn new(translation_key: &'static str) -> Self {
        Self { translation_key }
    }

    /// Creates an error without context from itself and from a slice of `N` arguments.
    #[must_use]
    pub fn create_without_context_args_slice(
        &'static self,
        args: &[TextComponent; N],
    ) -> CommandSyntaxError {
        CommandSyntaxError::create_without_context(
            self,
            TextComponent::translate(self.translation_key, args),
        )
    }

    /// Creates an error with context from itself and from a slice of `N` arguments.
    pub fn create_args_slice<C>(
        &'static self,
        context_provider: &C,
        args: &[TextComponent; N],
    ) -> CommandSyntaxError
    where
        C: ContextProvider,
    {
        CommandSyntaxError::create(
            self,
            TextComponent::translate(self.translation_key, args),
            context_provider,
        )
    }
}

/// A command error that requires **exactly** `N` translation arguments.
/// This takes a constant string literal. If you want the translatable version,
/// use [`CommandErrorType`].
///
/// **Use this for custom Pumpkin error messages.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LiteralCommandErrorType {
    pub literal: &'static str,
}

impl LiteralCommandErrorType {
    /// Creates an error type from a given literal string.
    #[must_use]
    pub const fn new(literal: &'static str) -> Self {
        Self { literal }
    }

    /// Creates an error without context from itself.
    #[must_use]
    pub fn create_without_context(&'static self) -> CommandSyntaxError {
        CommandSyntaxError::create_without_context(self, TextComponent::text(self.literal))
    }

    /// Creates an error with context from itself.
    pub fn create<C>(&'static self, context_provider: &C) -> CommandSyntaxError
    where
        C: ContextProvider,
    {
        CommandSyntaxError::create(self, TextComponent::text(self.literal), context_provider)
    }
}

// Prevent other crates from using this trait
// Thus, we can effectively 'seal' our trait meant
// only for `CommandErrorType<N>`.
mod sealed {
    /// Private trait to ensure only `CommandErrorType<N>` can implement `AnyCommandErrorType`.
    pub trait Sealed {}
}

/// A trait which is only implemented by [`CommandErrorType<N>`] types.
///
/// It exposes the common properties of such types, while making
/// it more dynamic to access its properties, like the translation
/// key and the number of arguments (at runtime).
pub trait AnyCommandErrorType: Sealed + std::fmt::Debug {
    /// Returns the underlying translation key of this specific error type.
    fn text(&self) -> CompileTimeText;

    /// Returns the number of arguments supported by this error type.
    fn argument_count(&self) -> usize;
}

impl Eq for dyn AnyCommandErrorType {}

impl PartialEq for dyn AnyCommandErrorType {
    fn eq(&self, other: &Self) -> bool {
        self.text() == other.text() && self.argument_count() == other.argument_count()
    }
}

// Implement the private trait for our types.
impl<const N: usize> Sealed for CommandErrorType<N> {}
impl<const N: usize> AnyCommandErrorType for CommandErrorType<N> {
    fn text(&self) -> CompileTimeText {
        CompileTimeText::TranslationKey(self.translation_key)
    }

    fn argument_count(&self) -> usize {
        N
    }
}

impl Sealed for LiteralCommandErrorType {}
impl AnyCommandErrorType for LiteralCommandErrorType {
    fn text(&self) -> CompileTimeText {
        CompileTimeText::Literal(self.literal)
    }

    fn argument_count(&self) -> usize {
        0
    }
}

// Ease-of-use Implementations:

/// Generates a specific implementation for `CommandErrorType<N>` with two methods to create
/// an error without using a slice. Instead, they take a specific number of
/// arguments. Each `impl` is given a specific value of `N` and some argument names.
macro_rules! error_type_no_arg_slice_impl {
    (N = $N: literal => $($arg:ident),* | $doc: expr) => {
        impl CommandErrorType<$N> {
            /// Creates an error without context from this error type
            #[doc = $doc]
            #[must_use]
            pub fn create_without_context(
                &'static self,
                $($arg: TextComponent),*
            ) -> CommandSyntaxError {
                self.create_without_context_args_slice(&[
                    $($arg),*
                ])
            }

            /// Creates an error with context from this error type
            #[doc = $doc]
            #[must_use]
            pub fn create<C>(
                &'static self,
                context_provider: &C,
                $($arg: TextComponent),*
            ) -> CommandSyntaxError
            where
                C: ContextProvider,
            {
                self.create_args_slice(context_provider, &[
                    $($arg),*
                ])
            }
        }
    };
}

// Implementations are defined below:

error_type_no_arg_slice_impl!(N = 0 =>                        | "without taking any translation arguments.");
error_type_no_arg_slice_impl!(N = 1 => arg1                   | "by taking 1 translation argument.");
error_type_no_arg_slice_impl!(N = 2 => arg1, arg2             | "by taking 2 translation arguments.");
error_type_no_arg_slice_impl!(N = 3 => arg1, arg2, arg3       | "by taking 3 translation arguments.");
error_type_no_arg_slice_impl!(N = 4 => arg1, arg2, arg3, arg4 | "by taking 4 translation arguments.");
