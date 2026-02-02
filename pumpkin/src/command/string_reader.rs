use crate::command::errors::command_syntax_error::{
    CommandSyntaxError, CommandSyntaxErrorContext, ContextProvider,
};
use crate::command::errors::error_types;
use crate::command::errors::error_types::{DynamicCommandErrorType, SimpleCommandErrorType};
use pumpkin_util::text::TextComponent;
use std::borrow::Cow;
use std::str::FromStr;

/// A structure that can read a string character by character for commands.
///
/// It internally uses a cursor to read them, which is
/// very important to determine the location of the cause
/// of a syntax error arising from this parser.
#[derive(Clone, Debug)]
pub struct StringReader<'a> {
    string: Cow<'a, str>,
    byte_cursor: usize,
}

const SYNTAX_ESCAPE: char = '\\';
const SYNTAX_SINGLE_QUOTE: char = '\'';
const SYNTAX_DOUBLE_QUOTE: char = '"';

impl<'a> StringReader<'a> {
    /// Returns a new instance of a [`StringReader`]
    /// from a borrowed or owned string, with the cursor
    /// initially being set to the start.
    pub fn new<S>(string: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        Self {
            string: string.into(),
            byte_cursor: 0,
        }
    }

    /// Returns a reference to the internal string used for parsing.
    #[must_use]
    pub fn string(&self) -> &str {
        &self.string
    }

    /// Returns the current byte position of the internal cursor.
    #[must_use]
    pub const fn cursor(&self) -> usize {
        self.byte_cursor
    }

    /// Sets the current byte position of the internal cursor.
    pub const fn set_cursor(&mut self, cursor: usize) {
        self.byte_cursor = cursor;
    }

    /// Gets the total byte length of the internal string.
    #[must_use]
    pub fn total_length(&self) -> usize {
        self.string.len()
    }

    /// Gets the remaining, unread byte length of the internal string.
    #[must_use]
    pub fn remaining_length(&self) -> usize {
        self.string.len() - self.byte_cursor
    }

    /// Gets the read part of the internal string.
    #[must_use]
    pub fn read_part(&self) -> &str {
        &self.string[0..self.byte_cursor]
    }

    /// Gets the remaining, unread part of the internal string.
    #[must_use]
    pub fn remaining_part(&self) -> &str {
        &self.string[self.byte_cursor..]
    }

    /// Returns whether the reader is able to read `length` more bytes
    /// without going out of bounds.
    #[must_use]
    pub fn can_read_bytes(&self, length: usize) -> bool {
        self.byte_cursor + length <= self.string.len()
    }

    /// Returns whether the reader is able to read 1 more byte
    /// without going out of bounds.
    #[must_use]
    pub fn can_read_byte(&self) -> bool {
        self.can_read_bytes(1)
    }

    /// Returns whether the reader is able to read `length` more [`char`]s
    /// without going out of bounds.
    #[must_use]
    pub fn can_read_chars(&self, length: usize) -> bool {
        self.string[self.byte_cursor..].chars().take(length).count() == length
    }

    /// Returns whether the reader is able to read 1 more [`char`]
    /// without going out of bounds.
    #[must_use]
    pub fn can_read_char(&self) -> bool {
        self.can_read_chars(1)
    }

    /// Peeks a byte, where the cursor is at, without advancing.
    #[must_use]
    pub fn peek_byte(&self) -> Option<u8> {
        self.byte_at(self.byte_cursor)
    }

    /// Fetches a byte, at a specific byte-index.
    #[must_use]
    pub fn byte_at(&self, i: usize) -> Option<u8> {
        self.string.as_bytes().get(i).copied()
    }

    /// Peeks a [`char`], where the cursor is at, without advancing.
    #[must_use]
    pub fn peek(&self) -> Option<char> {
        self.string[self.byte_cursor..].chars().next()
    }

    /// Reads a [`char`], where the cursor is at, before advancing.
    #[must_use = "to skip a character use `skip()` instead"]
    pub fn read(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.byte_cursor += c.len_utf8();
        Some(c)
    }

    /// Skips a [`char`], where the cursor is at.
    pub fn skip(&mut self) {
        if let Some(c) = self.peek() {
            self.byte_cursor += c.len_utf8();
        }
    }

    /// Whether the given [`char`] is allowed in a number.
    #[must_use]
    pub const fn is_allowed_in_number(c: char) -> bool {
        matches!(c, '0'..='9' | '.' | '-')
    }

    /// Whether the given [`char`] is allowed as the start and the end of a quoted string.
    #[must_use]
    pub const fn is_allowed_as_quoted_string_start_end(c: char) -> bool {
        matches!(c, SYNTAX_SINGLE_QUOTE | SYNTAX_DOUBLE_QUOTE)
    }

    /// Whether the given [`char`] is allowed in an unquoted string.
    #[must_use]
    pub const fn is_allowed_in_unquoted_string(c: char) -> bool {
        matches!(c, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_' | '-' | '.' | '+')
    }

    /// Skips any kind of whitespace until there isn't more of it to skip.
    pub fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.skip();
            } else {
                break;
            }
        }
    }

    /// Parses a given type [`T`].
    fn read_and_parse<T: FromStr>(
        &mut self,
        expected_error_type: SimpleCommandErrorType,
        invalid_error_type: DynamicCommandErrorType<1>,
    ) -> Result<T, CommandSyntaxError> {
        let start = self.byte_cursor;
        while let Some(c) = self.peek() {
            if Self::is_allowed_in_number(c) {
                self.skip();
            } else {
                break;
            }
        }
        let result = &self.string[start..self.byte_cursor];
        if result.is_empty() {
            return Err(expected_error_type.instance_with_context(self));
        }
        if let Ok(value) = result.parse::<T>() {
            Ok(value)
        } else {
            self.byte_cursor = start;
            Err(invalid_error_type
                .instance_with_context(self, &[TextComponent::text(result.to_owned())]))
        }
    }

    /// Parses a [`bool`], resulting in an [`Err`] if unsuccessful.
    pub fn read_bool(&mut self) -> Result<bool, CommandSyntaxError> {
        let start = self.byte_cursor;
        let value = self.read_string()?;
        match value.as_str() {
            "" => Err(error_types::READER_EXPECTED_BOOL.instance_with_context(self)),
            "true" => Ok(true),
            "false" => Ok(false),
            _ => {
                self.byte_cursor = start;
                Err(error_types::READER_INVALID_BOOL
                    .instance_with_context(self, &[TextComponent::text(value.clone())]))
            }
        }
    }

    /// Parses a [`i32`], resulting in an [`Err`] if unsuccessful.
    pub fn read_int(&mut self) -> Result<i32, CommandSyntaxError> {
        self.read_and_parse(
            error_types::READER_EXPECTED_INT,
            error_types::READER_INVALID_INT,
        )
    }

    /// Parses a [`i64`], resulting in an [`Err`] if unsuccessful.
    pub fn read_long(&mut self) -> Result<i64, CommandSyntaxError> {
        self.read_and_parse(
            error_types::READER_EXPECTED_LONG,
            error_types::READER_INVALID_LONG,
        )
    }

    /// Parses a [`f32`], resulting in an [`Err`] if unsuccessful.
    pub fn read_float(&mut self) -> Result<f32, CommandSyntaxError> {
        self.read_and_parse(
            error_types::READER_EXPECTED_FLOAT,
            error_types::READER_INVALID_FLOAT,
        )
    }

    /// Parses a [`f64`], resulting in an [`Err`] if unsuccessful.
    pub fn read_double(&mut self) -> Result<f64, CommandSyntaxError> {
        self.read_and_parse(
            error_types::READER_EXPECTED_DOUBLE,
            error_types::READER_INVALID_DOUBLE,
        )
    }

    /// Reads an unquoted string (not enclosed in quotes)
    pub fn read_unquoted_string(&mut self) -> Result<String, CommandSyntaxError> {
        let start = self.byte_cursor;
        while let Some(c) = self.peek() {
            if Self::is_allowed_in_unquoted_string(c) {
                self.skip();
            } else {
                break;
            }
        }
        Ok(self.string[start..self.byte_cursor].to_string())
    }

    /// Reads any string, whether it be quoted or unquoted.
    pub fn read_string(&mut self) -> Result<String, CommandSyntaxError> {
        let Some(next) = self.peek() else {
            return Ok(String::new());
        };
        if Self::is_allowed_as_quoted_string_start_end(next) {
            self.skip();
            self.read_string_until(next)
        } else {
            self.read_unquoted_string()
        }
    }

    /// Reads a quoted string (enclosed in quotes)
    pub fn read_quoted_string(&mut self) -> Result<String, CommandSyntaxError> {
        let Some(next) = self.peek() else {
            return Ok(String::new());
        };
        if Self::is_allowed_as_quoted_string_start_end(next) {
            self.skip();
            self.read_string_until(next)
        } else {
            Err(error_types::READER_EXPECTED_START_QUOTE.instance_with_context(self))
        }
    }

    /// Reads a string until the given character.
    pub fn read_string_until(&mut self, terminator: char) -> Result<String, CommandSyntaxError> {
        let mut result: String = String::new();
        let mut escaped: bool = false;
        while let Some(c) = self.peek() {
            if escaped {
                if c == terminator || c == SYNTAX_ESCAPE {
                    result.push(c);
                    self.skip();
                    escaped = false;
                } else {
                    return Err(error_types::READER_INVALID_ESCAPE
                        .instance_with_context(self, &[TextComponent::text(c.to_string())]));
                }
            } else {
                self.skip();
                if c == SYNTAX_ESCAPE {
                    escaped = true;
                } else if c == terminator {
                    return Ok(result);
                } else {
                    result.push(c);
                }
            }
        }
        Err(error_types::READER_EXPECTED_END_QUOTE.instance_with_context(self))
    }

    /// Expects to consume a specific [`char`], or return an [`Err`].
    pub fn expect(&mut self, c: char) -> Result<(), CommandSyntaxError> {
        if self.peek() == Some(c) {
            self.skip();
            Ok(())
        } else {
            Err(error_types::READER_EXPECTED_SYMBOL
                .instance_with_context(self, &[TextComponent::text(c.to_string())]))
        }
    }
}

impl ContextProvider for StringReader<'_> {
    fn context(&self) -> CommandSyntaxErrorContext {
        CommandSyntaxErrorContext {
            input: self.string.to_string(),
            cursor: self.byte_cursor,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::command::string_reader::StringReader;

    #[test]
    fn non_parsing_methods() {
        let mut reader = StringReader::new("hello 🎃! ");
        assert_eq!(reader.read(), Some('h'));

        assert_eq!(reader.peek(), Some('e'));
        assert_eq!(reader.read(), Some('e'));

        assert!(reader.can_read_char());
        assert!(reader.can_read_chars(7));
        assert!(!reader.can_read_chars(8));

        reader.expect('l').unwrap();
        reader.skip();
        reader.expect('o').unwrap();

        // Note: 🎃 carries 4 bytes in UTF-8
        assert!(reader.can_read_bytes(6));
        assert_eq!(reader.remaining_length(), 7);
        assert!(!reader.can_read_bytes(8));

        assert_eq!(reader.read_part(), "hello");
        reader.set_cursor(6);
        assert_eq!(reader.remaining_part(), "🎃! ");
        assert_eq!(reader.peek_byte(), Some(0xF0));
        assert_eq!(reader.read(), Some('🎃'));

        reader.skip_whitespace(); // Should be a NO-OP
        assert_eq!(reader.read(), Some('!'));

        reader.skip_whitespace();
        assert_ne!(reader.expect(' '), Ok(()));
    }

    #[test]
    fn read_types() {
        let mut reader =
            StringReader::new("12 34  7890123456     1.233   1.592394582  false'false' faux");
        assert_eq!(reader.read_int(), Ok(12));
        reader.skip_whitespace();
        assert_eq!(reader.read_long(), Ok(34));
        reader.skip_whitespace();

        assert!(reader.read_int().is_err());
        reader.skip_whitespace();
        assert_eq!(reader.read_long(), Ok(7890123456));
        reader.skip_whitespace();

        assert!((reader.read_float().unwrap() - 1.233f32).abs() < 1e-07);
        reader.skip_whitespace();

        assert!((reader.read_double().unwrap() - 1.592394582f64).abs() < 1e-15);
        reader.skip_whitespace();

        assert_eq!(reader.read_bool(), Ok(false));
        assert_eq!(reader.read_bool(), Ok(false));
        reader.skip_whitespace();
        assert_ne!(reader.read_bool(), Ok(false));
    }

    #[test]
    fn read_strings() {
        let mut reader = StringReader::new("'apple' banana orange \"orange\" 'hello\"");

        assert_eq!(reader.read_quoted_string(), Ok("apple".to_string()));
        reader.skip_whitespace();
        assert_eq!(reader.read_unquoted_string(), Ok("banana".to_string()));
        reader.skip_whitespace();

        assert_eq!(reader.read_string(), Ok("orange".to_string()));
        reader.skip_whitespace();
        assert_eq!(reader.read_string(), Ok("orange".to_string()));
        reader.skip_whitespace();

        assert!(reader.read_quoted_string().is_err());
    }
}
