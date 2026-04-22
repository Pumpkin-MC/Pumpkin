#![allow(clippy::print_stdout)]

use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_nbt::{from_bytes_unnamed, from_pnbt, to_bytes_unnamed, to_pnbt};
use serde::{Deserialize, Serialize};
use std::hint::black_box;
use std::io::Cursor;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct LargeData {
    id: i32,
    name: String,
    metadata: Vec<Metadata>,
    inventory: Vec<Item>,
    scores: Vec<i32>,
    active: bool,
    position: (f64, f64, f64),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Metadata {
    key: String,
    value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Item {
    id: String,
    count: i8,
    slot: i32,
}

fn create_large_data() -> LargeData {
    LargeData {
        id: 1234567,
        name: "Pumpkin King".to_string(),
        metadata: (0..50)
            .map(|i| Metadata {
                key: format!("meta_key_{i}"),
                value: format!("meta_value_{i}"),
            })
            .collect(),
        inventory: (0..27)
            .map(|i| Item {
                id: "minecraft:diamond_sword".to_string(),
                count: 64,
                slot: i,
            })
            .collect(),
        scores: (0..100).map(|i| i * 1000).collect(),
        active: true,
        position: (1234.56, 64.0, -789.12),
    }
}

fn bench_nbt(c: &mut Criterion) {
    let data = create_large_data();

    // Size comparison
    let mut vanilla_bytes = Vec::new();
    to_bytes_unnamed(&data, &mut vanilla_bytes).unwrap();
    let pnbt_bytes = to_pnbt(&data).unwrap();

    println!("\nSize Comparison (LargeData):");
    println!("Vanilla NBT size: {} bytes", vanilla_bytes.len());
    println!("PNBT size:        {} bytes", pnbt_bytes.len());
    println!(
        "Reduction:            {:.2}%\n",
        (1.0 - (pnbt_bytes.len() as f64 / vanilla_bytes.len() as f64)) * 100.0
    );

    let mut group = c.benchmark_group("NBT Comparison");

    group.bench_function("Vanilla Serialize", |b| {
        b.iter(|| {
            let mut out = Vec::with_capacity(vanilla_bytes.len());
            to_bytes_unnamed(black_box(&data), &mut out).unwrap();
        });
    });

    group.bench_function("PNBT Serialize", |b| {
        b.iter(|| {
            to_pnbt(black_box(&data)).unwrap();
        });
    });

    group.bench_function("Vanilla Deserialize", |b| {
        b.iter(|| {
            let cursor = Cursor::new(&vanilla_bytes);
            let _: LargeData = from_bytes_unnamed(cursor).unwrap();
        });
    });

    group.bench_function("PNBT Deserialize", |b| {
        b.iter(|| {
            let _: LargeData = from_pnbt(black_box(&pnbt_bytes)).unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, bench_nbt);
criterion_main!(benches);
