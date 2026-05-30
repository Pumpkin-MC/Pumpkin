use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_nbt::{Nbt, NbtCompound, deserializer, serializer, tag::NbtTag};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct LargeTest {
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
    nested: Option<Box<Self>>,
}

fn create_large_test_data(depth: usize) -> LargeTest {
    LargeTest {
        byte: 123,
        short: 1342,
        int: 4313,
        long: 34,
        float: 1.00,
        double: 69.42,
        string: "Hello test benchmark data".to_string(),
        byte_array: vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        int_array: vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19],
        long_array: vec![20, 21, 22, 23, 24, 25, 26, 27, 28, 29],
        list_string: vec!["one".to_string(), "two".to_string(), "three".to_string()],
        nested: (depth > 0).then(|| Box::new(create_large_test_data(depth - 1))),
    }
}

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
    let test_data = create_large_test_data(5);
    let mut ser_bytes_java = Vec::new();
    let mut ser_bytes_bedrock = Vec::new();
    serializer::to_bytes(&test_data, &mut ser_bytes_java).unwrap();
    serializer::to_bytes_bedrock(&test_data, &mut ser_bytes_bedrock).unwrap();

    let compound_data = create_large_compound(5);
    let nbt_wrapper = Nbt::new(String::new(), compound_data.clone());
    let wrapper_bytes_java = nbt_wrapper.clone().write();
    let wrapper_bytes_bedrock = nbt_wrapper.write_bedrock();

    c.bench_function("nbt/java/serialize/serde", |b| {
        b.iter(|| {
            let mut out = Vec::new();
            serializer::to_bytes(&test_data, &mut out).unwrap();
        });
    });

    c.bench_function("nbt/java/deserialize/serde", |b| {
        b.iter(|| {
            let cursor = Cursor::new(&ser_bytes_java);
            let _: LargeTest = deserializer::from_bytes(cursor).unwrap();
        });
    });

    c.bench_function("nbt/java/serialize/raw", |b| {
        b.iter(|| {
            let nbt = Nbt::new(String::new(), compound_data.clone());
            let _ = nbt.write();
        });
    });

    c.bench_function("nbt/java/deserialize/raw", |b| {
        b.iter(|| {
            let mut cursor = Cursor::new(&wrapper_bytes_java);
            let mut reader = deserializer::NbtReadHelperJava::new(&mut cursor);
            Nbt::read(&mut reader).unwrap();
        });
    });

    c.bench_function("nbt/bedrock/serialize/serde", |b| {
        b.iter(|| {
            let mut out = Vec::new();
            serializer::to_bytes_bedrock(&test_data, &mut out).unwrap();
        });
    });

    c.bench_function("nbt/bedrock/deserialize/serde", |b| {
        b.iter(|| {
            let cursor = Cursor::new(&ser_bytes_bedrock);
            let _: LargeTest = deserializer::from_bytes_bedrock(cursor).unwrap();
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
            let mut cursor = Cursor::new(&wrapper_bytes_bedrock);
            let mut reader = deserializer::NbtReadHelperBedrock::new(&mut cursor);
            Nbt::read(&mut reader).unwrap();
        });
    });
}

criterion_group!(benches, bench_nbt);
criterion_main!(benches);
