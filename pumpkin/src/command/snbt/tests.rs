use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::text::TextComponent;

const BUILT_IN_LIKE_SUGGESTIONS: &[&str] = &["(", "bool", "false", "true", "uuid"];

use crate::command::{
    errors::{
        command_syntax_error::CommandSyntaxError,
        error_types::{CommandErrorType, LITERAL_INCORRECT},
    },
    snbt::{NUMBER_PARSE_FAILURE, SnbtParser},
    string_reader::StringReader,
    suggestion::suggestions::SuggestionsBuilder,
};

fn parse(snbt: &str) -> Result<NbtTag, CommandSyntaxError> {
    SnbtParser::parse_for_commands(&mut StringReader::new(snbt))
}

fn suggestions(snbt: &str) -> Vec<String> {
    let mut builder = SuggestionsBuilder::new(snbt, 0);
    let suggestions = SnbtParser::parse_for_suggestions(&mut StringReader::new(snbt), builder);
    suggestions
        .suggestions
        .into_iter()
        .map(|suggestion| suggestion.text_as_string())
        .collect()
}

macro_rules! assert_parse_ok {
    ($snbt:expr, $tag:expr) => {
        let mut reader = StringReader::new($snbt);
        match SnbtParser::parse_for_commands(&mut reader) {
            Err(error) => {
                panic!("Expected a successful parse, but instead got error: {error:?}")
            }
            Ok(tag_parsed) => {
                assert_eq!(
                    tag_parsed, $tag,
                    "Parsed tag does not match the required one"
                );
                assert!(
                    reader.cursor() == reader.string().len(),
                    "Expected everything to get parsed, but found trailing data: {}",
                    &reader.string()[reader.cursor()..]
                );
            },
        }
    };
}

macro_rules! assert_parse_ok_but_trailing {
    ($snbt:expr, $trailing_data:expr) => {
        let mut reader = StringReader::new($snbt);
        if let Err(error) = SnbtParser::parse_for_commands(&mut reader) {
            panic!("Expected a successful parse, but instead got error: {error:?}")
        }
        assert!(reader.cursor() < reader.string().len(), "Expected trailing data, but everything was parsed successfully");
        assert_eq!(&reader.string()[reader.cursor()..], $trailing_data, "Trailing data don't match")
    };
}

macro_rules! assert_parse_err {
    ($snbt:expr, $error_message:expr, $cursor:expr) => {
        let parsed = parse($snbt);
        match parsed {
            Ok(tag) => panic!("Expected command error, but instead got result: {tag:#?}"),
            Err(error) => {
                assert_eq!(
                    error.message.get_text(),
                    $error_message,
                    "Error messages don't match"
                );
                // There should always be a context in SNBT parsing.
                assert_eq!(
                    error.context.unwrap().cursor,
                    $cursor,
                    "Cursor positions for error don't match"
                );
            }
        }
    };

    // Without this, we keep getting the error message: type annotations needed
    ($snbt:expr, $error_message:expr, $cursor:expr, []) => {
        assert_parse_err!($snbt, $error_message, $cursor);
        let suggestions = suggestions($snbt);
        assert!(
            suggestions.is_empty(),
            "Expected no suggestions, but got one or more: {suggestions:?}"
        );
    };
    ($snbt:expr, $error_message:expr, $cursor:expr, $suggestions:expr) => {
        assert_parse_err!($snbt, $error_message, $cursor);
        let suggestions = suggestions($snbt);
        assert_eq!(suggestions, $suggestions, "Suggestions don't match");
    };
}

#[test]
fn integers() {
    assert_parse_ok!("9", NbtTag::Int(9));
    assert_parse_ok!("5_0_0_0", NbtTag::Int(5000));
    assert_parse_err!(
        "5_0_0_0_",
        "Expected literal (",
        8,
        BUILT_IN_LIKE_SUGGESTIONS
    );
    assert_parse_err!(
        "5_0_0_0_",
        "Expected literal (",
        8,
        BUILT_IN_LIKE_SUGGESTIONS
    );

    assert_parse_ok!("3ub", NbtTag::Byte(3));
    assert_parse_ok!("-7s", NbtTag::Short(-7));
    assert_parse_ok!("255uB", NbtTag::Byte(-1));
    assert_parse_err!("256ub", "Failed to parse number: out of range: 256", 5, []);
    assert_parse_ok!("256ss", NbtTag::Short(256));

    assert_parse_err!(
        "3_000_000_000",
        "Expected literal .",
        13,
        [
            ".", "b", "B", "d", "D", "e", "E", "f", "F", "i", "I", "l", "L", "s", "S", "u", "U"
        ]
    );

    assert_parse_ok!("+3_000_000_000uI", NbtTag::Int(-1_294_967_296));
    assert_parse_ok!("+3_000_000_000sL", NbtTag::Long(3_000_000_000));
    assert_parse_ok!("-3_000_000_000sL", NbtTag::Long(-3_000_000_000));

    assert_parse_err!(
        "-3_000_000_000i",
        "Failed to parse number: For input string: \"-3000000000\"",
        15,
        []
    );
    assert_parse_err!("-3_000_000_000UI", "Expected a non-negative number", 16, []);

    assert_parse_err!(
        "00",
        "Expected literal .",
        2,
        [
            "(", ".", "bool", "d", "D", "e", "E", "f", "F", "false", "true", "uuid"
        ]
    );
    assert_parse_err!(
        "0x",
        "Expected a hexadecimal number",
        2,
        BUILT_IN_LIKE_SUGGESTIONS
    );

    assert_parse_ok!("0b", NbtTag::Byte(0));
    assert_parse_ok!("0X111", NbtTag::Int(273));
    assert_parse_err!("0x_111", "Expected literal (", 6, BUILT_IN_LIKE_SUGGESTIONS);
    assert_parse_err!(
        "0xAbCdEfs",
        "Expected literal b|B",
        9,
        ["b", "B", "i", "I", "l", "L", "s", "S"]
    );
    assert_parse_ok_but_trailing!(
        "0xABCDEFG",
        "G"
    );
    assert_parse_ok!("0xABCDUS", NbtTag::Short(-21555));

    // Should not parse as byte of 0xAB
    assert_parse_ok!("0xABB", NbtTag::Int(2747));
}