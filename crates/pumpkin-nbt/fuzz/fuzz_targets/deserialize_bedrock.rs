#![no_main]
use libfuzzer_sys::fuzz_target;
use pumpkin_nbt::Nbt;
use pumpkin_nbt::deserializer::NbtReadHelperBedrock;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let mut cursor = Cursor::new(data);
    let mut reader = NbtReadHelperBedrock::new(&mut cursor);
    let _ = Nbt::read(&mut reader);
});
