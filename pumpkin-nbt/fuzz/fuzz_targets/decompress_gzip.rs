#![no_main]
use libfuzzer_sys::fuzz_target;
use pumpkin_nbt::nbt_compress::read_gzip_compound_tag;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let cursor = Cursor::new(data);
    let _ = read_gzip_compound_tag(cursor);
});
