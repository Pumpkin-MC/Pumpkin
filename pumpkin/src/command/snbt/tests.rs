use std::collections::HashMap;

use pumpkin_nbt::{compound::NbtCompound, nbt_compound_tag, tag::NbtTag};

const BUILT_IN_LIKE_SUGGESTIONS: &[&str] = &["(", "bool", "false", "true", "uuid"];

use crate::command::{
    errors::command_syntax_error::CommandSyntaxError, snbt::SnbtParser,
    string_reader::StringReader, suggestion::suggestions::SuggestionsBuilder,
};

fn parse(snbt: &str) -> Result<NbtTag, CommandSyntaxError> {
    SnbtParser::parse_for_commands(&mut StringReader::new(snbt))
}

fn suggestions(snbt: &str) -> Vec<String> {
    let builder = SuggestionsBuilder::new(snbt, 0);
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

    assert_parse_ok!("'\\b'", NbtTag::String("\u{8}".to_string()));
    assert_parse_ok!("'hello\\sword'", NbtTag::String("hello word".to_string()));
    assert_parse_ok!(
        "'hello\\tword\n'",
        NbtTag::String("hello\tword\n".to_string())
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
fn operations() {
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

    assert_parse_ok!(
        "uuid('3d569d3a-93ef-44a0-9f1c-f69db9d37a56')",
        NbtTag::IntArray(vec![1029086522, -1813035872, -1625491811, -1177322922])
    );
    assert_parse_ok!(
        "uuid(ad569d3a-93ef-44a0-9f1c-f69db9d37a56)",
        NbtTag::IntArray(vec![-1386832582, -1813035872, -1625491811, -1177322922])
    );
    assert_parse_err!(
        "uuid(3d53a-f40-c-f69db9d37a56)",
        "Expected literal ,",
        7,
        [")", ","]
    );
    assert_parse_ok!(
        "uuid(fffffffffffffff-0-0-0-0)",
        NbtTag::IntArray(vec![-1, 0, 0, 0])
    );
    assert_parse_ok!(
        "uuid(AaaAaaAaaAaaAaA-BBbBbbBbBbbbBB-c-D-e)",
        NbtTag::IntArray(vec![-1431655766, -1145372660, 851968, 14])
    );
    assert_parse_ok!(
        "uuid(a1-+2-+3-+4-+5)",
        NbtTag::IntArray(vec![161, 131075, 262144, 5])
    );
    assert_parse_err!(
        "uuid(x)",
        "Expected a string representing a valid UUID",
        7,
        []
    );
}

#[test]
fn maps() {
    assert_parse_ok!(
        "{x:5}",
        NbtTag::Compound(NbtCompound {
            child_tags: HashMap::from([("x".to_string(), NbtTag::Int(5))])
        })
    );
    assert_parse_ok!(
        "{ a:1b,B :2uS , c:3L  }",
        NbtTag::Compound(NbtCompound {
            child_tags: HashMap::from([
                ("a".to_string(), NbtTag::Byte(1)),
                ("B".to_string(), NbtTag::Short(2)),
                ("c".to_string(), NbtTag::Long(3))
            ])
        })
    );
    assert_parse_ok!(
        "{ a:1b, \"a\":2b, 'a':\"hi\" }",
        nbt_compound_tag! {
            "a": NbtTag::String("hi".to_string())
        }
    );
    assert_parse_ok!(
        "{ elem: 'this', next: { elem: 'is', next: { elem: 'a', next: { elem: 'linked', next: 'list' } } } }",
        nbt_compound_tag! {
            "elem": NbtTag::String("this".to_string()),
            "next": nbt_compound_tag! {
                "elem": NbtTag::String("is".to_string()),
                "next": nbt_compound_tag! {
                    "elem": NbtTag::String("a".to_string()),
                    "next": nbt_compound_tag! {
                        "elem": NbtTag::String("linked".to_string()),
                        "next": NbtTag::String("list".to_string()),
                    }
                }
            }
        }
    );

    assert_parse_err!("{'x': 5f", "Expected literal ,", 8, [",", "}"]);
    assert_parse_err!(
        "{text:\"cool\",\"color:dark_red}",
        "Invalid string contents",
        29,
        ["\"", "'", "\\"]
    );
    assert_parse_ok!(
        "{9._+._+foo:1}",
        nbt_compound_tag! {
            "9._+._+foo": NbtTag::Int(1)
        }
    );
    assert_parse_err!("{9._+._+=foo:1}", "Expected literal :", 8, [":"]);

    assert_parse_err!("{\"a\":b", "Expected literal (", 6, ["(", ",", "}"]);
    assert_parse_err!(
        "{\"a\":25",
        "Expected literal .",
        7,
        [
            ",", ".", "b", "B", "d", "D", "e", "E", "f", "F", "i", "I", "l", "L", "s", "S", "u",
            "U", "}"
        ]
    );
    assert_parse_err!("{\"a\":25,", "Expected literal \"", 8, ["\"", "'", "}"]);
    assert_parse_err!("{,}", "Expected literal \"", 1, ["\"", "'", "}"]);
    assert_parse_err!("{{}}", "Expected literal \"", 1, ["\"", "'", "}"]);

    assert_parse_ok!(
        "{1:1}",
        nbt_compound_tag! {
            "1": NbtTag::Int(1)
        }
    );
}

#[test]
fn lists() {
    assert_parse_ok!("[  ]", NbtTag::List(Vec::new()));
    assert_parse_ok!(
        "[5, _]",
        NbtTag::List(vec![NbtTag::Int(5), NbtTag::String("_".to_string())])
    );
    assert_parse_ok!(
        "[a, [true, c], [4s, uuid(f-0-0-0-0), f, [[], 7f, [\"\\n\", [200Ub, {x: 1e1}]]]]]",
        NbtTag::List(vec![
            NbtTag::String("a".to_string()),
            NbtTag::List(vec![NbtTag::Byte(1), NbtTag::String("c".to_string())]),
            NbtTag::List(vec![
                NbtTag::Short(4),
                NbtTag::IntArray(vec![15, 0, 0, 0]),
                NbtTag::String("f".to_string()),
                NbtTag::List(vec![
                    NbtTag::List(Vec::new()),
                    NbtTag::Float(7.0),
                    NbtTag::List(vec![
                        NbtTag::String("\n".to_string()),
                        NbtTag::List(vec![
                            NbtTag::Byte(-56),
                            nbt_compound_tag! {
                                "x": NbtTag::Double(10.0)
                            }
                        ]),
                    ]),
                ]),
            ])
        ])
    );

    assert_parse_err!(
        "[1;1]",
        "Expected literal .",
        2,
        [
            ",", ".", "]", "b", "B", "d", "D", "e", "E", "f", "F", "i", "I", "l", "L", "s", "S",
            "u", "U"
        ]
    );

    assert_parse_err!("[{]}", "Expected literal \"", 2, ["\"", "'", "}"]);
    assert_parse_err!("{[}]", "Expected literal \"", 1, ["\"", "'", "}"]);
    assert_parse_err!("[,]", "Expected literal B", 1, ["]", "B", "I", "L"]);
    assert_parse_err!("[Z;9]", "Expected literal (", 2, ["(", ",", "]"]);
}

#[test]
fn arrays() {
    assert_parse_ok!("[B;]", NbtTag::ByteArray(vec![]));
    assert_parse_ok!("[I ;1  ,2 ,  3,]", NbtTag::IntArray(vec![1, 2, 3]));
    assert_parse_ok!("[L;1  ,2 ,  3,   4]", NbtTag::LongArray(vec![1, 2, 3, 4]));

    assert_parse_err!("[B;1i]", "Invalid array element type", 6, []);
    assert_parse_err!("[I;1L]", "Invalid array element type", 6, []);
    assert_parse_err!(
        "[B;128]",
        "Failed to parse number: Value out of range. Value:\"128\" Radix:10",
        7,
        []
    );
    assert_parse_err!(
        "[B;3000000000]",
        "Failed to parse number: For input string: \"3000000000\"",
        14,
        []
    );
    assert_parse_err!(
        "[I;3000000000]",
        "Failed to parse number: For input string: \"3000000000\"",
        14,
        []
    );
    assert_parse_err!(
        "[I; 1.0]",
        "Expected literal u|U",
        5,
        [",", "]", "b", "B", "i", "I", "l", "L", "s", "S", "u", "U"]
    );
    assert_parse_err!("[I;{}]", "Expected literal +", 3, ["+", "-", "0", "]"]);
    assert_parse_err!("[i;4]", "Expected literal (", 2, ["(", ",", "]"]);

    assert_parse_ok!("[B; 0b11111111]", NbtTag::ByteArray(vec![-1]));
    assert_parse_ok!("[L; 0xFFFFFFFFFFFFFFFF]", NbtTag::LongArray(vec![-1]));
    assert_parse_err!(
        "[L; 0xFFFFFFFFFFFFFFFFF]",
        "Failed to parse number: String value FFFFFFFFFFFFFFFFF exceeds range of unsigned long.",
        24,
        []
    );
}
