#![no_main]
use libfuzzer_sys::fuzz_target;
use pumpkin_nbt::deserializer::{from_bytes, from_bytes_unnamed};
use serde::Deserialize;
use std::io::Cursor;

#[derive(Deserialize, Debug, PartialEq)]
struct FuzzStruct {
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
    nested: Option<Box<FuzzStruct>>,
}

fuzz_target!(|data: &[u8]| {
    let cursor = Cursor::new(data);
    let _: Result<FuzzStruct, _> = from_bytes(cursor);

    let cursor_unnamed = Cursor::new(data);
    let _: Result<FuzzStruct, _> = from_bytes_unnamed(cursor_unnamed);
});
