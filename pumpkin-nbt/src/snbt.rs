//! SNBT (Stringified NBT) parser.
//!
//! Parses SNBT text format into [`NbtTag`] values. SNBT is Minecraft's
//! human-readable text representation of NBT data, used in commands,
//! data packs, and debugging.
//!
//! # SNBT Syntax
//!
//! | NBT Type    | SNBT Example                  |
//! |-------------|-------------------------------|
//! | Byte        | `1b` or `1B` or `true`/`false`|
//! | Short       | `1s` or `1S`                  |
//! | Int         | `1`                           |
//! | Long        | `1l` or `1L`                  |
//! | Float       | `1.0f` or `1.0F`              |
//! | Double      | `1.0d` or `1.0D` or `1.0`     |
//! | String      | `"hello"` or `'hello'` or `hello` |
//! | `ByteArray` | `[B; 1b, 2b, 3b]`            |
//! | `IntArray`  | `[I; 1, 2, 3]`               |
//! | `LongArray` | `[L; 1L, 2L, 3L]`            |
//! | List        | `[1, 2, 3]`                   |
//! | Compound    | `{key: value, key2: value2}`  |
//!
//! # Example
//!
//! ```
//! use pumpkin_nbt::snbt::from_snbt;
//! use pumpkin_nbt::tag::NbtTag;
//!
//! let tag = from_snbt("{name: \"Steve\", health: 20.0f}").unwrap();
//! if let NbtTag::Compound(compound) = &tag {
//!     assert_eq!(compound.get_string("name"), Some("Steve"));
//!     assert_eq!(compound.get_float("health"), Some(20.0));
//! }
//! ```

use crate::compound::NbtCompound;
use crate::tag::NbtTag;

/// Errors that can occur during SNBT parsing.
#[derive(Debug)]
pub enum SnbtError {
    /// Unexpected end of input.
    UnexpectedEof,
    /// Expected a specific character but found something else.
    Expected {
        expected: char,
        found: Option<char>,
        pos: usize,
    },
    /// Invalid number format.
    InvalidNumber { text: String, pos: usize },
    /// Unterminated string literal.
    UnterminatedString { pos: usize },
    /// Unexpected character at the given position.
    UnexpectedChar { ch: char, pos: usize },
    /// Trailing characters after a complete value.
    TrailingData { pos: usize },
    /// Invalid escape sequence in a string.
    InvalidEscape { ch: char, pos: usize },
}

impl std::fmt::Display for SnbtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "Unexpected end of SNBT input"),
            Self::Expected {
                expected,
                found,
                pos,
            } => match found {
                Some(ch) => write!(f, "Expected '{expected}' at position {pos}, found '{ch}'"),
                None => write!(f, "Expected '{expected}' at position {pos}, found EOF"),
            },
            Self::InvalidNumber { text, pos } => {
                write!(f, "Invalid number '{text}' at position {pos}")
            }
            Self::UnterminatedString { pos } => {
                write!(f, "Unterminated string starting at position {pos}")
            }
            Self::UnexpectedChar { ch, pos } => {
                write!(f, "Unexpected character '{ch}' at position {pos}")
            }
            Self::TrailingData { pos } => {
                write!(f, "Trailing data at position {pos}")
            }
            Self::InvalidEscape { ch, pos } => {
                write!(f, "Invalid escape sequence '\\{ch}' at position {pos}")
            }
        }
    }
}

impl std::error::Error for SnbtError {}

/// Parse an SNBT string into an [`NbtTag`].
///
/// The input must contain exactly one complete SNBT value with no trailing
/// data (except whitespace).
pub fn from_snbt(input: &str) -> Result<NbtTag, SnbtError> {
    let mut parser = SnbtParser::new(input);
    let tag = parser.parse_value()?;
    parser.skip_whitespace();
    if parser.pos < parser.input.len() {
        return Err(SnbtError::TrailingData { pos: parser.pos });
    }
    Ok(tag)
}

/// Parse an SNBT string into an [`NbtCompound`].
///
/// Convenience function that parses a compound and returns it directly.
/// The input must represent a compound value (starting with `{`).
pub fn from_snbt_compound(input: &str) -> Result<NbtCompound, SnbtError> {
    match from_snbt(input)? {
        NbtTag::Compound(compound) => Ok(compound),
        _other => Err(SnbtError::UnexpectedChar {
            ch: input.chars().next().unwrap_or(' '),
            pos: 0,
        }),
    }
}

struct SnbtParser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> SnbtParser<'a> {
    const fn new(input: &'a str) -> Self {
        Self {
            input: input.as_bytes(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let ch = self.input.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, expected: u8) -> Result<(), SnbtError> {
        match self.advance() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => Err(SnbtError::Expected {
                expected: expected as char,
                found: Some(ch as char),
                pos: self.pos - 1,
            }),
            None => Err(SnbtError::Expected {
                expected: expected as char,
                found: None,
                pos: self.pos,
            }),
        }
    }

    fn parse_value(&mut self) -> Result<NbtTag, SnbtError> {
        self.skip_whitespace();

        match self.peek() {
            None => Err(SnbtError::UnexpectedEof),
            Some(b'{') => self.parse_compound().map(NbtTag::Compound),
            Some(b'[') => self.parse_list_or_array(),
            Some(b'"') => self.parse_quoted_string(b'"').map(NbtTag::String),
            Some(b'\'') => self.parse_quoted_string(b'\'').map(NbtTag::String),
            Some(_) => self.parse_primitive(),
        }
    }

    fn parse_compound(&mut self) -> Result<NbtCompound, SnbtError> {
        self.expect(b'{')?;
        self.skip_whitespace();

        let mut compound = NbtCompound::new();

        if self.peek() == Some(b'}') {
            self.advance();
            return Ok(compound);
        }

        loop {
            self.skip_whitespace();

            // Parse key: quoted or unquoted string
            let key = self.parse_key()?;

            self.skip_whitespace();
            self.expect(b':')?;

            let value = self.parse_value()?;
            compound.child_tags.push((key, value));

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.advance();
                }
                Some(b'}') => {
                    self.advance();
                    return Ok(compound);
                }
                Some(ch) => {
                    return Err(SnbtError::Expected {
                        expected: '}',
                        found: Some(ch as char),
                        pos: self.pos,
                    });
                }
                None => return Err(SnbtError::UnexpectedEof),
            }
        }
    }

    fn parse_key(&mut self) -> Result<String, SnbtError> {
        match self.peek() {
            Some(b'"') => self.parse_quoted_string(b'"'),
            Some(b'\'') => self.parse_quoted_string(b'\''),
            _ => self.parse_unquoted_string(),
        }
    }

    fn parse_quoted_string(&mut self, quote: u8) -> Result<String, SnbtError> {
        let start = self.pos;
        self.expect(quote)?;

        let mut result = String::new();
        loop {
            match self.advance() {
                None => return Err(SnbtError::UnterminatedString { pos: start }),
                Some(b'\\') => {
                    // Escape sequence
                    match self.advance() {
                        None => return Err(SnbtError::UnterminatedString { pos: start }),
                        Some(b'"') => result.push('"'),
                        Some(b'\'') => result.push('\''),
                        Some(b'\\') => result.push('\\'),
                        Some(b'n') => result.push('\n'),
                        Some(b't') => result.push('\t'),
                        Some(b'r') => result.push('\r'),
                        Some(ch) => {
                            return Err(SnbtError::InvalidEscape {
                                ch: ch as char,
                                pos: self.pos - 1,
                            });
                        }
                    }
                }
                Some(ch) if ch == quote => return Ok(result),
                Some(ch) => result.push(ch as char),
            }
        }
    }

    fn parse_unquoted_string(&mut self) -> Result<String, SnbtError> {
        let start = self.pos;
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            // Unquoted strings can contain alphanumeric, underscore, hyphen, dot, plus
            if ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'-' || ch == b'.' || ch == b'+' {
                self.pos += 1;
            } else {
                break;
            }
        }

        if self.pos == start {
            return Err(match self.peek() {
                Some(ch) => SnbtError::UnexpectedChar {
                    ch: ch as char,
                    pos: self.pos,
                },
                None => SnbtError::UnexpectedEof,
            });
        }

        Ok(std::str::from_utf8(&self.input[start..self.pos])
            .unwrap()
            .to_string())
    }

    fn parse_list_or_array(&mut self) -> Result<NbtTag, SnbtError> {
        // Peek ahead to check for array prefix: [B;  [I;  [L;
        self.expect(b'[')?;
        self.skip_whitespace();

        if self.pos + 1 < self.input.len() {
            let type_char = self.input[self.pos];
            // Check for whitespace between type char and semicolon
            let mut check_pos = self.pos + 1;
            while check_pos < self.input.len() && self.input[check_pos].is_ascii_whitespace() {
                check_pos += 1;
            }
            if check_pos < self.input.len() && self.input[check_pos] == b';' {
                match type_char {
                    b'B' => {
                        self.pos = check_pos + 1;
                        return self.parse_byte_array();
                    }
                    b'I' => {
                        self.pos = check_pos + 1;
                        return self.parse_int_array();
                    }
                    b'L' => {
                        self.pos = check_pos + 1;
                        return self.parse_long_array();
                    }
                    _ => {}
                }
            }
        }

        // It's a regular list
        self.parse_list()
    }

    fn parse_byte_array(&mut self) -> Result<NbtTag, SnbtError> {
        self.skip_whitespace();
        let mut values = Vec::new();

        if self.peek() == Some(b']') {
            self.advance();
            return Ok(NbtTag::ByteArray(values.into_boxed_slice()));
        }

        loop {
            self.skip_whitespace();
            let value = self.parse_byte_value()?;
            values.push(value as u8);

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.advance();
                }
                Some(b']') => {
                    self.advance();
                    return Ok(NbtTag::ByteArray(values.into_boxed_slice()));
                }
                Some(ch) => {
                    return Err(SnbtError::Expected {
                        expected: ']',
                        found: Some(ch as char),
                        pos: self.pos,
                    });
                }
                None => return Err(SnbtError::UnexpectedEof),
            }
        }
    }

    fn parse_byte_value(&mut self) -> Result<i8, SnbtError> {
        // Check for boolean values
        let start = self.pos;
        let text = self.collect_number_text();

        if text == "true" {
            return Ok(1);
        }
        if text == "false" {
            return Ok(0);
        }

        // Strip trailing 'b' or 'B'
        let num_text = if text.ends_with('b') || text.ends_with('B') {
            &text[..text.len() - 1]
        } else {
            &text
        };

        num_text
            .parse::<i8>()
            .map_err(|_| SnbtError::InvalidNumber {
                text: text.clone(),
                pos: start,
            })
    }

    fn parse_int_array(&mut self) -> Result<NbtTag, SnbtError> {
        self.skip_whitespace();
        let mut values = Vec::new();

        if self.peek() == Some(b']') {
            self.advance();
            return Ok(NbtTag::IntArray(values));
        }

        loop {
            self.skip_whitespace();
            let start = self.pos;
            let text = self.collect_number_text();
            let value = text.parse::<i32>().map_err(|_| SnbtError::InvalidNumber {
                text: text.clone(),
                pos: start,
            })?;
            values.push(value);

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.advance();
                }
                Some(b']') => {
                    self.advance();
                    return Ok(NbtTag::IntArray(values));
                }
                Some(ch) => {
                    return Err(SnbtError::Expected {
                        expected: ']',
                        found: Some(ch as char),
                        pos: self.pos,
                    });
                }
                None => return Err(SnbtError::UnexpectedEof),
            }
        }
    }

    fn parse_long_array(&mut self) -> Result<NbtTag, SnbtError> {
        self.skip_whitespace();
        let mut values = Vec::new();

        if self.peek() == Some(b']') {
            self.advance();
            return Ok(NbtTag::LongArray(values));
        }

        loop {
            self.skip_whitespace();
            let start = self.pos;
            let text = self.collect_number_text();

            // Strip trailing 'l' or 'L'
            let num_text = if text.ends_with('l') || text.ends_with('L') {
                &text[..text.len() - 1]
            } else {
                &text
            };

            let value = num_text
                .parse::<i64>()
                .map_err(|_| SnbtError::InvalidNumber {
                    text: text.clone(),
                    pos: start,
                })?;
            values.push(value);

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.advance();
                }
                Some(b']') => {
                    self.advance();
                    return Ok(NbtTag::LongArray(values));
                }
                Some(ch) => {
                    return Err(SnbtError::Expected {
                        expected: ']',
                        found: Some(ch as char),
                        pos: self.pos,
                    });
                }
                None => return Err(SnbtError::UnexpectedEof),
            }
        }
    }

    fn parse_list(&mut self) -> Result<NbtTag, SnbtError> {
        self.skip_whitespace();
        let mut values = Vec::new();

        if self.peek() == Some(b']') {
            self.advance();
            return Ok(NbtTag::List(values));
        }

        loop {
            let value = self.parse_value()?;
            values.push(value);

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.advance();
                }
                Some(b']') => {
                    self.advance();
                    return Ok(NbtTag::List(values));
                }
                Some(ch) => {
                    return Err(SnbtError::Expected {
                        expected: ']',
                        found: Some(ch as char),
                        pos: self.pos,
                    });
                }
                None => return Err(SnbtError::UnexpectedEof),
            }
        }
    }

    /// Collect characters that could be part of a number or unquoted string.
    fn collect_number_text(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'-' || ch == b'.' || ch == b'+' {
                self.pos += 1;
            } else {
                break;
            }
        }
        std::str::from_utf8(&self.input[start..self.pos])
            .unwrap()
            .to_string()
    }

    fn parse_primitive(&mut self) -> Result<NbtTag, SnbtError> {
        let text = self.collect_number_text();

        if text.is_empty() {
            return Err(match self.peek() {
                Some(ch) => SnbtError::UnexpectedChar {
                    ch: ch as char,
                    pos: self.pos,
                },
                None => SnbtError::UnexpectedEof,
            });
        }

        // Boolean literals
        if text == "true" {
            return Ok(NbtTag::Byte(1));
        }
        if text == "false" {
            return Ok(NbtTag::Byte(0));
        }

        let last_char = text.as_bytes()[text.len() - 1];

        // Byte: suffix b or B
        if last_char == b'b' || last_char == b'B' {
            let num_text = &text[..text.len() - 1];
            if let Ok(v) = num_text.parse::<i8>() {
                return Ok(NbtTag::Byte(v));
            }
        }

        // Short: suffix s or S
        if last_char == b's' || last_char == b'S' {
            let num_text = &text[..text.len() - 1];
            if let Ok(v) = num_text.parse::<i16>() {
                return Ok(NbtTag::Short(v));
            }
        }

        // Long: suffix l or L
        if last_char == b'l' || last_char == b'L' {
            let num_text = &text[..text.len() - 1];
            if let Ok(v) = num_text.parse::<i64>() {
                return Ok(NbtTag::Long(v));
            }
        }

        // Float: suffix f or F
        if last_char == b'f' || last_char == b'F' {
            let num_text = &text[..text.len() - 1];
            if let Ok(v) = num_text.parse::<f32>() {
                return Ok(NbtTag::Float(v));
            }
        }

        // Double: suffix d or D
        if last_char == b'd' || last_char == b'D' {
            let num_text = &text[..text.len() - 1];
            if let Ok(v) = num_text.parse::<f64>() {
                return Ok(NbtTag::Double(v));
            }
        }

        // Integer (no suffix, no decimal point)
        if !text.contains('.')
            && let Ok(v) = text.parse::<i32>()
        {
            return Ok(NbtTag::Int(v));
        }

        // Double (no suffix, has decimal point)
        if text.contains('.')
            && let Ok(v) = text.parse::<f64>()
        {
            return Ok(NbtTag::Double(v));
        }

        // Fall back to unquoted string
        Ok(NbtTag::String(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Primitive types ---

    #[test]
    fn parse_byte() {
        assert_eq!(from_snbt("42b").unwrap(), NbtTag::Byte(42));
        assert_eq!(from_snbt("-1B").unwrap(), NbtTag::Byte(-1));
        assert_eq!(from_snbt("0b").unwrap(), NbtTag::Byte(0));
        assert_eq!(from_snbt("127b").unwrap(), NbtTag::Byte(127));
        assert_eq!(from_snbt("-128b").unwrap(), NbtTag::Byte(-128));
    }

    #[test]
    fn parse_boolean() {
        assert_eq!(from_snbt("true").unwrap(), NbtTag::Byte(1));
        assert_eq!(from_snbt("false").unwrap(), NbtTag::Byte(0));
    }

    #[test]
    fn parse_short() {
        assert_eq!(from_snbt("1342s").unwrap(), NbtTag::Short(1342));
        assert_eq!(from_snbt("-100S").unwrap(), NbtTag::Short(-100));
        assert_eq!(from_snbt("0s").unwrap(), NbtTag::Short(0));
    }

    #[test]
    fn parse_int() {
        assert_eq!(from_snbt("4313").unwrap(), NbtTag::Int(4313));
        assert_eq!(from_snbt("-999").unwrap(), NbtTag::Int(-999));
        assert_eq!(from_snbt("0").unwrap(), NbtTag::Int(0));
        assert_eq!(from_snbt("2147483647").unwrap(), NbtTag::Int(i32::MAX));
        assert_eq!(from_snbt("-2147483648").unwrap(), NbtTag::Int(i32::MIN));
    }

    #[test]
    fn parse_long() {
        assert_eq!(from_snbt("34L").unwrap(), NbtTag::Long(34));
        assert_eq!(from_snbt("-500l").unwrap(), NbtTag::Long(-500));
        assert_eq!(from_snbt("0L").unwrap(), NbtTag::Long(0));
    }

    #[test]
    fn parse_float() {
        assert_eq!(from_snbt("1.0f").unwrap(), NbtTag::Float(1.0));
        assert_eq!(from_snbt("-69.42F").unwrap(), NbtTag::Float(-69.42));
        assert_eq!(from_snbt("0.0f").unwrap(), NbtTag::Float(0.0));
    }

    #[test]
    fn parse_double() {
        assert_eq!(from_snbt("1.5d").unwrap(), NbtTag::Double(1.5));
        assert_eq!(from_snbt("-3.15D").unwrap(), NbtTag::Double(-3.15));
        // Bare decimal is double
        assert_eq!(from_snbt("2.719").unwrap(), NbtTag::Double(2.719));
    }

    // --- Strings ---

    #[test]
    fn parse_double_quoted_string() {
        assert_eq!(
            from_snbt("\"hello world\"").unwrap(),
            NbtTag::String("hello world".to_string())
        );
    }

    #[test]
    fn parse_single_quoted_string() {
        assert_eq!(
            from_snbt("'hello world'").unwrap(),
            NbtTag::String("hello world".to_string())
        );
    }

    #[test]
    fn parse_string_with_escapes() {
        assert_eq!(
            from_snbt("\"hello\\nworld\"").unwrap(),
            NbtTag::String("hello\nworld".to_string())
        );
        assert_eq!(
            from_snbt("\"tab\\there\"").unwrap(),
            NbtTag::String("tab\there".to_string())
        );
        assert_eq!(
            from_snbt("\"escaped\\\\backslash\"").unwrap(),
            NbtTag::String("escaped\\backslash".to_string())
        );
        assert_eq!(
            from_snbt("\"quote\\\"inside\"").unwrap(),
            NbtTag::String("quote\"inside".to_string())
        );
    }

    #[test]
    fn parse_unquoted_string() {
        // Unquoted strings that don't match any numeric pattern
        assert_eq!(
            from_snbt("minecraft_stone").unwrap(),
            NbtTag::String("minecraft_stone".to_string())
        );
    }

    // --- Arrays ---

    #[test]
    fn parse_byte_array() {
        assert_eq!(
            from_snbt("[B; 1b, 2b, 3b]").unwrap(),
            NbtTag::ByteArray(vec![1, 2, 3].into_boxed_slice())
        );
    }

    #[test]
    fn parse_byte_array_empty() {
        assert_eq!(
            from_snbt("[B;]").unwrap(),
            NbtTag::ByteArray(vec![].into_boxed_slice())
        );
    }

    #[test]
    fn parse_int_array() {
        assert_eq!(
            from_snbt("[I; 100, 200, 300]").unwrap(),
            NbtTag::IntArray(vec![100, 200, 300])
        );
    }

    #[test]
    fn parse_int_array_empty() {
        assert_eq!(from_snbt("[I;]").unwrap(), NbtTag::IntArray(vec![]));
    }

    #[test]
    fn parse_long_array() {
        assert_eq!(
            from_snbt("[L; 1L, 2L, 3L]").unwrap(),
            NbtTag::LongArray(vec![1, 2, 3])
        );
    }

    #[test]
    fn parse_long_array_empty() {
        assert_eq!(from_snbt("[L;]").unwrap(), NbtTag::LongArray(vec![]));
    }

    // --- Lists ---

    #[test]
    fn parse_list_of_ints() {
        assert_eq!(
            from_snbt("[1, 2, 3]").unwrap(),
            NbtTag::List(vec![NbtTag::Int(1), NbtTag::Int(2), NbtTag::Int(3)])
        );
    }

    #[test]
    fn parse_list_of_strings() {
        assert_eq!(
            from_snbt("[\"a\", \"b\"]").unwrap(),
            NbtTag::List(vec![
                NbtTag::String("a".to_string()),
                NbtTag::String("b".to_string()),
            ])
        );
    }

    #[test]
    fn parse_empty_list() {
        assert_eq!(from_snbt("[]").unwrap(), NbtTag::List(vec![]));
    }

    #[test]
    fn parse_nested_list() {
        assert_eq!(
            from_snbt("[[1, 2], [3, 4]]").unwrap(),
            NbtTag::List(vec![
                NbtTag::List(vec![NbtTag::Int(1), NbtTag::Int(2)]),
                NbtTag::List(vec![NbtTag::Int(3), NbtTag::Int(4)]),
            ])
        );
    }

    // --- Compounds ---

    #[test]
    fn parse_compound_simple() {
        let tag = from_snbt("{x: 1, y: 2, z: 3}").unwrap();
        if let NbtTag::Compound(compound) = tag {
            assert_eq!(compound.get_int("x"), Some(1));
            assert_eq!(compound.get_int("y"), Some(2));
            assert_eq!(compound.get_int("z"), Some(3));
        } else {
            panic!("Expected compound");
        }
    }

    #[test]
    fn parse_compound_empty() {
        let tag = from_snbt("{}").unwrap();
        if let NbtTag::Compound(compound) = tag {
            assert!(compound.is_empty());
        } else {
            panic!("Expected compound");
        }
    }

    #[test]
    fn parse_compound_quoted_keys() {
        let tag = from_snbt("{\"key with spaces\": 42}").unwrap();
        if let NbtTag::Compound(compound) = tag {
            assert_eq!(compound.get_int("key with spaces"), Some(42));
        } else {
            panic!("Expected compound");
        }
    }

    #[test]
    fn parse_compound_nested() {
        let tag = from_snbt("{outer: {inner: 1}}").unwrap();
        if let NbtTag::Compound(compound) = tag {
            let outer = compound.get_compound("outer").unwrap();
            assert_eq!(outer.get_int("inner"), Some(1));
        } else {
            panic!("Expected compound");
        }
    }

    #[test]
    fn parse_compound_mixed_types() {
        let tag = from_snbt("{name: \"Steve\", health: 20.0f, level: 5, alive: true}").unwrap();
        if let NbtTag::Compound(compound) = tag {
            assert_eq!(compound.get_string("name"), Some("Steve"));
            assert_eq!(compound.get_float("health"), Some(20.0));
            assert_eq!(compound.get_int("level"), Some(5));
            assert_eq!(compound.get_bool("alive"), Some(true));
        } else {
            panic!("Expected compound");
        }
    }

    // --- from_snbt_compound ---

    #[test]
    fn parse_snbt_compound_convenience() {
        let compound = from_snbt_compound("{a: 1, b: 2}").unwrap();
        assert_eq!(compound.get_int("a"), Some(1));
        assert_eq!(compound.get_int("b"), Some(2));
    }

    // --- Whitespace handling ---

    #[test]
    fn parse_with_extra_whitespace() {
        assert_eq!(from_snbt("  42  ").unwrap(), NbtTag::Int(42));
        assert_eq!(
            from_snbt("  {  x  :  1  ,  y  :  2  }  ").unwrap(),
            from_snbt("{x:1,y:2}").unwrap()
        );
    }

    // --- Error cases ---

    #[test]
    fn error_empty_input() {
        assert!(from_snbt("").is_err());
    }

    #[test]
    fn error_unterminated_string() {
        assert!(from_snbt("\"hello").is_err());
    }

    #[test]
    fn error_unterminated_compound() {
        assert!(from_snbt("{x: 1").is_err());
    }

    #[test]
    fn error_unterminated_list() {
        assert!(from_snbt("[1, 2").is_err());
    }

    #[test]
    fn error_trailing_data() {
        assert!(from_snbt("42 extra").is_err());
    }

    #[test]
    fn error_invalid_escape() {
        assert!(from_snbt("\"hello\\x\"").is_err());
    }

    // --- Display roundtrip ---

    #[test]
    fn display_then_parse_roundtrip() {
        let original = NbtTag::Compound({
            let mut c = NbtCompound::new();
            c.put_int("x", 42);
            c.put_string("name", "test".to_string());
            c.put_byte("b", 1);
            c.put_long("l", 100);
            c.put_float("f", 3.15);
            c.put_double("d", 2.719);
            c
        });

        let snbt = format!("{original}");
        let parsed = from_snbt(&snbt).unwrap();

        if let (NbtTag::Compound(orig), NbtTag::Compound(pars)) = (&original, &parsed) {
            assert_eq!(orig.get_int("x"), pars.get_int("x"));
            assert_eq!(orig.get_string("name"), pars.get_string("name"));
            assert_eq!(orig.get_byte("b"), pars.get_byte("b"));
            assert_eq!(orig.get_long("l"), pars.get_long("l"));
            // Floats need approximate comparison due to display precision
        } else {
            panic!("Expected compounds");
        }
    }

    #[test]
    fn parse_negative_numbers_in_arrays() {
        assert_eq!(
            from_snbt("[B; -1b, -128b]").unwrap(),
            NbtTag::ByteArray(vec![255, 128].into_boxed_slice()) // -1i8 as u8 = 255, -128i8 as u8 = 128
        );
        assert_eq!(
            from_snbt("[I; -1, -2147483648]").unwrap(),
            NbtTag::IntArray(vec![-1, i32::MIN])
        );
        assert_eq!(
            from_snbt("[L; -1L, -9223372036854775808L]").unwrap(),
            NbtTag::LongArray(vec![-1, i64::MIN])
        );
    }

    // --- Display escape roundtrip tests ---

    #[test]
    fn display_escape_roundtrip_backslash() {
        let tag = NbtTag::String("back\\slash".to_string());
        let snbt = format!("{tag}");
        assert_eq!(snbt, "\"back\\\\slash\"");
        let parsed = from_snbt(&snbt).unwrap();
        assert_eq!(parsed, tag);
    }

    #[test]
    fn display_escape_roundtrip_quotes() {
        let tag = NbtTag::String("say \"hello\"".to_string());
        let snbt = format!("{tag}");
        assert_eq!(snbt, "\"say \\\"hello\\\"\"");
        let parsed = from_snbt(&snbt).unwrap();
        assert_eq!(parsed, tag);
    }

    #[test]
    fn display_escape_roundtrip_newlines() {
        let tag = NbtTag::String("line1\nline2\ttab\rcarriage".to_string());
        let snbt = format!("{tag}");
        assert_eq!(snbt, "\"line1\\nline2\\ttab\\rcarriage\"");
        let parsed = from_snbt(&snbt).unwrap();
        assert_eq!(parsed, tag);
    }

    #[test]
    fn display_escape_compound_with_special_strings() {
        let mut c = NbtCompound::new();
        c.put_string("msg", "hello \"world\"\nnewline".to_string());
        c.put_string("path", "C:\\Users\\test".to_string());
        let tag = NbtTag::Compound(c);

        let snbt = format!("{tag}");
        let parsed = from_snbt(&snbt).unwrap();

        if let (NbtTag::Compound(orig), NbtTag::Compound(pars)) = (&tag, &parsed) {
            assert_eq!(orig.get_string("msg"), pars.get_string("msg"));
            assert_eq!(orig.get_string("path"), pars.get_string("path"));
        } else {
            panic!("Expected compounds");
        }
    }

    #[test]
    fn display_array_roundtrip() {
        let tag = NbtTag::ByteArray(vec![1, 2, 255].into_boxed_slice());
        let snbt = format!("{tag}");
        // ByteArray Display shows signed: 1b, 2b, -1b (255 as i8 = -1)
        let parsed = from_snbt(&snbt).unwrap();
        // Should match: -1i8 as u8 = 255
        assert_eq!(parsed, tag);
    }

    #[test]
    fn display_int_array_roundtrip() {
        let tag = NbtTag::IntArray(vec![-1, 0, i32::MAX, i32::MIN]);
        let snbt = format!("{tag}");
        let parsed = from_snbt(&snbt).unwrap();
        assert_eq!(parsed, tag);
    }

    #[test]
    fn display_long_array_roundtrip() {
        let tag = NbtTag::LongArray(vec![-1, 0, i64::MAX, i64::MIN]);
        let snbt = format!("{tag}");
        let parsed = from_snbt(&snbt).unwrap();
        assert_eq!(parsed, tag);
    }
}
