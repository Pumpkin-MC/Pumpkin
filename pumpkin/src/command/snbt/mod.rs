pub mod error_entries;
mod markers;

use std::ops::Range;

use crate::command::errors::error_types::{CommandErrorType, LITERAL_INCORRECT};
use crate::command::snbt::error_entries::ErrorEntries;
use crate::command::snbt::markers::{IntegerSuffix, SignedPrefix, TypeSuffix};
use crate::command::string_reader::StringReader;
use num_bigint::Sign;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

pub const EXPECTED_BINARY_NUMERAL: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_EXPECTED_BINARY_NUMERAL);

pub const EXPECTED_DECIMAL_NUMERAL: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_EXPECTED_DECIMAL_NUMERAL);

pub const EXPECTED_HEX_NUMERAL: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_EXPECTED_HEX_NUMERAL);

pub const UNDERSCORE_NOT_ALLOWED: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_UNDESCORE_NOT_ALLOWED);

/// A structure that parses SNBT.
///
/// This stores a reader and gives the furthest error, or suggestions
/// to fix errors that have ever occurred while parsing.
struct SnbtParser<'a, E: ErrorEntries> {
    reader: StringReader<'a>,
    errors: SnbtErrors<E>,
}

/// A structure that represents
/// errors recorded while parsing.
struct SnbtErrors<E: ErrorEntries> {
    cursor: usize,
    entries: E,
}

/// Represents a type of numeral.
#[derive(Copy, Clone, Debug)]
pub enum Numeral {
    Binary,
    Decimal,
    Hexadecimal,
}

impl Numeral {
    pub fn should_allow(self, c: char) -> bool {
        match (self, c) {
            (_, '_') => true,
            (Numeral::Binary, '0' | '1') => true,
            (Numeral::Decimal, '0'..='9') => true,
            (Numeral::Hexadecimal, '0'..='9' | 'A'..='F' | 'a'..='f') => true,
            _ => false,
        }
    }

    pub fn no_value_error_type(self) -> &'static CommandErrorType<0> {
        match self {
            Numeral::Binary => &EXPECTED_BINARY_NUMERAL,
            Numeral::Decimal => &EXPECTED_DECIMAL_NUMERAL,
            Numeral::Hexadecimal => &EXPECTED_HEX_NUMERAL,
        }
    }
}

//
// RULES
//
impl<'a, E: ErrorEntries> SnbtParser<'a, E> {
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
                self.store_dynamic_error_and_suggest(
                    &LITERAL_INCORRECT,
                    || TextComponent::text("+"),
                    || vec!['+'.to_string(), '-'.to_string()],
                );
                None
            }
        }
    }

    fn integer_suffix(&mut self) -> Option<IntegerSuffix> {
        self.reader.skip_whitespace();
        match self.reader.peek() {
            Some('u') | Some('U') => {
                self.reader.skip();
                Some(IntegerSuffix(
                    SignedPrefix::Unsigned,
                    self.integer_type_suffix()?,
                ))
            }
            Some('s') | Some('S') => {
                self.reader.skip();
                Some(IntegerSuffix(
                    SignedPrefix::Signed,
                    self.integer_type_suffix()?,
                ))
            }
            _ => {
                self.store_dynamic_error_and_suggest(
                    &LITERAL_INCORRECT,
                    || TextComponent::text("u|U"),
                    || {
                        vec![
                            'u'.to_string(),
                            'U'.to_string(),
                            's'.to_string(),
                            'S'.to_string(),
                        ]
                    },
                );
                None
            }
        }
    }

    /// If successful, returns the range of the indices bordering the successfully parsed numeral.
    fn binary_numeral(&mut self) -> Option<Range<usize>> {
        self.parse_numeral(Numeral::Binary)
    }

    /// If successful, returns the range of the indices bordering the successfully parsed numeral.
    fn decimal_numeral(&mut self) -> Option<Range<usize>> {
        self.parse_numeral(Numeral::Decimal)
    }

    /// If successful, returns the range of the indices bordering the successfully parsed numeral.
    fn hexadecimal_numeral(&mut self) -> Option<Range<usize>> {
        self.parse_numeral(Numeral::Hexadecimal)
    }
}

//
// HELPER FUNCTIONS
//
impl<'a, E: ErrorEntries> SnbtParser<'a, E> {
    /// Records that a simple error occurred while parsing, and adds suggestions to counteract it.
    fn store_simple_error_and_suggest(
        &mut self,
        error_type: &'static CommandErrorType<0>,
        suggestions: impl FnOnce() -> Vec<String>,
    ) {
        self.errors
            .entries
            .simple(&self.reader, error_type, suggestions);
    }

    /// Records that a dynamic error occurred while parsing, and adds suggestions to counteract it.
    fn store_dynamic_error_and_suggest(
        &mut self,
        error_type: &'static CommandErrorType<1>,
        arg1: impl FnOnce() -> TextComponent,
        suggestions: impl FnOnce() -> Vec<String>,
    ) {
        self.errors
            .entries
            .dynamic(&self.reader, error_type, arg1, suggestions);
    }

    /// Records that a simple error occurred while parsing.
    fn store_simple_error(&mut self, error_type: &'static CommandErrorType<0>) {
        self.errors
            .entries
            .simple(&self.reader, error_type, || vec![]);
    }

    /// Records that a dynamic error occurred while parsing.
    fn store_dynamic_error(
        &mut self,
        error_type: &'static CommandErrorType<1>,
        arg1: impl FnOnce() -> TextComponent,
    ) {
        self.errors
            .entries
            .dynamic(&self.reader, error_type, arg1, || vec![]);
    }

    /// Utility method that parses a type suffix of an integer.
    fn integer_type_suffix(&mut self) -> Option<TypeSuffix> {
        self.reader.skip_whitespace();
        match self.reader.peek() {
            Some('b') | Some('B') => {
                self.reader.skip();
                Some(TypeSuffix::Byte)
            }
            Some('s') | Some('S') => {
                self.reader.skip();
                Some(TypeSuffix::Short)
            }
            Some('i') | Some('I') => {
                self.reader.skip();
                Some(TypeSuffix::Int)
            }
            Some('l') | Some('L') => {
                self.reader.skip();
                Some(TypeSuffix::Long)
            }
            _ => {
                // Only B|b is given as the error, being the first erroneous choice.
                self.store_dynamic_error_and_suggest(
                    &LITERAL_INCORRECT,
                    || TextComponent::text("B|b"),
                    || {
                        vec![
                            'b'.to_string(),
                            'B'.to_string(),
                            's'.to_string(),
                            'S'.to_string(),
                            'i'.to_string(),
                            'I'.to_string(),
                            'l'.to_string(),
                            'L'.to_string(),
                        ]
                    },
                );
                None
            }
        }
    }

    /// Utility method that parses an integer of a specific radix.
    /// If successful, returns the range of the indices bordering the successfully parsed string.
    fn parse_numeral(&mut self, numeral: Numeral) -> Option<Range<usize>> {
        self.reader.skip_whitespace();
        let slice = self.reader.string();

        let start = self.reader.cursor();

        let mut end = slice.len();
        for (i, c) in slice[start..].char_indices() {
            if !numeral.should_allow(c) {
                end = start + i;
                break;
            }
        }

        if start == end {
            self.store_simple_error(numeral.no_value_error_type());
            None
        } else if slice.as_bytes()[start] == b'_' || slice.as_bytes()[end - 1] == b'_' {
            self.store_simple_error(&UNDERSCORE_NOT_ALLOWED);
            None
        } else {
            Some(start..end)
        }
    }
}
