use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::{CommandErrorType, LITERAL_INCORRECT};
use crate::command::string_reader::StringReader;
use pumpkin_util::text::TextComponent;

/// A structure that parses SNBT.
///
/// This stores a reader and gives the furthest error, or suggestions
/// to fix errors that have ever occurred while parsing.
struct SnbtParser<'a, E: ErrorEntries> {
    reader: StringReader<'a>,
    errors: SnbtErrors<E>
}

/// A structure that represents
/// errors recorded while parsing.
struct SnbtErrors<E: ErrorEntries> {
    cursor: usize,
    entries: E
}

enum Sign {
    Plus,
    Minus
}

enum SignedPrefix {
    None,
    Unsigned,
    Signed
}

enum TypeSuffix {
    None,
    Byte,
    Short,
    Int,
    Long
}

struct IntegerSuffix(SignedPrefix, TypeSuffix);

impl IntegerSuffix {
    const EMPTY: IntegerSuffix = IntegerSuffix(SignedPrefix::None, TypeSuffix::None);
}

// Rules
impl<'a, E> SnbtParser<'a, E> {
    fn sign(&mut self) -> Option<Sign> {
        self.reader.skip_whitespace();
        match self.reader.peek() {
            Some('+') => {
                self.reader.skip();
                Some(Sign::Plus)
            }
            Some('-') => {
                self.reader.skip();
                Some(Sign::Minus)
            }
            _ => {
                self.store_dynamic_error(
                    &LITERAL_INCORRECT,
                    || TextComponent::text("+"),
                    || vec!['+'.to_string(), '-'.to_string()]
                );
                None
            }
        }
    }

    fn integer_suffix(&mut self) -> Option<IntegerSuffix> {
        self.reader.skip_whitespace();
        Some(
            match self.reader.peek() {
                Some('u') | Some('U') => {
                    self.reader.skip();
                    IntegerSuffix(
                        SignedPrefix::Unsigned,
                        self.integer_type_suffix()?
                    )
                },
                Some('s') | Some('S') => {
                    self.reader.skip();
                    IntegerSuffix(
                        SignedPrefix::Signed,
                        self.integer_type_suffix()?
                    )
                }
                _ => {
                    self.store_dynamic_error(
                        &LITERAL_INCORRECT,
                        || TextComponent::text("u|U"),
                        || vec!['u'.to_string(), 'U'.to_string(), 's'.to_string(), 'S'.to_string()]
                    );
                    None
                }
            }
        )
    }
}

// Helper functions
impl<'a> SnbtParser<'a> {
    /// Records that a simple error occurred while parsing.
    /// Only the longest error is kept track of (i.e. the cursor that has proceeded the furthest)
    fn store_simple_error(
        &mut self,
        error_type: &'static CommandErrorType<0>,
        suggestions: impl FnOnce() -> Vec<String>,
    ) {
        if !self
            .longest_error
            .as_ref()
            .is_some_and(|error| self.reader.cursor() <= error.cursor)
        {
            self.longest_error = Some(SnbtError {
                cursor: self.reader.cursor(),
                error: error_type.create(&self.reader),
                suggestions: suggestions(),
            });
        }
    }

    /// Records that a dynamic error occurred while parsing.
    /// Only the longest error is kept track of (i.e. the cursor that has proceeded the furthest)
    fn store_dynamic_error(
        &mut self,
        error_type: &'static CommandErrorType<1>,
        arg: impl FnOnce() -> TextComponent,
        suggestions: impl FnOnce() -> Vec<String>,
    ) {
        if !self
            .longest_error
            .as_ref()
            .is_some_and(|error| self.reader.cursor() <= error.cursor)
        {
            self.longest_error = Some(SnbtError {
                cursor: self.reader.cursor(),
                error: error_type.create(&self.reader, arg()),
                suggestions: suggestions(),
            });
        }
    }

    /// Utility method that parses a type suffix of an integer.
    fn integer_type_suffix(&mut self) -> Option<TypeSuffix> {
        self.reader.skip_whitespace();
        match self.reader.peek() {
            Some('b') | Some('B') => {
                self.reader.skip();
                Some(TypeSuffix::Byte)
            },
            Some('s') | Some('S') => {
                self.reader.skip();
                Some(TypeSuffix::Short)
            },
            Some('i') | Some('I') => {
                self.reader.skip();
                Some(TypeSuffix::Int)
            },
            Some('l') | Some('L') => {
                self.reader.skip();
                Some(TypeSuffix::Long)
            },
            _ => {
                // Only B|b is given as the error, being the first erroneous choice.
                self.store_dynamic_error(
                    &LITERAL_INCORRECT,
                    || TextComponent::text("B|b"),
                    || vec![
                        'b'.to_string(),
                        'B'.to_string(),
                        's'.to_string(),
                        'S'.to_string(),
                        'i'.to_string(),
                        'I'.to_string(),
                        'l'.to_string(),
                        'L'.to_string(),
                    ]
                );
                None
            }
        }
    }

    /// Utility method that parses an integer of a specific radix.
    fn parse_numeral(&mut self) -> Option<String> {
        self.reader.skip_whitespace();
        let slice = self.reader.string();
        let start = self.reader.cursor();
        let pos = start;

        while pos < slice.len() &&
    }
}

trait ErrorEntries {
    fn simple(
        previous: Self,
        reader: &StringReader,
        error_type: &'static CommandErrorType<0>,
        suggestions: impl FnOnce() -> Vec<String>,
    ) -> Self;

    fn dynamic(
        previous: Self,
        reader: &StringReader,
        error_type: &'static CommandErrorType<1>,
        arg1: impl FnOnce() -> TextComponent,
        suggestions: impl FnOnce() -> Vec<String>,
    ) -> Self;
}

/// A data structure that keeps track of errors. However,
/// this structure only stores at most 1 error as only up to 1
/// error will be thrown to the sender.
struct CommandErrorEntries(Option<CommandSyntaxError>);
impl ErrorEntries for CommandErrorEntries {
    fn simple(previous: Self, reader: &StringReader, error_type: &'static CommandErrorType<0>, _: impl FnOnce() -> Vec<String>) -> Self {
        // We only store the first 'longest' error that occurred.
        // 'Longest' here means the error that occurred the furthest in the string.
        match previous {
            CommandErrorEntries(None) => CommandErrorEntries(Some(error_type.create(reader))),
            CommandErrorEntries(Some(_)) => previous
        }
    }

    fn dynamic(mut previous: Self, reader: &StringReader, error_type: &'static CommandErrorType<1>, arg1: impl FnOnce() -> TextComponent, _: impl FnOnce() -> Vec<String>) -> Self {
        // We only store the first 'longest' error that occurred.
        // 'Longest' here means the error that occurred the furthest in the string.
        match previous {
            CommandErrorEntries(None) => CommandErrorEntries(Some(error_type.create(reader, arg1))),
            CommandErrorEntries(Some(_)) => previous
        }
    }
}

/// A data structure that keeps track of suggestions to fix errors.
struct SuggestionsErrorEntries(Vec<String>);
impl ErrorEntries for SuggestionsErrorEntries {
    fn simple(mut previous: Self, _: &StringReader, _: &'static CommandErrorType<0>, suggestions: impl FnOnce() -> Vec<String>) -> Self {
        previous.0.append(&mut suggestions());
        previous
    }

    fn dynamic(mut previous: Self, _: &StringReader, _: &'static CommandErrorType<1>, _: impl FnOnce() -> TextComponent, suggestions: impl FnOnce() -> Vec<String>) -> Self {
        previous.0.append(&mut suggestions());
        previous
    }
}