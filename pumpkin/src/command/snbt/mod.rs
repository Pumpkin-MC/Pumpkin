mod markers;
mod operations;
mod rules;

#[cfg(test)]
mod tests;

use crate::command::errors::command_syntax_error::{CommandSyntaxError, CommandSyntaxErrorContext};
use crate::command::errors::error_types::{CommandErrorType, LITERAL_INCORRECT};
use crate::command::parser::{Parser, ParserErrors};
use crate::command::snbt::markers::{
    ArrayPrefix, Base, IntegerLiteral, Sign, SignedPrefix, TypeSuffix,
};
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_codecs::Number;
use pumpkin_data::translation;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::text::TextComponent;

pub const NUMBER_PARSE_FAILURE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::SNBT_PARSER_NUMBER_PARSE_FAILURE,
    translation::java::SNBT_PARSER_NUMBER_PARSE_FAILURE,
);

pub const UNDERSCORE_NOT_ALLOWED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_UNDESCORE_NOT_ALLOWED,
    translation::java::SNBT_PARSER_UNDESCORE_NOT_ALLOWED,
);

pub const EXPECTED_HEX_ESCAPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::SNBT_PARSER_EXPECTED_HEX_ESCAPE,
    translation::java::SNBT_PARSER_EXPECTED_HEX_ESCAPE,
);

pub const EXPECTED_NON_NEGATIVE_NUMBER: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_EXPECTED_NON_NEGATIVE_NUMBER,
    translation::java::SNBT_PARSER_EXPECTED_NON_NEGATIVE_NUMBER,
);

pub const INVALID_ARRAY_ELEMENT_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_INVALID_ARRAY_ELEMENT_TYPE,
    translation::java::SNBT_PARSER_INVALID_ARRAY_ELEMENT_TYPE,
);

pub const EXPECTED_INTEGER_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_EXPECTED_INTEGER_TYPE,
    translation::java::SNBT_PARSER_EXPECTED_INTEGER_TYPE,
);

/// A structure that parses SNBT.
///
/// This stores a reader and gives the furthest error, or suggestions
/// to fix errors that have ever occurred while parsing.
pub struct SnbtParser<'r, 's> {
    reader: &'r mut StringReader<'s>,
    errors: ParserErrors,
}

//
// USAGE
//
impl SnbtParser<'_, '_> {
    /// Parses SNBT with a given [`StringReader`], giving the result or error from parsing.
    pub fn parse_for_commands(reader: &mut StringReader) -> Result<NbtTag, CommandSyntaxError> {
        let (result, errors) = {
            let mut parser = SnbtParser {
                reader,
                errors: ParserErrors::default(),
            };

            let literal = parser.parse();
            let errors = parser.errors;

            (literal, errors)
        };

        result.ok_or_else(|| {
            if let Some(error) = errors.command_error {
                CommandSyntaxError {
                    error_type: error.error_type,
                    message: TextComponent::translate_cross(
                        error.java_translation_key,
                        error.bedrock_translation_key,
                        error
                            .arguments
                            .into_iter()
                            .map(TextComponent::text)
                            .collect::<Vec<_>>(),
                    ),
                    context: Some(CommandSyntaxErrorContext { input: reader.string().to_string(), cursor: errors.cursor }),
                }
            } else {
                // This shouldn't happen... If it didn't parse successfully, there should be an error to supplement it.
                // Hacky way to report an error:
                const PARSING_FAILED_WITHOUT_ERRORS: CommandErrorType<0> = CommandErrorType::new(
                    translation::java::COMMAND_FAILED,
                    translation::java::COMMAND_FAILED
                );
                tracing::error!("Failed to parse SNBT, while having zero errors to report (report this to Pumpkin): {}", reader.string());
                PARSING_FAILED_WITHOUT_ERRORS.create(reader)
            }
        })
    }

    // Parses SNBT with a given [`StringReader`], giving the suggestions to fix errors from parsing.
    pub fn parse_for_suggestions(
        reader: &mut StringReader,
        mut builder: SuggestionsBuilder,
    ) -> Suggestions {
        let mut parser = SnbtParser {
            reader,
            errors: ParserErrors::default(),
        };

        let _ = parser.parse();

        if !parser.errors.suggestions.is_empty() {
            builder = builder.create_offset(parser.errors.cursor);
            for suggestion in &parser.errors.suggestions {
                builder = builder.suggest(suggestion.to_string());
            }
        }

        builder.build()
    }
}

//
// HELPER FUNCTIONS
//
impl SnbtParser<'_, '_> {
    /// Utility method that parses a type suffix of an integer.
    fn integer_type_suffix(&mut self) -> Option<TypeSuffix> {
        self.reader.skip_whitespace();
        match self.reader.peek() {
            Some('b' | 'B') => {
                self.reader.skip();
                Some(TypeSuffix::Byte)
            }
            Some('s' | 'S') => {
                self.reader.skip();
                Some(TypeSuffix::Short)
            }
            Some('i' | 'I') => {
                self.reader.skip();
                Some(TypeSuffix::Int)
            }
            Some('l' | 'L') => {
                self.reader.skip();
                Some(TypeSuffix::Long)
            }
            _ => {
                // Only b|B is given as the error, being the first errored choice.
                self.store_dynamic_error_and_suggest(
                    &LITERAL_INCORRECT,
                    "b|B",
                    &["b", "B", "s", "S", "i", "I", "l", "L"],
                );
                None
            }
        }
    }

    /// General method that parses an integer of a specific base.
    fn parse_numeral(&mut self, base: Base) -> Option<String> {
        self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            let slice = parser.reader.string();

            let start = parser.reader.cursor();

            let mut end = start;
            for (i, c) in slice[start..].char_indices() {
                if !base.should_allow(c) {
                    break;
                }
                end = start + i + c.len_utf8();
            }

            if start == end {
                parser.store_simple_error(base.no_value_error_type());
                None
            } else if slice.as_bytes()[start] == b'_' || slice.as_bytes()[end - 1] == b'_' {
                parser.store_simple_error(&UNDERSCORE_NOT_ALLOWED);
                None
            } else {
                parser.reader.set_cursor(end);
                Some(parser.reader.string()[start..end].to_string())
            }
        })
    }

    /// Parses a value, and if unsuccessful, reverts back to what the state initially was.
    #[inline]
    fn parse_or_revert<T>(&mut self, closure: impl FnOnce(&mut Self) -> Option<T>) -> Option<T> {
        let start = self.reader.cursor();
        let result = closure(self);
        if result.is_none() {
            self.reader.set_cursor(start);
        }
        result
    }

    /// Appends every character given in the `reference` slice except `_` in the provided `buffer`.
    fn clean_and_append(buffer: &mut String, reference: &str) {
        // This could really be optimized further
        // with bytes instead of chars, but that
        // probably requires unsafe code. Is that worth it?
        // TODO
        for c in reference.chars() {
            if c != '_' {
                buffer.push(c);
            }
        }
    }

    /// General method to parse a specific number of hexadecimal digits greedily (no underscores are allowed).
    fn hex_literal(&mut self, digits: usize) -> Option<String> {
        self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            let slice = parser.reader.string();

            let start = parser.reader.cursor();

            let mut end = start;
            for (count, (i, c)) in slice[start..].char_indices().enumerate() {
                if count == digits || !c.is_ascii_hexdigit() {
                    break;
                }
                end = start + i + c.len_utf8();
            }

            if end - start < digits {
                parser.store_dynamic_error(&EXPECTED_HEX_ESCAPE, digits.to_string());
                None
            } else {
                parser.reader.set_cursor(end);
                Some(parser.reader.string()[start..end].to_string())
            }
        })
    }

    fn repeated_with_trailing_comma<T, S>(
        &mut self,
        rule: impl Fn(&mut Self) -> Option<T>,
        new: S,
        insert: impl Fn(&mut S, T),
    ) -> S {
        let mut elements = new;
        let mut first = true;

        loop {
            if !first {
                let parse_comma = self.parse_or_revert(|parser| {
                    parser.reader.skip_whitespace();
                    if parser.reader.peek() == Some(',') {
                        parser.reader.skip();
                        Some(())
                    } else {
                        parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, ",", &[","]);
                        None
                    }
                });
                if parse_comma.is_none() {
                    break;
                }
            }

            if let Some(parsed) = self.parse_or_revert(&rule) {
                insert(&mut elements, parsed);
            } else {
                break;
            }

            first = false;
        }

        elements
    }

    fn repeated_with_trailing_comma_vec<T>(
        &mut self,
        rule: impl Fn(&mut Self) -> Option<T>,
    ) -> Vec<T> {
        self.repeated_with_trailing_comma(rule, Vec::new(), Vec::push)
    }

    fn parse_integer_literal(
        &mut self,
        literal: &IntegerLiteral,
        suffix: TypeSuffix,
    ) -> Option<Number> {
        let signed = literal.get_signed_prefix_or_default() == SignedPrefix::Signed;
        if !signed && literal.sign == Sign::Minus {
            self.store_simple_error(&EXPECTED_NON_NEGATIVE_NUMBER);
            return None;
        }

        let mut number =
            String::with_capacity(literal.digits.len() + literal.sign.minimum_size_parsable());

        literal.sign.append_minimum_str_parsable(&mut number);
        Self::clean_and_append(&mut number, &literal.digits);

        let radix = literal.base.radix();

        // The error messages vary by a lot to match the error messages in Java.
        match (signed, suffix) {
            (true, TypeSuffix::Byte) => {
                let integer = self.parse_int_or_error(&number, radix)?;

                integer.try_into().map_or_else(
                    |_| {
                        self.store_dynamic_error(
                            &NUMBER_PARSE_FAILURE,
                            format!("Value out of range. Value:\"{number}\" Radix:{radix}"),
                        );
                        None
                    },
                    |byte| Some(Number::Byte(byte)),
                )
            }
            (true, TypeSuffix::Short) => {
                let integer = self.parse_int_or_error(&number, radix)?;

                integer.try_into().map_or_else(
                    |_| {
                        self.store_dynamic_error(
                            &NUMBER_PARSE_FAILURE,
                            format!("Value out of range. Value:\"{number}\" Radix:{radix}"),
                        );
                        None
                    },
                    |short| Some(Number::Short(short)),
                )
            }
            (true, TypeSuffix::Int) => Some(Number::Int(self.parse_int_or_error(&number, radix)?)),
            (true, TypeSuffix::Long) => i64::from_str_radix(&number, radix).map_or_else(
                |_| {
                    self.store_dynamic_error(
                        &NUMBER_PARSE_FAILURE,
                        format!("For input string: \"{number}\""),
                    );
                    None
                },
                |long| Some(Number::Long(long)),
            ),
            (false, TypeSuffix::Byte) => {
                let integer = self.parse_int_or_error(&number, radix)?;

                TryInto::<u8>::try_into(integer).map_or_else(
                    |_| {
                        self.store_dynamic_error(
                            &NUMBER_PARSE_FAILURE,
                            format!("out of range: {number}"),
                        );
                        None
                    },
                    |byte| Some(Number::Byte(byte as i8)),
                )
            }
            (false, TypeSuffix::Short) => {
                let integer = self.parse_int_or_error(&number, radix)?;

                TryInto::<u16>::try_into(integer).map_or_else(
                    |_| {
                        self.store_dynamic_error(
                            &NUMBER_PARSE_FAILURE,
                            format!("out of range: {number}"),
                        );
                        None
                    },
                    |short| Some(Number::Short(short as i16)),
                )
            }
            (false, TypeSuffix::Int) => u32::from_str_radix(&number, radix).map_or_else(
                |_| {
                    self.store_dynamic_error(
                        &NUMBER_PARSE_FAILURE,
                        format!("String value {number} exceeds range of unsigned int."),
                    );
                    None
                },
                |int| Some(Number::Int(int as i32)),
            ),
            (false, TypeSuffix::Long) => u64::from_str_radix(&number, radix).map_or_else(
                |_| {
                    self.store_dynamic_error(
                        &NUMBER_PARSE_FAILURE,
                        format!("String value {number} exceeds range of unsigned long."),
                    );
                    None
                },
                |long| Some(Number::Long(long as i64)),
            ),
            _ => {
                self.store_simple_error(&EXPECTED_INTEGER_TYPE);
                None
            }
        }
    }

    fn parse_int_or_error(&mut self, number: &str, radix: u32) -> Option<i32> {
        i32::from_str_radix(number, radix).map_or_else(
            |_| {
                self.store_dynamic_error(
                    &NUMBER_PARSE_FAILURE,
                    format!("For input string: \"{number}\""),
                );
                None
            },
            Some,
        )
    }

    fn create_prefixed_array(
        &mut self,
        values: &[IntegerLiteral],
        prefix: ArrayPrefix,
    ) -> Option<NbtTag> {
        match prefix {
            ArrayPrefix::Byte => self.create_byte_array(values),
            ArrayPrefix::Int => self.create_int_array(values),
            ArrayPrefix::Long => self.create_long_array(values),
        }
    }

    fn create_byte_array(&mut self, values: &[IntegerLiteral]) -> Option<NbtTag> {
        let mut bytes = Vec::with_capacity(values.len());
        for value in values {
            if !matches!(value.suffix.1, TypeSuffix::None | TypeSuffix::Byte) {
                self.store_simple_error(&INVALID_ARRAY_ELEMENT_TYPE);
                return None;
            }
            bytes.push(self.parse_integer_literal(value, TypeSuffix::Byte)?.into());
        }
        Some(NbtTag::ByteArray(bytes))
    }

    fn create_int_array(&mut self, values: &[IntegerLiteral]) -> Option<NbtTag> {
        let mut ints = Vec::with_capacity(values.len());
        for value in values {
            let suffix = value.suffix.1.or(TypeSuffix::Int);
            if !matches!(
                suffix,
                TypeSuffix::Byte | TypeSuffix::Short | TypeSuffix::Int
            ) {
                self.store_simple_error(&INVALID_ARRAY_ELEMENT_TYPE);
                return None;
            }
            ints.push(self.parse_integer_literal(value, suffix)?.into());
        }
        Some(NbtTag::IntArray(ints))
    }

    fn create_long_array(&mut self, values: &[IntegerLiteral]) -> Option<NbtTag> {
        let mut longs = Vec::with_capacity(values.len());
        for value in values {
            let suffix = value.suffix.1.or(TypeSuffix::Long);
            if !matches!(
                suffix,
                TypeSuffix::Byte | TypeSuffix::Short | TypeSuffix::Int | TypeSuffix::Long
            ) {
                self.store_simple_error(&INVALID_ARRAY_ELEMENT_TYPE);
                return None;
            }
            longs.push(self.parse_integer_literal(value, suffix)?.into());
        }
        Some(NbtTag::LongArray(longs))
    }
}

impl<'r, 's> Parser<'r, 's> for SnbtParser<'r, 's> {
    fn state_mut(&mut self) -> (&mut StringReader<'s>, &mut ParserErrors) {
        (self.reader, &mut self.errors)
    }
}
