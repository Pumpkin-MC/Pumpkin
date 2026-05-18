#![no_main]
use libfuzzer_sys::fuzz_target;
use pumpkin_nbt::Nbt;
use pumpkin_nbt::deserializer::NbtReadHelper;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let mut cursor = Cursor::new(data);
    let mut reader = NbtReadHelper::new(&mut cursor);
    let _ = Nbt::read(&mut reader);

    let mut cursor_unnamed = Cursor::new(data);
    let mut reader_unnamed = NbtReadHelper::new(&mut cursor_unnamed);
    let _ = Nbt::read_unnamed(&mut reader_unnamed);
});
