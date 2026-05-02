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
            }
        }
    };
}

macro_rules! assert_parse_ok_but_trailing {
    ($snbt:expr, $trailing_data:expr) => {
        let mut reader = StringReader::new($snbt);
        if let Err(error) = SnbtParser::parse_for_commands(&mut reader) {
            panic!("Expected a successful parse, but instead got error: {error:?}")
        }
        assert!(
            reader.cursor() < reader.string().len(),
            "Expected trailing data, but everything was parsed successfully"
        );
        assert_eq!(
            &reader.string()[reader.cursor()..],
            $trailing_data,
            "Trailing data don't match"
        )
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
    assert_parse_ok!("256 s s", NbtTag::Short(256));

    assert_parse_err!(
        "3_000_000_000",
        "Expected literal .",
        13,
        [
            ".", "b", "B", "d", "D", "e", "E", "f", "F", "i", "I", "l", "L", "s", "S", "u", "U"
        ]
    );

    assert_parse_ok!("+3_000_000_000uI", NbtTag::Int(-1_294_967_296));
    assert_parse_ok!("+3_000_000_000s L", NbtTag::Long(3_000_000_000));
    assert_parse_ok!("-3_000_000_000 sL", NbtTag::Long(-3_000_000_000));

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
    assert_parse_ok!("0b10101", NbtTag::Int(21));

    assert_parse_ok!("0X111", NbtTag::Int(273));
    assert_parse_err!("0x_111", "Expected literal (", 6, BUILT_IN_LIKE_SUGGESTIONS);
    assert_parse_err!(
        "0xAbCdEfs",
        "Expected literal b|B",
        9,
        ["b", "B", "i", "I", "l", "L", "s", "S"]
    );
    assert_parse_ok_but_trailing!("0xABCDEFG", "G");
    assert_parse_ok!("0xABCDUS", NbtTag::Short(-21555));

    // Should not parse as byte of 0xAB
    assert_parse_ok!("0xABB", NbtTag::Int(2747));
}

#[test]
fn floats() {
    assert_parse_ok!("0.", NbtTag::Double(0.0));
    assert_parse_ok!("0.f", NbtTag::Float(0.0));
    assert_parse_ok!("0.D", NbtTag::Double(0.0));

    assert_parse_ok!(".0", NbtTag::Double(0.0));
    assert_parse_ok!(".0F", NbtTag::Float(0.0));
    assert_parse_ok!(".0d", NbtTag::Double(0.0));

    assert_parse_ok!("1.024", NbtTag::Double(1.024));
    assert_parse_err!("1_.024", "Expected literal (", 6, BUILT_IN_LIKE_SUGGESTIONS);
    assert_parse_ok_but_trailing!("1._024", "_024");
    assert_parse_ok!("1.0_2_4", NbtTag::Double(1.024));

    assert_parse_ok!("1e1", NbtTag::Double(10.0));
    assert_parse_ok!("2e+2", NbtTag::Double(200.0));
    assert_parse_ok!("4e-2", NbtTag::Double(0.04));

    assert_parse_ok!("4e-2", NbtTag::Double(0.04));
    assert_parse_ok!("0E100_000_000", NbtTag::Double(0.0));
    assert_parse_ok_but_trailing!("0.1e100_000_000", ".1e100_000_000");
    assert_parse_ok!("0.1e-100_000_000", NbtTag::Double(0.0));

    assert_parse_ok!("1e38f", NbtTag::Float(1e38));
    assert_parse_ok_but_trailing!("1e39f", "e39f");
    assert_parse_ok!("1e39", NbtTag::Double(1e39));
    assert_parse_ok!("0.001e41f", NbtTag::Float(1e38));
    assert_parse_ok_but_trailing!("0.01E41f", ".01E41f");

    assert_parse_ok!("1.28E308", NbtTag::Double(1.28E308));
    assert_parse_ok_but_trailing!("1.8e308", ".8e308");

    assert_parse_ok_but_trailing!("1.E", "E");

    assert_parse_ok!("2000f", NbtTag::Float(2000.0));
    assert_parse_ok!("70d", NbtTag::Double(70.0));
    assert_parse_ok!("03f", NbtTag::Float(3.0));
    assert_parse_ok!("03.70", NbtTag::Double(3.7));
    assert_parse_ok!("+1e-1", NbtTag::Double(0.1));
}

#[test]
fn quoted_string_literals() {
    assert_parse_ok!("''", NbtTag::String(String::new()));
    assert_parse_ok!("\"\"", NbtTag::String(String::new()));

    assert_parse_ok!("\"'hello'\"", NbtTag::String("'hello'".to_string()));
    assert_parse_ok!("'\"hello\"'", NbtTag::String("\"hello\"".to_string()));
    assert_parse_ok!("'\\\\'", NbtTag::String("\\".to_string()));
    assert_parse_ok_but_trailing!("'\"'\"", "\"");

    assert_parse_err!("'\\'", "Invalid string contents", 3, ["\"", "'", "\\"]);

    assert_parse_ok!(
        "'hello worl\\bd'",
        NbtTag::String("hello worl\u{8}d".to_string())
    );
    assert_parse_ok!("'hello\\sword'", NbtTag::String("hello word".to_string()));
    assert_parse_ok!("'hello\\tword'", NbtTag::String("hello\tword".to_string()));
    assert_parse_ok!(
        "'\\some or \\none'",
        NbtTag::String(" ome or \none".to_string())
    );
    assert_parse_ok!("'\\f\\r'", NbtTag::String("\u{c}\r".to_string()));

    assert_parse_ok!("'hello \\x65!'", NbtTag::String("hello e!".to_string()));
    assert_parse_ok!(
        "'\\x53\\x65\\u0063\\U00000072\\x65\\x74\\x21'",
        NbtTag::String("Secret!".to_string())
    );

    assert_parse_err!(
        "'\\U1234567'",
        "Expected a character literal of length 8",
        3,
        []
    );

    assert_parse_ok!(
        "'\\uD83C\\uDF83 or \\U0001F383'",
        NbtTag::String("🎃 or 🎃".to_string())
    );

    // TODO: make tests for when \N is implemented
}

#[test]
fn unquoted_string_literals() {
    assert_parse_ok!("abc", NbtTag::String("abc".to_string()));
    assert_parse_ok!(
        "abc-def_ghi+jkl.mno",
        NbtTag::String("abc-def_ghi+jkl.mno".to_string())
    );
    assert_parse_ok!("_1234", NbtTag::String("_1234".to_string()));
    assert_parse_ok!("x+1", NbtTag::String("x+1".to_string()));
    assert_parse_ok_but_trailing!("x*1", "*1");

    assert_parse_ok!("true", NbtTag::Byte(1));
    assert_parse_ok!("false", NbtTag::Byte(0));
    assert_parse_ok!("maybe", NbtTag::String("maybe".to_string()));
    assert_parse_ok!("bool", NbtTag::String("bool".to_string()));
}

#[test]
fn operation() {
    assert_parse_ok!("bool( true)", NbtTag::Byte(1));
    assert_parse_ok!("bool (false )", NbtTag::Byte(0));

    assert_parse_ok!("bool(0)", NbtTag::Byte(0));
    assert_parse_ok!("bool( 1 )", NbtTag::Byte(1));
    assert_parse_ok!("bool (2.5  )", NbtTag::Byte(1));
    assert_parse_ok!("bool ( -4.3412e+12  )", NbtTag::Byte(0));

    assert_parse_err!("bool(", "Expected a valid unquoted string", 5, [")"]);
    assert_parse_err!("bool()", "No such operation: bool/0", 6, []);
    assert_parse_err!("bool(1, 2)", "No such operation: bool/2", 10, []);
    assert_parse_err!(
        "bool (1,2,3",
        "Expected literal .",
        11,
        [
            ")", ",", ".", "b", "B", "d", "D", "e", "E", "f", "F", "i", "I", "l", "L", "s", "S",
            "u", "U"
        ]
    );
}
