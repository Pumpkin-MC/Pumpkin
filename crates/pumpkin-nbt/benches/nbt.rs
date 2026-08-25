#![allow(clippy::unwrap_used)]

use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_nbt::{Nbt, NbtCompound, deserializer, tag::NbtTag};
use std::io::Cursor;

fn create_large_compound(depth: usize) -> NbtCompound {
    let mut compound = NbtCompound::new();
    compound.put_byte("byte", 123);
    compound.put_short("short", 1342);
    compound.put_int("int", 4313);
    compound.put_long("long", 34);
    compound.put_float("float", 1.00);
    compound.put_double("double", 69.42);
    compound.put_string("string", "Hello test benchmark data".to_string());
    compound.put(
        "byte_array",
        NbtTag::ByteArray(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9].into()),
    );
    compound.put(
        "int_array",
        NbtTag::IntArray(vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19]),
    );
    compound.put(
        "long_array",
        NbtTag::LongArray(vec![20, 21, 22, 23, 24, 25, 26, 27, 28, 29]),
    );

    let list = vec![
        NbtTag::String("one".into()),
        NbtTag::String("two".into()),
        NbtTag::String("three".into()),
    ];
    compound.put_list("list_string", list);

    if depth > 0 {
        compound.put_compound("nested", create_large_compound(depth - 1));
    }
    compound
}

pub fn bench_nbt(c: &mut Criterion) {
    let compound_data = create_large_compound(5);
    let nbt_wrapper = Nbt::new(String::new(), compound_data.clone());
    let wrapper_bytes_java = nbt_wrapper.clone().write();
    let wrapper_bytes_bedrock = nbt_wrapper.write_bedrock();

    c.bench_function("nbt/java/serialize/raw", |b| {
        b.iter(|| {
            let nbt = Nbt::new(String::new(), compound_data.clone());
            let _ = nbt.write();
        });
    });

    c.bench_function("nbt/java/deserialize/raw", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(&wrapper_bytes_java[..]);
            let mut reader = deserializer::NbtReadHelperJava::new(&mut cursor);
            Nbt::read(&mut reader).unwrap();
        });
    });

    c.bench_function("nbt/bedrock/serialize/raw", |b| {
        b.iter(|| {
            let nbt = Nbt::new(String::new(), compound_data.clone());
            let _ = nbt.write_bedrock();
        });
    });

    c.bench_function("nbt/bedrock/deserialize/raw", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(&wrapper_bytes_bedrock[..]);
            let mut reader = deserializer::NbtReadHelperBedrock::new(&mut cursor);
            Nbt::read(&mut reader).unwrap();
        });
    });
}

criterion_group!(benches, bench_nbt);
criterion_main!(benches);
