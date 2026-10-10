use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_data::{Block, BlockStateId};
use pumpkin_world::chunk::palette::BlockPalette;

fn bedrock_palette(c: &mut Criterion) {
    let mut group = c.benchmark_group("bedrock_block_palette");
    for count in [2, 16, 65, 256, 300] {
        let mut palette = BlockPalette::default();
        for y in 0..16 {
            for z in 0..16 {
                for x in 0..16 {
                    palette.set(
                        x,
                        y,
                        z,
                        BlockStateId::new_or_air(((y * 256 + z * 16 + x) % count) as u16),
                    );
                }
            }
        }
        group.bench_function(count.to_string(), |b| {
            b.iter(|| black_box(black_box(&palette).convert_be_network()));
        });
    }
    group.finish();

    let mut water = BlockPalette::default();
    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                if (x + y + z) % 3 == 0 {
                    water.set(x, y, z, Block::KELP.default_state.id);
                }
            }
        }
    }
    c.bench_function("bedrock_water_palette", |b| {
        b.iter(|| black_box(black_box(&water).convert_be_water_network()));
    });
}

criterion_group!(benches, bedrock_palette);
criterion_main!(benches);
