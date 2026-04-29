#![allow(unused)] // temporary

pub mod errors;
mod markers;
mod operations;

use std::borrow::Cow;

use crate::command::errors::error_types::{CommandErrorType, LITERAL_INCORRECT};
use crate::command::snbt::errors::SnbtErrors;
use crate::command::snbt::markers::{
    ArrayPrefix, Base, IntegerLiteral, IntegerSuffix, Sign, Signed, SignedPrefix, TypeSuffix,
};
use crate::command::snbt::operations::SnbtOperations;
use crate::command::string_reader::StringReader;
use pumpkin_codecs::{DynamicOps, Number};
use pumpkin_data::translation;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::nbt_ops::NbtOps;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::text::TextComponent;

pub const NUMBER_PARSE_FAILURE: CommandErrorType<1> =
    CommandErrorType::new(translation::SNBT_PARSER_NUMBER_PARSE_FAILURE);

pub const EXPECTED_HEX_ESCAPE: CommandErrorType<1> =
    CommandErrorType::new(translation::SNBT_PARSER_EXPECTED_HEX_ESCAPE);

pub const INVALID_CODEPOINT: CommandErrorType<1> =
    CommandErrorType::new(translation::SNBT_PARSER_INVALID_CODEPOINT);

pub const NO_SUCH_OPERATION: CommandErrorType<1> =
    CommandErrorType::new(translation::SNBT_PARSER_NO_SUCH_OPERATION);

pub const EXPECTED_INTEGER_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_EXPECTED_INTEGER_TYPE);

pub const EXPECTED_FLOAT_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_EXPECTED_FLOAT_TYPE);

pub const EXPECTED_NON_NEGATIVE_NUMBER: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_EXPECTED_NON_NEGATIVE_NUMBER);

pub const INVALID_CHARACTER_NAME: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_INVALID_CHARACTER_NAME);

pub const INVALID_ARRAY_ELEMENT_TYPE: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_INVALID_ARRAY_ELEMENT_TYPE);

pub const INVALID_UNQUOTED_START: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_INVALID_UNQUOTED_START);

pub const EXPECTED_UNQUOTED_STRING: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_EXPECTED_UNQUOTED_STRING);

pub const INVALID_STRING_CONTENTS: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_INVALID_STRING_CONTENTS);

pub const EXPECTED_BINARY_NUMERAL: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_EXPECTED_BINARY_NUMERAL);

pub const EXPECTED_DECIMAL_NUMERAL: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_EXPECTED_DECIMAL_NUMERAL);

pub const EXPECTED_HEX_NUMERAL: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_EXPECTED_HEX_NUMERAL);

pub const UNDERSCORE_NOT_ALLOWED: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_UNDESCORE_NOT_ALLOWED);

pub const EMPTY_KEY: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_EMPTY_KEY);

pub const LEADING_ZERO_NOT_ALLOWED: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_LEADING_ZERO_NOT_ALLOWED);

pub const INFINITY_NOT_ALLOWED: CommandErrorType<0> =
    CommandErrorType::new(translation::SNBT_PARSER_INFINITY_NOT_ALLOWED);

/// Traverses through each alternative from left to right,
/// stopping at the first successful parse.
macro_rules! alternatives {
    ($reader: expr, $($alternative:block),*) => {
        'result: {
            let start = $reader.cursor();
            $(
                let result = $alternative;
                if result.is_some() {
                    break 'result result;
                }
                $reader.set_cursor(start);
            )*
            None
        }
    };
}

/// Parses a string literal.
macro_rules! parse_string_literal {
    ($parser:expr, $quote:literal) => {{
        let mut buffer = String::new();
        let mut high_surrogate_queue: u32 = 0;

        loop {
            match $parser.reader.read() {
                Some($quote) => break Some(buffer),
                Some('\\') => {
                    let i = $parser.escape_sequence()?;
                    if let Some(c) = char::from_u32(i) {
                        buffer.push(c);
                    } else if high_surrogate_queue == 0 && matches!(i, 0xD800..=0xDBFF) {
                        // High surrogate incoming.
                        high_surrogate_queue = i;
                    } else if high_surrogate_queue != 0 && matches!(i, 0xDC00..=0xDFFF) {
                        // Low surrogate incoming.
                        let high_bits = high_surrogate_queue - 0xD800;
                        let low_bits = i - 0xDC00;
                        let bits = high_bits << 10 | low_bits;
                        let i = bits + 0x10000;
                        // This really shouldn't fail though.
                        if let Some(c) = char::from_u32(i) {
                            buffer.push(c);
                        } else {
                            buffer.push('\u{FFFD}');
                        }
                        high_surrogate_queue = 0;
                    } else {
                        // Add replacement character.
                        buffer.push('\u{FFFD}');
                        if high_surrogate_queue != 0 {
                            buffer.push('\u{FFFD}');
                        }
                        high_surrogate_queue = 0;
                    }
                }
                Some(ch) => {
                    if high_surrogate_queue != 0 {
                        // Add replacement character.
                        buffer.push('\u{FFFD}');
                        high_surrogate_queue = 0;
                    }
                    buffer.push(ch);
                }
                None => {
                    // reached EOL
                    $parser.store_simple_error_and_suggest(
                        &INVALID_STRING_CONTENTS,
                        &["'", "\"", "\\"],
                    );
                    break None;
                }
            }
        }
    }};
}

/// A structure that parses SNBT.
///
/// This stores a reader and gives the furthest error, or suggestions
/// to fix errors that have ever occurred while parsing.
pub struct SnbtParser<'r, 's> {
    reader: &'r mut StringReader<'s>,
    errors: SnbtErrors,
}

//
// CREATION
//
impl<'r, 's> SnbtParser<'r, 's> {
    /// Creates a new [`SnbtParser`] from a string reader.
    fn new(reader: &'r mut StringReader<'s>) -> Self {
        SnbtParser {
            reader,
            errors: SnbtErrors::default(),
        }
    }
}

//
// RULES
//
impl SnbtParser<'_, '_> {
    fn sign(&mut self) -> Option<Sign> {
        self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            match parser.reader.read() {
                Some('+') => Some(Sign::Plus),
                Some('-') => Some(Sign::Minus),
                _ => {
                    parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "+", &["+", "-"]);
                    None
                }
            }
        })
    }

    fn integer_suffix(&mut self) -> Option<IntegerSuffix> {
        self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            match parser.reader.read() {
                Some('u' | 'U') => Some(IntegerSuffix(
                    SignedPrefix::Unsigned,
                    parser.integer_type_suffix()?,
                )),
                Some('s' | 'S') => Some(IntegerSuffix(
                    SignedPrefix::Signed,
                    parser.integer_type_suffix()?,
                )),
                _ => {
                    parser.store_dynamic_error_and_suggest(
                        &LITERAL_INCORRECT,
                        "u|U",
                        &["u", "U", "s", "S"],
                    );
                    None
                }
            }
        })
    }

    fn binary_numeral(&mut self) -> Option<String> {
        self.parse_numeral(Base::Binary)
    }

    fn decimal_numeral(&mut self) -> Option<String> {
        self.parse_numeral(Base::Decimal)
    }

    fn hexadecimal_numeral(&mut self) -> Option<String> {
        self.parse_numeral(Base::Hexadecimal)
    }

    /// Parses an integer literal.
    fn integer_literal(&mut self) -> Option<IntegerLiteral> {
        let mut result = self.parse_or_revert(|parser| {
            let sign = parser.parse_or_revert(Self::sign).unwrap_or(Sign::Plus);
            parser.reader.skip_whitespace();
            if parser.reader.peek() == Some('0') {
                parser.reader.skip();
                parser.reader.skip_whitespace();
                match parser.reader.peek() {
                    Some('x' | 'X') => {
                        parser.reader.skip();
                        if let Some(number) = parser.hexadecimal_numeral() {
                            return Some(IntegerLiteral {
                                sign,
                                base: Base::Hexadecimal,
                                suffix: IntegerSuffix::EMPTY,
                                digits: number,
                            });
                        }
                    }
                    Some('b' | 'B') => {
                        parser.reader.skip();
                        if let Some(number) = parser.binary_numeral() {
                            return Some(IntegerLiteral {
                                sign,
                                base: Base::Binary,
                                suffix: IntegerSuffix::EMPTY,
                                digits: number,
                            });
                        }
                    }
                    _ => {
                        if parser.decimal_numeral().is_none() {
                            return Some(IntegerLiteral {
                                sign,
                                base: Base::Decimal,
                                suffix: IntegerSuffix::EMPTY,
                                digits: "0".to_string(),
                            });
                        }
                        parser.store_simple_error(&LEADING_ZERO_NOT_ALLOWED);
                    }
                }
            } else if let Some(number) = parser.decimal_numeral() {
                return Some(IntegerLiteral {
                    sign,
                    base: Base::Decimal,
                    suffix: IntegerSuffix::EMPTY,
                    digits: number,
                });
            } else {
                parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "0", &["0"]);
            }
            None
        })?;

        result.suffix = self
            .parse_or_revert(Self::integer_suffix)
            .unwrap_or(IntegerSuffix::EMPTY);

        Some(result)
    }

    fn float_type_suffix(&mut self) -> Option<TypeSuffix> {
        self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            match parser.reader.read() {
                Some('f' | 'F') => Some(TypeSuffix::Float),
                Some('d' | 'D') => Some(TypeSuffix::Double),
                _ => {
                    parser.store_dynamic_error_and_suggest(
                        &LITERAL_INCORRECT,
                        "f|F",
                        &["f", "F", "d", "D"],
                    );
                    None
                }
            }
        })
    }

    fn float_exponent_part(&mut self) -> Option<Signed<String>> {
        self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            if matches!(parser.reader.read(), Some('e' | 'E')) {
                let sign = parser.parse_or_revert(Self::sign).unwrap_or(Sign::Plus);
                let value = parser.decimal_numeral()?;

                Some(Signed { sign, value })
            } else {
                parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "e|E", &["e", "E"]);
                None
            }
        })
    }

    fn float_literal(&mut self) -> Option<NbtTag> {
        struct FloatingPointIntermediate {
            whole_part: String,
            fraction_part: Option<String>,
            exponent_part: Option<Signed<String>>,
            type_suffix: Option<TypeSuffix>,
        }

        // Paths:
        // A --- XXX.[yyy][eZZZ][suffix]
        // B --- .yyy[eZZZ][suffix]
        // C --- XXXeZZZ[suffix]
        // D --- XXX[eZZZ]suffix
        //
        // where [a] means 'optionally parse a',
        //       XXX is the whole part, yyy is the decimal part,
        //       eZZZ is the float exponent path, and
        //       suffix is float type suffix.
        //
        // Ruleset:
        // If we encounter a digit, we must parse a decimal number. Then:
        //     If we encounter a decimal point, we must choose path A.
        //     Try to parse [eZZZ] AND [suffix]:
        //         if [eZZZ] parses, then irrespective of [suffix], choose path D.
        //         if ONLY [suffix] parses, choose path C.
        //         if none parse, FAIL.
        // If we encounter a decimal point, we must choose path B.
        // FAIL if nether a period or a digit

        let sign = self.parse_or_revert(Self::sign).unwrap_or(Sign::Plus);

        let intermediate = self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            if let Some(whole_part) = parser.parse_or_revert(Self::decimal_numeral) {
                // Must be pathway A, C, or D.
                parser.reader.skip_whitespace();
                if parser.reader.peek() == Some('.') {
                    // We choose pathway A.
                    parser.reader.skip();

                    let fraction_part = parser.decimal_numeral();
                    let exponent_part = parser.float_exponent_part();
                    let type_suffix = parser.float_type_suffix();

                    Some(FloatingPointIntermediate {
                        whole_part,
                        fraction_part,
                        exponent_part,
                        type_suffix,
                    })
                } else {
                    // This error won't actually matter if the following part
                    // parses successfully.
                    parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, ".", &["."]);

                    // Must be pathway C or D.
                    let exponent_part = parser.float_exponent_part();
                    let type_suffix = parser.float_type_suffix();

                    (exponent_part.is_some() || type_suffix.is_some()).then_some(
                        FloatingPointIntermediate {
                            whole_part,
                            fraction_part: None,
                            exponent_part,
                            type_suffix,
                        },
                    )
                }
            } else {
                // We must parse a decimal point.
                parser.reader.skip_whitespace();
                if parser.reader.peek() == Some('.') {
                    parser.reader.skip();
                    // We choose pathway B.
                    let fraction_part = parser.decimal_numeral()?;
                    let exponent_part = parser.float_exponent_part();
                    let type_suffix = parser.float_type_suffix();

                    Some(FloatingPointIntermediate {
                        whole_part: String::new(),
                        fraction_part: Some(fraction_part),
                        exponent_part,
                        type_suffix,
                    })
                } else {
                    // We cannot choose a pathway.
                    parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, ".", &["."]);
                    None
                }
            }
        })?;

        // Parsing the float:
        let mut buffer = String::with_capacity(
            sign.minimum_size_parsable()
                + intermediate.whole_part.len()
                + intermediate
                    .fraction_part
                    .as_ref()
                    .map_or(0, |s| 1 + s.len())
                + intermediate
                    .exponent_part
                    .as_ref()
                    .map_or(0, |s| 1 + s.sign.minimum_size_parsable() + s.value.len()),
        );

        sign.append_minimum_str_parsable(&mut buffer);
        Self::clean_and_append(&mut buffer, &intermediate.whole_part);
        if let Some(fraction) = &intermediate.fraction_part {
            buffer.push('.');
            Self::clean_and_append(&mut buffer, fraction);
        }
        if let Some(exponent) = &intermediate.exponent_part {
            buffer.push('e');
            exponent.sign.append_minimum_str_parsable(&mut buffer);
            Self::clean_and_append(&mut buffer, &exponent.value);
        }

        match intermediate.type_suffix {
            None | Some(TypeSuffix::Double) => match buffer.parse::<f64>() {
                Err(_) => self.store_dynamic_error(&NUMBER_PARSE_FAILURE, "Invalid float literal"),
                Ok(value) if value.is_finite() => {
                    return Some(NbtTag::Double(value));
                }
                Ok(_) => self.store_simple_error(&INFINITY_NOT_ALLOWED),
            },
            Some(TypeSuffix::Float) => match buffer.parse::<f32>() {
                Err(error) => {
                    self.store_dynamic_error(&NUMBER_PARSE_FAILURE, "Invalid float literal")
                }
                Ok(value) if value.is_finite() => {
                    return Some(NbtTag::Float(value));
                }
                Ok(_) => self.store_simple_error(&INFINITY_NOT_ALLOWED),
            },
            _ => self.store_simple_error(&EXPECTED_FLOAT_TYPE),
        }

        None
    }

    fn string_hex_2(&mut self) -> Option<String> {
        self.hex_literal(2)
    }

    fn string_hex_4(&mut self) -> Option<String> {
        self.hex_literal(4)
    }

    fn string_hex_8(&mut self) -> Option<String> {
        self.hex_literal(8)
    }

    /// Parses a unicode name pattern.
    fn string_unicode_name(&mut self) -> Option<String> {
        self.parse_or_revert(|parser| {
            let start = parser.reader.cursor();
            let mut end = start;

            // Since the only characters allowed are all ASCII, it should
            // be fine to go byte by byte.
            let bytes = parser.reader.string().as_bytes();

            while end < bytes.len() {
                let b = bytes[end];
                if matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b' ' | b'-') {
                    end += 1;
                } else {
                    break;
                }
            }

            if start == end {
                parser.store_simple_error(&INVALID_CHARACTER_NAME);
                None
            } else {
                parser.reader.set_cursor(end);
                Some(parser.reader.string()[start..end].to_string())
            }
        })
    }

    /// Parses an escape sequence.
    /// The returned character will be expressed as a `u32`
    /// due to Rust's strictness on `char` of surrogate codepoints.
    fn escape_sequence(&mut self) -> Option<u32> {
        enum EscapeSequenceBranch {
            Return(char),
            CheckValidity(u32),
            UnicodeName(String),
        };

        let branch = self.parse_or_revert(|parser| match parser.reader.read() {
            Some('b') => Some(EscapeSequenceBranch::Return('\x08')),
            Some('s') => Some(EscapeSequenceBranch::Return(' ')),
            Some('t') => Some(EscapeSequenceBranch::Return('\t')),
            Some('n') => Some(EscapeSequenceBranch::Return('\n')),
            Some('f') => Some(EscapeSequenceBranch::Return('\x0C')),
            Some('r') => Some(EscapeSequenceBranch::Return('\r')),
            Some('\\') => Some(EscapeSequenceBranch::Return('\\')),
            Some('\'') => Some(EscapeSequenceBranch::Return('\'')),
            Some('"') => Some(EscapeSequenceBranch::Return('"')),
            Some('x') => Some(EscapeSequenceBranch::CheckValidity(
                u32::from_str_radix(&parser.string_hex_2()?, 16)
                    .expect("Hexadecimal parsed should have been valid"),
            )),
            Some('u') => Some(EscapeSequenceBranch::CheckValidity(
                u32::from_str_radix(&parser.string_hex_4()?, 16)
                    .expect("Hexadecimal parsed should have been valid"),
            )),
            Some('U') => Some(EscapeSequenceBranch::CheckValidity(
                u32::from_str_radix(&parser.string_hex_8()?, 16)
                    .expect("Hexadecimal parsed should have been valid"),
            )),
            Some('N') => {
                if parser.reader.read() != Some('{') {
                    parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "{", &["}"]);
                    return None;
                }
                let string_unicode_name = parser.string_unicode_name()?;
                if parser.reader.read() != Some('}') {
                    parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "}", &["}"]);
                    return None;
                }
                Some(EscapeSequenceBranch::UnicodeName(string_unicode_name))
            }
            _ => {
                parser.store_dynamic_error_and_suggest(
                    &LITERAL_INCORRECT,
                    "b",
                    &[
                        "b", "s", "t", "n", "f", "r", "\\", "'", "\"", "x", "u", "U", "N",
                    ],
                );
                None
            }
        })?;

        match branch {
            EscapeSequenceBranch::Return(ch) => Some(ch as u32),
            EscapeSequenceBranch::CheckValidity(value) => {
                // Value must be <= 0x10FFFF to be a valid codepoint.
                // (Surrogates are handled outside this function)
                if value <= 0x10FFFF {
                    Some(value)
                } else {
                    self.store_dynamic_error(&INVALID_CODEPOINT, format!("U+{value:08X}"));
                    None
                }
            }
            EscapeSequenceBranch::UnicodeName(name) => todo!(),
        }
    }

    fn quoted_string_literal(&mut self) -> Option<String> {
        self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            match parser.reader.read() {
                Some('\'') => parse_string_literal!(parser, '\''),
                Some('"') => parse_string_literal!(parser, '"'),
                _ => {
                    parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "'", &["'", "\""]);
                    None
                }
            }
        })
    }

    fn unquoted_string_literal(&mut self) -> Option<String> {
        self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            let value = parser.reader.read_unquoted_string();
            if value.is_empty() {
                parser.store_simple_error(&EXPECTED_UNQUOTED_STRING);
                None
            } else {
                Some(value)
            }
        })
    }

    fn arguments(&mut self) -> Option<Vec<NbtTag>> {
        self.repeated_with_trailing_comma(Self::literal)
    }

    fn unquoted_string_or_built_in(&mut self) -> Option<NbtTag> {
        let literal = self.unquoted_string_literal()?;
        let arguments = self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            if parser.reader.read() != Some('(') {
                parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "(", &["("]);
                return None;
            }
            let arguments = parser.arguments()?;
            if parser.reader.read() == Some(')') {
                Some(arguments)
            } else {
                parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, ")", &[")"]);
                None
            }
        });

        // This should be fine as the characters in the predicate are all ASCII.
        if literal.is_empty() || matches!(literal.as_bytes()[0], b'0'..=b'9' | b'+' | b'-' | b'.') {
            self.store_simple_error_and_suggest(
                &INVALID_UNQUOTED_START,
                SnbtOperations::BUILTIN_IDS,
            );
            return None;
        }

        if let Some(arguments) = arguments {
            if let Some(operation) = SnbtOperations::search(&literal, arguments.len()) {
                operation(self, &arguments[..])
            } else {
                self.store_dynamic_error(&NO_SUCH_OPERATION, literal);
                None
            }
        } else if literal.eq_ignore_ascii_case("true") {
            Some(NbtTag::Byte(1))
        } else if literal.eq_ignore_ascii_case("false") {
            Some(NbtTag::Byte(0))
        } else {
            Some(NbtTag::String(literal))
        }
    }

    fn map_key(&mut self) -> Option<String> {
        self.quoted_string_literal()
            .map_or_else(|| self.unquoted_string_literal(), Some)
    }

    fn map_entry(&mut self) -> Option<(String, NbtTag)> {
        let entry = self.parse_or_revert(|parser| {
            let key = parser.map_key()?;
            parser.reader.skip_whitespace();
            if parser.reader.read() == Some(':') {
                Some((key, parser.literal()?))
            } else {
                parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, ":", &[":"]);
                None
            }
        })?;

        if entry.0.is_empty() {
            self.store_simple_error(&EMPTY_KEY);
            None
        } else {
            Some(entry)
        }
    }

    fn map_entries(&mut self) -> Option<Vec<(String, NbtTag)>> {
        self.repeated_with_trailing_comma(Self::map_entry)
    }

    fn map_literal(&mut self) -> Option<NbtTag> {
        let entries = self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            if parser.reader.read() != Some('{') {
                parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "{", &["}"]);
                return None;
            }
            let entries = parser.map_entries()?;
            if parser.reader.read() != Some('}') {
                parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "}", &["}"]);
                return None;
            }
            Some(entries)
        })?;

        Some(NbtTag::Compound(NbtCompound {
            child_tags: entries,
        }))
    }

    fn list_entries(&mut self) -> Option<Vec<NbtTag>> {
        self.repeated_with_trailing_comma(Self::literal)
    }

    fn array_prefix(&mut self) -> Option<ArrayPrefix> {
        self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            match parser.reader.read() {
                Some('B') => Some(ArrayPrefix::Byte),
                Some('I') => Some(ArrayPrefix::Int),
                Some('L') => Some(ArrayPrefix::Long),
                _ => {
                    parser.store_dynamic_error_and_suggest(
                        &LITERAL_INCORRECT,
                        "B",
                        &["B", "I", "L"],
                    );
                    None
                }
            }
        })
    }

    fn int_array_entries(&mut self) -> Option<Vec<IntegerLiteral>> {
        self.repeated_with_trailing_comma(Self::integer_literal)
    }

    fn list_literal(&mut self) -> Option<NbtTag> {
        self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            if parser.reader.read() != Some('[') {
                parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "[", &["["]);
                return None;
            }

            if let Some((prefix, literals)) = parser.parse_or_revert(|parser| {
                let prefix = parser.array_prefix()?;
                parser.reader.skip_whitespace();
                if parser.reader.read() != Some(';') {
                    parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, ";", &[";"]);
                    None
                } else {
                    Some((prefix, parser.int_array_entries()?))
                }
            }) {
                parser.create_prefixed_array(&literals[..], prefix)
            } else {
                Some(NbtOps.create_list(parser.list_entries()?))
            }
        })
    }

    fn literal(&mut self) -> Option<NbtTag> {
        enum Literal {
            Tag(NbtTag),
            Integer(IntegerLiteral),
            String(String),
        }

        let literal = self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            match parser.reader.peek_byte() {
                Some(b'0'..=b'9' | b'+' | b'-' | b'.') => {
                    if let Some(result) = parser.parse_or_revert(Self::float_literal) {
                        Some(Literal::Tag(result))
                    } else {
                        Some(Literal::Integer(parser.integer_literal()?))
                    }
                }
                Some(b'"' | b'\'') => Some(Literal::String(parser.quoted_string_literal()?)),
                Some(b'{') => Some(Literal::Tag(parser.map_literal()?)),
                Some(b'[') => Some(Literal::Tag(parser.list_literal()?)),
                _ => Some(Literal::Tag(parser.unquoted_string_or_built_in()?)),
            }
        })?;

        Some(match literal {
            Literal::Tag(tag) => tag,
            Literal::Integer(int) => {
                NbtOps.create_number(self.parse_integer_literal(&int, TypeSuffix::Int)?)
            }
            Literal::String(string) => NbtTag::String(string),
        })
    }
}

//
// HELPER FUNCTIONS
//
impl SnbtParser<'_, '_> {
    /// Records that a simple error occurred while parsing, and adds suggestions to counteract it.
    fn store_simple_error_and_suggest(
        &mut self,
        error_type: &'static CommandErrorType<0>,
        suggestions: &[&'static str],
    ) {
        self.errors
            .simple_static(self.reader, error_type, suggestions);
    }

    /// Records that a dynamic error occurred while parsing, and adds suggestions to counteract it.
    fn store_dynamic_error_and_suggest(
        &mut self,
        error_type: &'static CommandErrorType<1>,
        arg1: impl Into<Cow<'static, str>>,
        suggestions: &[&'static str],
    ) {
        self.errors
            .dynamic_static(self.reader, error_type, arg1, suggestions);
    }

    /// Records that a simple error occurred while parsing.
    fn store_simple_error(&mut self, error_type: &'static CommandErrorType<0>) {
        self.errors.simple(self.reader, error_type, Vec::new());
    }

    /// Records that a dynamic error occurred while parsing.
    fn store_dynamic_error(
        &mut self,
        error_type: &'static CommandErrorType<1>,
        arg1: impl Into<Cow<'static, str>>,
    ) {
        self.errors
            .dynamic(self.reader, error_type, arg1, Vec::new());
    }

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
                // Only B|b is given as the error, being the first errored choice.
                self.store_dynamic_error_and_suggest(
                    &LITERAL_INCORRECT,
                    "B|b",
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

    fn repeated_with_trailing_comma<T>(
        &mut self,
        rule: impl Fn(&mut Self) -> Option<T>,
    ) -> Option<Vec<T>> {
        let list_cursor = self.reader.cursor();
        let mut elements = Vec::new();
        let mut first = true;

        loop {
            if !first {
                self.parse_or_revert(|parser| {
                    parser.reader.skip_whitespace();
                    if parser.reader.read() == Some(',') {
                        Some(())
                    } else {
                        parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, ",", &[","]);
                        None
                    }
                })?;
            }

            elements.push(self.parse_or_revert(&rule)?);
            first = false;
        }

        Some(elements)
    }

    fn parse_integer_literal(
        &mut self,
        literal: &IntegerLiteral,
        suffix: TypeSuffix,
    ) -> Option<Number> {
        let unsigned = literal.suffix.0 == SignedPrefix::Unsigned;
        if unsigned && literal.sign == Sign::Minus {
            self.store_simple_error(&EXPECTED_NON_NEGATIVE_NUMBER);
            return None;
        }

        let mut number = String::with_capacity(literal.digits.len());
        Self::clean_and_append(&mut number, &literal.digits);
        let radix = literal.base.radix();

        // The error messages vary by a lot to match the error messages in Java.
        match (unsigned, suffix) {
            (false, TypeSuffix::Byte) => {
                let integer = self.parse_int_or_error(&number, radix)?;

                if let Ok(byte) = integer.try_into() {
                    Some(Number::Byte(byte))
                } else {
                    self.store_dynamic_error(
                        &NUMBER_PARSE_FAILURE,
                        format!("Value out of range. Value:\"{number}\" Radix:{radix}"),
                    );
                    None
                }
            }
            (false, TypeSuffix::Short) => {
                let integer = self.parse_int_or_error(&number, radix)?;

                if let Ok(short) = integer.try_into() {
                    Some(Number::Short(short))
                } else {
                    self.store_dynamic_error(
                        &NUMBER_PARSE_FAILURE,
                        format!("Value out of range. Value:\"{number}\" Radix:{radix}"),
                    );
                    None
                }
            }
            (false, TypeSuffix::Int) => Some(Number::Int(self.parse_int_or_error(&number, radix)?)),
            (false, TypeSuffix::Long) => {
                if let Ok(long) = i64::from_str_radix(&number, radix) {
                    Some(Number::Long(long))
                } else {
                    self.store_dynamic_error(
                        &NUMBER_PARSE_FAILURE,
                        format!("For input string: \"{number}\""),
                    );
                    None
                }
            }
            (true, TypeSuffix::Byte) => {
                let integer = self.parse_int_or_error(&number, radix)?;

                if let Ok(byte) = TryInto::<u8>::try_into(integer) {
                    Some(Number::Byte(byte as i8))
                } else {
                    self.store_dynamic_error(
                        &NUMBER_PARSE_FAILURE,
                        format!("out of range: {number}"),
                    );
                    None
                }
            }
            (true, TypeSuffix::Short) => {
                let integer = self.parse_int_or_error(&number, radix)?;

                if let Ok(short) = TryInto::<u16>::try_into(integer) {
                    Some(Number::Short(short as i16))
                } else {
                    self.store_dynamic_error(
                        &NUMBER_PARSE_FAILURE,
                        format!("out of range: {number}"),
                    );
                    None
                }
            }
            (true, TypeSuffix::Int) => {
                if let Ok(int) = u32::from_str_radix(&number, radix) {
                    Some(Number::Int(int as i32))
                } else {
                    self.store_dynamic_error(
                        &NUMBER_PARSE_FAILURE,
                        format!("String value {number} exceeds range of unsigned int."),
                    );
                    None
                }
            }
            (true, TypeSuffix::Long) => {
                if let Ok(long) = u64::from_str_radix(&number, radix) {
                    Some(Number::Long(long as i64))
                } else {
                    self.store_dynamic_error(
                        &NUMBER_PARSE_FAILURE,
                        format!("String value {number} exceeds range of unsigned long."),
                    );
                    None
                }
            }
            _ => {
                self.store_simple_error(&EXPECTED_INTEGER_TYPE);
                None
            }
        }
    }

    fn parse_int_or_error(&mut self, number: &str, radix: u32) -> Option<i32> {
        if let Ok(int) = i32::from_str_radix(&number, radix) {
            Some(int)
        } else {
            self.store_dynamic_error(
                &NUMBER_PARSE_FAILURE,
                format!("For input string: \"{number}\""),
            );
            None
        }
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
        Some(NbtTag::ByteArray(bytes.into_boxed_slice()))
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

#[cfg(test)]
mod test {}
