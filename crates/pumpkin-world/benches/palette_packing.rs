use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use pumpkin_world::chunk::palette::BlockPalette;

fn palette_packing(c: &mut Criterion) {
    let mut group = c.benchmark_group("palette_packing");
    for size in [16usize, 256, 257, 1024, 4096] {
        let palette: Vec<_> = (0..size as u16)
            .map(pumpkin_data::BlockStateId::new_or_air)
            .collect();
        let bits = pumpkin_util::encompassing_bits(size).max(4);
        let per_word = 64 / bits as usize;
        let mut packed = vec![0i64; 4096usize.div_ceil(per_word)];
        for index in 0..4096 {
            packed[index / per_word] |=
                ((index % size) as i64) << ((index % per_word) * bits as usize);
        }
        let container = BlockPalette::from_palette_and_packed_data(&palette, &packed, 4);
        group.bench_with_input(BenchmarkId::from_parameter(size), &container, |b, data| {
            b.iter(|| black_box(data).to_palette_and_packed_data(black_box(bits)));
        });
    }
    group.finish();
}

criterion_group!(benches, palette_packing);
criterion_main!(benches);
