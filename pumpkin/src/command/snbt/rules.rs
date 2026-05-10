use std::collections::HashMap;

use crate::command::errors::error_types::{CommandErrorType, LITERAL_INCORRECT};
use crate::command::parser::Parser;
use crate::command::snbt::markers::{
    ArrayPrefix, Base, IntegerLiteral, IntegerSuffix, Sign, Signed, SignedPrefix, TypeSuffix,
};
use crate::command::snbt::operations::SnbtOperations;
use crate::command::snbt::{NUMBER_PARSE_FAILURE, SnbtParser};
use pumpkin_codecs::{DynamicOps, Number};
use pumpkin_data::translation;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::nbt_ops::NbtOps;
use pumpkin_nbt::tag::NbtTag;

pub const INVALID_CODEPOINT: CommandErrorType<1> = CommandErrorType::new(
    translation::java::SNBT_PARSER_INVALID_CODEPOINT,
    translation::java::SNBT_PARSER_INVALID_CODEPOINT,
);

pub const NO_SUCH_OPERATION: CommandErrorType<1> = CommandErrorType::new(
    translation::java::SNBT_PARSER_NO_SUCH_OPERATION,
    translation::java::SNBT_PARSER_NO_SUCH_OPERATION,
);

pub const EXPECTED_FLOAT_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_EXPECTED_FLOAT_TYPE,
    translation::java::SNBT_PARSER_EXPECTED_FLOAT_TYPE,
);

pub const INVALID_CHARACTER_NAME: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_INVALID_CHARACTER_NAME,
    translation::java::SNBT_PARSER_INVALID_CHARACTER_NAME,
);

pub const INVALID_UNQUOTED_START: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_INVALID_UNQUOTED_START,
    translation::java::SNBT_PARSER_INVALID_UNQUOTED_START,
);

pub const EXPECTED_UNQUOTED_STRING: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_EXPECTED_UNQUOTED_STRING,
    translation::java::SNBT_PARSER_EXPECTED_UNQUOTED_STRING,
);

pub const INVALID_STRING_CONTENTS: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_INVALID_STRING_CONTENTS,
    translation::java::SNBT_PARSER_INVALID_STRING_CONTENTS,
);

pub const EXPECTED_BINARY_NUMERAL: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_EXPECTED_BINARY_NUMERAL,
    translation::java::SNBT_PARSER_EXPECTED_BINARY_NUMERAL,
);

pub const EXPECTED_DECIMAL_NUMERAL: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_EXPECTED_DECIMAL_NUMERAL,
    translation::java::SNBT_PARSER_EXPECTED_DECIMAL_NUMERAL,
);

pub const EXPECTED_HEX_NUMERAL: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_EXPECTED_HEX_NUMERAL,
    translation::java::SNBT_PARSER_EXPECTED_HEX_NUMERAL,
);

pub const EMPTY_KEY: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_EMPTY_KEY,
    translation::java::SNBT_PARSER_EMPTY_KEY,
);

pub const LEADING_ZERO_NOT_ALLOWED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_LEADING_ZERO_NOT_ALLOWED,
    translation::java::SNBT_PARSER_LEADING_ZERO_NOT_ALLOWED,
);

pub const INFINITY_NOT_ALLOWED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_INFINITY_NOT_ALLOWED,
    translation::java::SNBT_PARSER_INFINITY_NOT_ALLOWED,
);

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

impl SnbtParser<'_, '_> {
    fn sign(&mut self) -> Option<Sign> {
        self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            match parser.reader.peek() {
                Some('+') => {
                    parser.reader.skip();
                    Some(Sign::Plus)
                }
                Some('-') => {
                    parser.reader.skip();
                    Some(Sign::Minus)
                }
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
            match parser.reader.peek() {
                Some('u' | 'U') => {
                    parser.reader.skip();
                    Some(IntegerSuffix(
                        SignedPrefix::Unsigned,
                        parser.integer_type_suffix()?,
                    ))
                }
                Some('s' | 'S') => {
                    // Can mean signed or short.
                    Some(
                        if let Some(suffix) = parser.parse_or_revert(|parser| {
                            parser.reader.skip();
                            parser.integer_type_suffix()
                        }) {
                            IntegerSuffix(SignedPrefix::Signed, suffix)
                        } else {
                            IntegerSuffix(SignedPrefix::None, parser.integer_type_suffix()?)
                        },
                    )
                }
                Some('b' | 'B' | 'i' | 'I' | 'l' | 'L') => Some(IntegerSuffix(
                    SignedPrefix::None,
                    parser.integer_type_suffix()?,
                )),
                _ => {
                    parser.store_dynamic_error_and_suggest(
                        &LITERAL_INCORRECT,
                        "u|U",
                        &["u", "U", "s", "S", "b", "B", "s", "S", "i", "I", "l", "L"],
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

            // We need to be careful to make sure that
            // `0b` parses as a byte literal and NOT a prefix.
            let after_sign_cursor = parser.reader.cursor();
            if parser.reader.peek() == Some('0') {
                parser.reader.skip();
                parser.reader.skip_whitespace();
                match parser.reader.peek() {
                    Some('x' | 'X') => {
                        parser.reader.skip();
                        return parser.hexadecimal_numeral().map(|number| IntegerLiteral {
                            sign,
                            base: Base::Hexadecimal,
                            suffix: IntegerSuffix::EMPTY,
                            digits: number,
                        });
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
                        parser.reader.set_cursor(after_sign_cursor);
                    }
                    _ => {
                        return if parser.decimal_numeral().is_none() {
                            Some(IntegerLiteral {
                                sign,
                                base: Base::Decimal,
                                suffix: IntegerSuffix::EMPTY,
                                digits: "0".to_string(),
                            })
                        } else {
                            parser.store_simple_error(&LEADING_ZERO_NOT_ALLOWED);
                            None
                        };
                    }
                }
            }
            if let Some(number) = parser.decimal_numeral() {
                return Some(IntegerLiteral {
                    sign,
                    base: Base::Decimal,
                    suffix: IntegerSuffix::EMPTY,
                    digits: number,
                });
            }
            parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "0", &["0"]);
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
            match parser.reader.peek() {
                Some('f' | 'F') => {
                    parser.reader.skip();
                    Some(TypeSuffix::Float)
                }
                Some('d' | 'D') => {
                    parser.reader.skip();
                    Some(TypeSuffix::Double)
                }
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
            if matches!(parser.reader.peek(), Some('e' | 'E')) {
                parser.reader.skip();
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
                Err(_) => {
                    self.store_dynamic_error(&NUMBER_PARSE_FAILURE, "Invalid float literal");
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

    /// Parses an escape sequence (without the \)
    /// The returned character will be expressed as a `u32`
    /// due to Rust's strictness on `char` of surrogate codepoints.
    fn escape_sequence(&mut self) -> Option<u32> {
        enum EscapeSequenceBranch {
            Return(char),
            CheckValidity(u32),
            UnicodeName(String),
        }

        let cursor_at_escaping_char = self.reader.cursor();
        let branch = match self.reader.read() {
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
                u32::from_str_radix(&self.string_hex_2()?, 16)
                    .expect("Hexadecimal parsed should have been valid"),
            )),
            Some('u') => Some(EscapeSequenceBranch::CheckValidity(
                u32::from_str_radix(&self.string_hex_4()?, 16)
                    .expect("Hexadecimal parsed should have been valid"),
            )),
            Some('U') => Some(EscapeSequenceBranch::CheckValidity(
                u32::from_str_radix(&self.string_hex_8()?, 16)
                    .expect("Hexadecimal parsed should have been valid"),
            )),
            Some('N') => {
                if self.reader.peek() != Some('{') {
                    self.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "{", &["}"]);
                    return None;
                }
                self.reader.skip();
                let string_unicode_name = self.string_unicode_name()?;
                if self.reader.peek() != Some('}') {
                    self.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "}", &["}"]);
                    return None;
                }
                self.reader.skip();
                Some(EscapeSequenceBranch::UnicodeName(string_unicode_name))
            }
            _ => {
                self.reader.set_cursor(cursor_at_escaping_char);
                self.store_dynamic_error_and_suggest(
                    &LITERAL_INCORRECT,
                    "b",
                    &[
                        "b", "s", "t", "n", "f", "r", "\\", "'", "\"", "x", "u", "U", "N",
                    ],
                );
                None
            }
        }?;

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
            EscapeSequenceBranch::UnicodeName(_name) => {
                todo!("Unicode Name functionality has not been implemented yet")
            }
        }
    }

    fn quoted_string_literal(&mut self) -> Option<String> {
        self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            match parser.reader.peek() {
                Some('\'') => {
                    parser.reader.skip();
                    parse_string_literal!(parser, '\'')
                }
                Some('"') => {
                    parser.reader.skip();
                    parse_string_literal!(parser, '"')
                }
                _ => {
                    parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "\"", &["'", "\""]);
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

    fn arguments(&mut self) -> Vec<NbtTag> {
        self.repeated_with_trailing_comma_vec(Self::literal)
    }

    fn unquoted_string_or_built_in(&mut self) -> Option<NbtTag> {
        let literal = self.unquoted_string_literal()?;
        // Trying to match the same behaviour of storing arguments
        // in the scope even if the right bracket failed to parse:
        let mut arguments = None;

        let _ = self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            if parser.reader.peek() != Some('(') {
                parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "(", &["("]);
                return None;
            }
            parser.reader.skip();
            arguments = Some(parser.arguments());
            parser.reader.skip_whitespace();
            if parser.reader.peek() == Some(')') {
                parser.reader.skip();
                Some(())
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
            let count = arguments.len();
            if let Some(operation) = SnbtOperations::search(&literal, count) {
                operation(self, &arguments[..])
            } else {
                self.store_dynamic_error(&NO_SUCH_OPERATION, format!("{literal}/{count}"));
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
        self.parse_or_revert(Self::quoted_string_literal)
            .map_or_else(|| self.unquoted_string_literal(), Some)
    }

    fn map_entry(&mut self) -> Option<(String, NbtTag)> {
        let entry = self.parse_or_revert(|parser| {
            let key = parser.map_key()?;
            parser.reader.skip_whitespace();
            if parser.reader.peek() == Some(':') {
                parser.reader.skip();
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

    fn map_entries(&mut self) -> HashMap<String, NbtTag> {
        self.repeated_with_trailing_comma(Self::map_entry, HashMap::new(), |map, element| {
            map.insert(element.0, element.1);
        })
    }

    fn map_literal(&mut self) -> Option<NbtTag> {
        let entries = self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            if parser.reader.peek() != Some('{') {
                parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "{", &["}"]);
                return None;
            }
            parser.reader.skip();
            let entries = parser.map_entries();
            parser.reader.skip_whitespace();
            if parser.reader.peek() != Some('}') {
                parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "}", &["}"]);
                return None;
            }
            parser.reader.skip();
            Some(entries)
        })?;

        Some(NbtTag::Compound(NbtCompound {
            child_tags: entries,
        }))
    }

    fn list_entries(&mut self) -> Vec<NbtTag> {
        self.repeated_with_trailing_comma_vec(Self::literal)
    }

    fn array_prefix(&mut self) -> Option<ArrayPrefix> {
        self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            match parser.reader.peek() {
                Some('B') => {
                    parser.reader.skip();
                    Some(ArrayPrefix::Byte)
                }
                Some('I') => {
                    parser.reader.skip();
                    Some(ArrayPrefix::Int)
                }
                Some('L') => {
                    parser.reader.skip();
                    Some(ArrayPrefix::Long)
                }
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

    fn int_array_entries(&mut self) -> Vec<IntegerLiteral> {
        self.repeated_with_trailing_comma_vec(Self::integer_literal)
    }

    fn list_literal(&mut self) -> Option<NbtTag> {
        self.parse_or_revert(|parser| {
            parser.reader.skip_whitespace();
            if parser.reader.peek() != Some('[') {
                parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "[", &["["]);
                return None;
            }
            parser.reader.skip();

            if let Some((prefix, literals)) = parser.parse_or_revert(|parser| {
                let prefix = parser.array_prefix()?;
                parser.reader.skip_whitespace();
                if parser.reader.peek() == Some(';') {
                    parser.reader.skip();
                    let entries = parser.int_array_entries();
                    parser.reader.skip_whitespace();
                    if parser.reader.peek() != Some(']') {
                        parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "]", &["]"]);
                        return None;
                    }
                    parser.reader.skip();

                    Some((prefix, entries))
                } else {
                    parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, ";", &[";"]);
                    None
                }
            }) {
                parser.create_prefixed_array(&literals[..], prefix)
            } else {
                let entries = parser.list_entries();

                parser.reader.skip_whitespace();
                if parser.reader.peek() != Some(']') {
                    parser.store_dynamic_error_and_suggest(&LITERAL_INCORRECT, "]", &["]"]);
                    return None;
                }
                parser.reader.skip();

                Some(NbtOps.create_list(entries))
            }
        })
    }

    fn literal(&mut self) -> Option<NbtTag> {
        enum Literal {
            Tag(NbtTag),
            Integer(IntegerLiteral),
            String(String),
        }

        self.reader.skip_whitespace();

        // This has to match the actual rules, so this code is pretty awkward.
        let mut result = None;

        if matches!(self.reader.peek(), Some('0'..='9' | '+' | '-' | '.')) {
            if let Some(tag) = self.parse_or_revert(Self::float_literal) {
                result = Some(Literal::Tag(tag));
            } else if let Some(literal) = self.parse_or_revert(Self::integer_literal) {
                result = Some(Literal::Integer(literal));
            }
        }

        let result = if let Some(result) = result {
            result
        } else {
            match self.reader.peek() {
                Some('"' | '\'') => Literal::String(self.quoted_string_literal()?),
                Some('{') => Literal::Tag(self.map_literal()?),
                Some('[') => Literal::Tag(self.list_literal()?),
                _ => Literal::Tag(self.unquoted_string_or_built_in()?),
            }
        };

        Some(match result {
            Literal::Tag(tag) => tag,
            Literal::Integer(int) => {
                match self.parse_integer_literal(&int, int.suffix.1.or(TypeSuffix::Int))? {
                    Number::Byte(byte) => NbtTag::Byte(byte),
                    Number::Short(short) => NbtTag::Short(short),
                    Number::Int(int) => NbtTag::Int(int),
                    Number::Long(long) => NbtTag::Long(long),
                    _ => unreachable!(
                        "Got a floating-point number when only integers should be returned"
                    ),
                }
            }
            Literal::String(string) => NbtTag::String(string),
        })
    }

    pub(super) fn parse(&mut self) -> Option<NbtTag> {
        self.literal()
    }
}
