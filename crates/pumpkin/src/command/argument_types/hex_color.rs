use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::{CommandErrorType, DISPATCHER_PARSE_EXCEPTION};
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_data::translation::java::ARGUMENT_HEXCOLOR_INVALID;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::RGBColor;
use std::pin::Pin;

pub const INVALID_HEX_ERROR_TYPE: CommandErrorType<1> =
    CommandErrorType::new(ARGUMENT_HEXCOLOR_INVALID, ARGUMENT_HEXCOLOR_INVALID);

pub struct HexColorArgumentType;

impl ArgumentType for HexColorArgumentType {
    type Item = RGBColor;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let string = reader.read_unquoted_string();
        match string.len() {
            3 => Ok(RGBColor {
                red: Self::parse_one_hex_digit(reader, &string, 0)?,
                green: Self::parse_one_hex_digit(reader, &string, 1)?,
                blue: Self::parse_one_hex_digit(reader, &string, 2)?,
            }),
            6 => Ok(RGBColor {
                red: Self::parse_two_hex_digits(reader, &string, 0)?,
                green: Self::parse_two_hex_digits(reader, &string, 2)?,
                blue: Self::parse_two_hex_digits(reader, &string, 4)?,
            }),
            _ => Err(INVALID_HEX_ERROR_TYPE.create(reader, TextComponent::text(string))),
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::HexColor
    }

    fn list_suggestions<'a>(
        &'a self,
        _context: &'a CommandContext,
        builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send + 'a>> {
        Box::pin(async move { builder.filter_and_suggest(&["F00", "FF0000"]).build() })
    }

    fn examples(&self) -> Vec<String> {
        examples!("F00", "FF0000")
    }
}

impl HexColorArgumentType {
    fn parse_one_hex_digit(
        reader: &StringReader,
        string: &str,
        index: usize,
    ) -> Result<u8, CommandSyntaxError> {
        Self::parse_hex_digit(reader, &string[index..=index], 0)
            // We want to map:
            // 0x0 -> 0x00
            // 0x1 -> 0x11
            // 0x2 -> 0x22 ...
            .map(|x| x * 0x11)
    }

    fn parse_two_hex_digits(
        reader: &StringReader,
        string: &str,
        index_start: usize,
    ) -> Result<u8, CommandSyntaxError> {
        Ok(
            Self::parse_hex_digit(reader, &string[index_start..index_start + 2], 0)? << 4
                | Self::parse_hex_digit(reader, &string[index_start..index_start + 2], 1)?,
        )
    }

    #[inline]
    fn parse_hex_digit(
        reader: &StringReader,
        slice: &str,
        index: usize,
    ) -> Result<u8, CommandSyntaxError> {
        // This should be fine as `read_unquoted_string` parses only ASCII characters.
        let b = slice.as_bytes()[index];
        match b {
            b'0' => Ok(0x0),
            b'1' => Ok(0x1),
            b'2' => Ok(0x2),
            b'3' => Ok(0x3),
            b'4' => Ok(0x4),
            b'5' => Ok(0x5),
            b'6' => Ok(0x6),
            b'7' => Ok(0x7),
            b'8' => Ok(0x8),
            b'9' => Ok(0x9),
            b'a' | b'A' => Ok(0xA),
            b'b' | b'B' => Ok(0xB),
            b'c' | b'C' => Ok(0xC),
            b'd' | b'D' => Ok(0xD),
            b'e' | b'E' => Ok(0xE),
            b'f' | b'F' => Ok(0xF),
            _ => Err(DISPATCHER_PARSE_EXCEPTION.create(
                reader,
                TextComponent::text(format!("Error at index {index} in: \"{slice}\"")),
            )),
        }
    }
}

impl_copy_get!(HexColorArgumentType, RGBColor);
