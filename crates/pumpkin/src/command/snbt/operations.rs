use pumpkin_data::translation;
use pumpkin_nbt::{nbt_ops::NbtOps, tag::NbtTag};
use pumpkin_util::uuid::parse_uuid_vec;

use crate::command::{errors::error_types::CommandErrorType, parser::Parser, snbt::SnbtParser};
use pumpkin_codecs::DynamicOps;

pub const EXPECTED_NUMBER_OR_BOOLEAN: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_EXPECTED_NUMBER_OR_BOOLEAN,
    translation::java::SNBT_PARSER_EXPECTED_NUMBER_OR_BOOLEAN,
);

pub const EXPECTED_STRING_UUID: CommandErrorType<0> = CommandErrorType::new(
    translation::java::SNBT_PARSER_EXPECTED_STRING_UUID,
    translation::java::SNBT_PARSER_EXPECTED_STRING_UUID,
);

/// Represents an *operation* that can take *operands* and return a required *result*.
pub type SnbtOperation = fn(parser: &mut SnbtParser, args: &[NbtTag]) -> Option<NbtTag>;

/// A manager for SNBT operations baked at compile-time.
pub struct SnbtOperations;

impl SnbtOperations {
    pub const BUILTIN_IDS: &[&str] = &["true", "false", "bool", "uuid"];

    /// Searches for an operation to be run from the
    /// given identifier and argument count.
    pub fn search(id: &str, arg_count: usize) -> Option<SnbtOperation> {
        match (id, arg_count) {
            ("bool", 1) => Some(Self::bool),
            ("uuid", 1) => Some(Self::uuid),
            _ => None,
        }
    }

    /// Represents the `bool` unary operator in SNBT.
    ///
    /// Acts like an identity operation for booleans,
    /// and returns `true` for non-zero numbers.
    fn bool(parser: &mut SnbtParser, args: &[NbtTag]) -> Option<NbtTag> {
        NbtOps.get_bool(&args[0]).into_result().map_or_else(
            || {
                parser.store_simple_error(&EXPECTED_NUMBER_OR_BOOLEAN);
                None
            },
            |result| Some(NbtTag::Byte(result as i8)),
        )
    }

    /// Represents the `uuid` unary operator in SNBT.
    ///
    /// Parses a UUID in a string to an array of 4 integers.
    fn uuid(parser: &mut SnbtParser, args: &[NbtTag]) -> Option<NbtTag> {
        if let NbtTag::String(string) = &args[0]
            && let Some(ints) = parse_uuid_vec(string)
        {
            Some(NbtTag::IntArray(ints))
        } else {
            parser.store_simple_error(&EXPECTED_STRING_UUID);
            None
        }
    }
}
