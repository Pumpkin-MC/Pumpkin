#![no_main]
use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use pumpkin_nbt::deserializer::from_bytes;
use pumpkin_nbt::serializer::to_bytes;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Arbitrary, Serialize, Deserialize, Debug, PartialEq)]
struct RoundtripStruct {
    byte: i8,
    short: i16,
    int: i32,
    long: i64,
    float: f32,
    double: f64,
    string: String,
    byte_array: Vec<u8>,
    int_array: Vec<i32>,
    long_array: Vec<i64>,
    list_string: Vec<String>,
    // Nested struct to test recursion
    nested: Option<Box<RoundtripStruct>>,
}

fuzz_target!(|data: RoundtripStruct| {
    let mut fixed_data = data;

    let mut struct_to_fixup = &mut fixed_data;
    loop {
        // prevent NaN != NaN issues
        if struct_to_fixup.float != struct_to_fixup.float {
            struct_to_fixup.float = 0.0;
        }
        if struct_to_fixup.double != struct_to_fixup.double {
            struct_to_fixup.double = 0.0;
        }

        if struct_to_fixup.nested.is_none() {
            break;
        }
        struct_to_fixup = struct_to_fixup.nested.as_mut().unwrap();
    }

    let mut bytes = Vec::new();
    if to_bytes(&fixed_data, &mut bytes).is_ok() {
        let cursor = Cursor::new(bytes);
        let reconstructed: RoundtripStruct =
            from_bytes(cursor).expect("Failed to deserialize valid NBT");
        assert_eq!(fixed_data, reconstructed);
    }
});
