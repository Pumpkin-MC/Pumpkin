use pumpkin_data::translation;

// These are akin to the translatable built-in exceptions in Minecraft.
pub const READER_EXPECTED_START_QUOTE: CommandErrorType<0> =
    CommandErrorType::new(translation::PARSING_QUOTE_EXPECTED_START);
pub const READER_EXPECTED_END_QUOTE: CommandErrorType<0> =
    CommandErrorType::new(translation::PARSING_QUOTE_EXPECTED_END);
pub const READER_INVALID_ESCAPE: CommandErrorType<1> =
    CommandErrorType::new(translation::PARSING_QUOTE_ESCAPE);
pub const READER_INVALID_BOOL: CommandErrorType<1> =
    CommandErrorType::new(translation::PARSING_BOOL_INVALID);
pub const READER_EXPECTED_BOOL: CommandErrorType<0> =
    CommandErrorType::new(translation::PARSING_BOOL_EXPECTED);
pub const READER_INVALID_INT: CommandErrorType<1> =
    CommandErrorType::new(translation::PARSING_INT_INVALID);
pub const READER_EXPECTED_INT: CommandErrorType<0> =
    CommandErrorType::new(translation::PARSING_INT_EXPECTED);
pub const READER_INVALID_LONG: CommandErrorType<1> =
    CommandErrorType::new(translation::PARSING_LONG_INVALID);
pub const READER_EXPECTED_LONG: CommandErrorType<0> =
    CommandErrorType::new(translation::PARSING_LONG_EXPECTED);
pub const READER_INVALID_DOUBLE: CommandErrorType<1> =
    CommandErrorType::new(translation::PARSING_DOUBLE_INVALID);
pub const READER_EXPECTED_DOUBLE: CommandErrorType<0> =
    CommandErrorType::new(translation::PARSING_DOUBLE_EXPECTED);
pub const READER_INVALID_FLOAT: CommandErrorType<1> =
    CommandErrorType::new(translation::PARSING_FLOAT_INVALID);
pub const READER_EXPECTED_FLOAT: CommandErrorType<0> =
    CommandErrorType::new(translation::PARSING_FLOAT_EXPECTED);
pub const READER_EXPECTED_SYMBOL: CommandErrorType<1> =
    CommandErrorType::new(translation::PARSING_EXPECTED);

pub const LITERAL_INCORRECT: CommandErrorType<1> =
    CommandErrorType::new(translation::ARGUMENT_LITERAL_INCORRECT);

pub const DOUBLE_TOO_LOW: CommandErrorType<2> =
    CommandErrorType::new(translation::ARGUMENT_DOUBLE_LOW);
pub const DOUBLE_TOO_HIGH: CommandErrorType<2> =
    CommandErrorType::new(translation::ARGUMENT_DOUBLE_BIG);
pub const FLOAT_TOO_LOW: CommandErrorType<2> =
    CommandErrorType::new(translation::ARGUMENT_FLOAT_LOW);
pub const FLOAT_TOO_HIGH: CommandErrorType<2> =
    CommandErrorType::new(translation::ARGUMENT_FLOAT_BIG);
pub const INTEGER_TOO_LOW: CommandErrorType<2> =
    CommandErrorType::new(translation::ARGUMENT_INTEGER_LOW);
pub const INTEGER_TOO_HIGH: CommandErrorType<2> =
    CommandErrorType::new(translation::ARGUMENT_INTEGER_BIG);
pub const LONG_TOO_LOW: CommandErrorType<2> = CommandErrorType::new(translation::ARGUMENT_LONG_LOW);
pub const LONG_TOO_HIGH: CommandErrorType<2> =
    CommandErrorType::new(translation::ARGUMENT_LONG_BIG);

pub const DISPATCHER_UNKNOWN_COMMAND: CommandErrorType<0> =
    CommandErrorType::new(translation::COMMAND_UNKNOWN_COMMAND);
pub const DISPATCHER_UNKNOWN_ARGUMENT: CommandErrorType<0> =
    CommandErrorType::new(translation::COMMAND_UNKNOWN_ARGUMENT);
pub const DISPATCHER_EXPECTED_ARGUMENT_SEPARATOR: CommandErrorType<0> =
    CommandErrorType::new(translation::COMMAND_EXPECTED_SEPARATOR);
pub const DISPATCHER_PARSE_EXCEPTION: CommandErrorType<1> =
    CommandErrorType::new(translation::COMMAND_EXCEPTION);

use crate::command::errors::{
    command_syntax_error::{CommandSyntaxError, ContextProvider},
    error_types::sealed::Sealed,
};
use pumpkin_util::text::TextComponent;

/// Represents text which can be used as a template that is generated at
/// compile time and cannot change at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemplateText {
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

/// A command error that is not translated, and cannot take any arguments.
/// This takes a constant string literal. If you want the translatable version,
/// use [`CommandErrorType`].
///
/// [`CommandErrorType`] should be preferred to this whenever possible.
///
/// Use this for custom error messages, which don't have any
/// translation in vanilla.
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
    fn text(&self) -> TemplateText;

    /// Returns the number of arguments supported by this error type.
    fn argument_count(&self) -> usize;
}

impl Eq for dyn AnyCommandErrorType {}

impl PartialEq for dyn AnyCommandErrorType {
    fn eq(&self, other: &Self) -> bool {
        self.text() == other.text() && self.argument_count() == other.argument_count()
    }
}

impl<T: AnyCommandErrorType> PartialEq<T> for dyn AnyCommandErrorType {
    fn eq(&self, other: &T) -> bool {
        self.text() == other.text() && self.argument_count() == other.argument_count()
    }
}

// Implement the private trait for our types.
impl<const N: usize> Sealed for CommandErrorType<N> {}
impl<const N: usize> AnyCommandErrorType for CommandErrorType<N> {
    fn text(&self) -> TemplateText {
        TemplateText::TranslationKey(self.translation_key)
    }

    fn argument_count(&self) -> usize {
        N
    }
}

impl Sealed for LiteralCommandErrorType {}
impl AnyCommandErrorType for LiteralCommandErrorType {
    fn text(&self) -> TemplateText {
        TemplateText::Literal(self.literal)
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

#[cfg(test)]
mod test {
    use crate::command::errors::error_types::{CommandErrorType, LiteralCommandErrorType};
    use crate::command::string_reader::StringReader;
    use pumpkin_util::text::TextComponent;

    const TEST_LITERAL_ERROR_TYPE: LiteralCommandErrorType =
        LiteralCommandErrorType::new("Test error");
    const TEST_TRANSLATABLE_ERROR_TYPE: CommandErrorType<1> =
        CommandErrorType::new("this.key.is.arbitrary");

    #[test]
    fn create_literal_error() {
        let mut reader = StringReader::new("foo bar");
        reader.set_cursor(4);

        let error = TEST_LITERAL_ERROR_TYPE.create(&reader);

        assert_eq!(error.error_type, &TEST_LITERAL_ERROR_TYPE);
        assert_eq!(error.message, TextComponent::text("Test error"));

        match &error.context {
            Some(context) => {
                assert_eq!(context.cursor, 4);
                assert_eq!(context.input, "foo bar");
            }
            None => panic!("There should have been a context for the error"),
        }
    }

    #[test]
    fn create_literal_error_without_context() {
        let error = TEST_LITERAL_ERROR_TYPE.create_without_context();

        assert_eq!(error.error_type, &TEST_LITERAL_ERROR_TYPE);
        assert_eq!(error.message, TextComponent::text("Test error"));
        assert_eq!(error.context, None);
    }

    #[test]
    fn create_translatable_error() {
        let mut reader = StringReader::new("foo bar");
        reader.set_cursor(4);

        let error =
            TEST_TRANSLATABLE_ERROR_TYPE.create(&reader, TextComponent::text("some argument"));

        assert_eq!(error.error_type, &TEST_TRANSLATABLE_ERROR_TYPE);
        assert_eq!(
            error.message,
            TextComponent::translate(
                "this.key.is.arbitrary",
                [TextComponent::text("some argument")]
            )
        );

        match &error.context {
            Some(context) => {
                assert_eq!(context.cursor, 4);
                assert_eq!(context.input, "foo bar");
            }
            None => panic!("There should have been a context for the error"),
        }
    }
}
