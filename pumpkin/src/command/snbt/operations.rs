use pumpkin_data::translation;
use pumpkin_nbt::{nbt_ops::NbtOps, tag::NbtTag};

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
            && let Some(ints) = Self::parse_uuid(string)
        {
            Some(NbtTag::IntArray(ints))
        } else {
            parser.store_simple_error(&EXPECTED_STRING_UUID);
            None
        }
    }
}

impl SnbtOperations {
    /// Parses UUIDs the 'Java' way.
    #[inline]
    #[must_use]
    fn parse_uuid(uuid: &str) -> Option<Vec<i32>> {
        // We can't directly use the uuid crate to parse UUIDs, as it parses them
        // in a different way from Java.

        if uuid.len() > 36 {
            // UUID string is too large.
            return None;
        }

        // Split by hyphen. (5 segments)
        let mut parts = uuid.split('-');
        let mut parsed_parts: [i64; 5] = [0; 5];

        for part in &mut parsed_parts {
            // If a part is empty, the parsing functions will error anyway - this is what we want.
            *part = i64::from_str_radix(parts.next()?, 16).ok()?;
        }

        if parts.next().is_some() {
            // UUIDs must have exactly 5 parts.
            return None;
        }

        let bits = [
            (parsed_parts[0] & 0xFFFFFFFF) << 32
                | (parsed_parts[1] & 0xFFFF) << 16
                | (parsed_parts[2] & 0xFFFF),
            (parsed_parts[3] & 0xFFFF) << 48 | (parsed_parts[4] & 0xFFFFFFFFFFFF),
        ];

        Some(vec![
            (bits[0] >> 32) as i32,
            bits[0] as i32,
            (bits[1] >> 32) as i32,
            bits[1] as i32,
        ])
    }
}

#[cfg(test)]
mod test {
    use crate::command::snbt::operations::SnbtOperations;

    #[test]
    fn parse_uuids() {
        assert_eq!(
            SnbtOperations::parse_uuid("3d569d3a-93ef-44a0-9f1c-f69db9d37a56"),
            Some(vec![1029086522, -1813035872, -1625491811, -1177322922])
        );
        assert_eq!(
            SnbtOperations::parse_uuid("3d53a-f-40-c-f69db9d37a56"),
            Some(vec![251194, 983104, 849565, -1177322922])
        );
        assert_eq!(SnbtOperations::parse_uuid("3d53a-f40-c-f69db9d37a56"), None);
        assert_eq!(
            SnbtOperations::parse_uuid("fffffffffffffff-0-0-0-0"),
            Some(vec![-1, 0, 0, 0])
        );
        assert_eq!(SnbtOperations::parse_uuid("ffffffffffffffff-0-0-0-0"), None);
        assert_eq!(
            SnbtOperations::parse_uuid("+1-+2-+3-+4-+5"),
            Some(vec![1, 131075, 262144, 5])
        );
        assert_eq!(
            SnbtOperations::parse_uuid("aaaaaaaaaaaaaaa-bbbbbbbbbbbbbb-c-d-e"),
            Some(vec![-1431655766, -1145372660, 851968, 14])
        );
    }
}
