use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use flate2::{Compress, Compression, Decompress, FlushCompress, FlushDecompress, Status};
use std::hint::black_box;

#[derive(Clone, Copy)]
enum PayloadProfile {
    Repetitive,
    Structured,
    PseudoRandom,
}

impl PayloadProfile {
    const fn name(self) -> &'static str {
        match self {
            Self::Repetitive => "repetitive",
            Self::Structured => "structured",
            Self::PseudoRandom => "pseudo_random",
        }
    }

    fn generate(self, size: usize) -> Vec<u8> {
        match self {
            Self::Repetitive => (0..size)
                .map(|index| b"chunkdata"[index % b"chunkdata".len()])
                .collect(),
            Self::Structured => (0..size)
                .map(|index| {
                    let section = index / 4096;
                    let within_section = index % 4096;
                    if within_section < 3072 {
                        (section % 8) as u8
                    } else {
                        ((within_section / 17 + section * 13) % 251) as u8
                    }
                })
                .collect(),
            Self::PseudoRandom => {
                let mut state = 0xD1B5_4A32_D192_ED03u64;
                (0..size)
                    .map(|_| {
                        state ^= state << 13;
                        state ^= state >> 7;
                        state ^= state << 17;
                        state as u8
                    })
                    .collect()
            }
        }
    }
}

fn compress(
    compressor: &mut Compress,
    output: &mut Vec<u8>,
    payload: &[u8],
) -> Result<usize, flate2::CompressError> {
    output.clear();
    let reserve_hint = payload
        .len()
        .saturating_add(payload.len() / 16)
        .saturating_add(64);
    if output.capacity() < reserve_hint {
        output.reserve(reserve_hint - output.capacity());
    }
    compressor.reset();
    let status = compressor.compress_vec(payload, output, FlushCompress::Finish)?;
    assert_eq!(status, Status::StreamEnd);
    Ok(output.len())
}

fn verify_round_trip(payload: &[u8], compressed: &[u8]) {
    let mut decompressor = Decompress::new(true);
    let mut output = Vec::with_capacity(payload.len());
    let status = decompressor
        .decompress_vec(compressed, &mut output, FlushDecompress::Finish)
        .unwrap();
    assert_eq!(status, Status::StreamEnd);
    assert_eq!(output, payload);
}

fn benchmark_case(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    level: u32,
    profile: PayloadProfile,
    size: usize,
) {
    let payload = profile.generate(size);
    let mut compressor = Compress::new(Compression::new(level), true);
    let mut compressed = Vec::new();
    compress(&mut compressor, &mut compressed, &payload).unwrap();
    verify_round_trip(&payload, &compressed);

    group.throughput(Throughput::Bytes(size as u64));
    group.bench_with_input(
        BenchmarkId::new(format!("level_{level}/{}", profile.name()), size),
        &payload,
        |b, payload| {
            let mut compressor = Compress::new(Compression::new(level), true);
            let mut output = Vec::with_capacity(
                payload
                    .len()
                    .saturating_add(payload.len() / 16)
                    .saturating_add(64),
            );
            b.iter(|| {
                black_box(
                    compress(&mut compressor, &mut output, black_box(payload.as_slice())).unwrap(),
                )
            });
        },
    );
}

fn bench_packet_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("packet_compression");
    group.sample_size(20);
    group.warm_up_time(std::time::Duration::from_secs(1));
    group.measurement_time(std::time::Duration::from_secs(2));

    for size in [256, 1024, 16 * 1024, 64 * 1024, 1024 * 1024] {
        for profile in [
            PayloadProfile::Repetitive,
            PayloadProfile::Structured,
            PayloadProfile::PseudoRandom,
        ] {
            benchmark_case(&mut group, 4, profile, size);
        }
    }

    for level in [1, 6] {
        benchmark_case(&mut group, level, PayloadProfile::Structured, 64 * 1024);
    }

    group.finish();
}

criterion_group!(benches, bench_packet_compression);
criterion_main!(benches);
