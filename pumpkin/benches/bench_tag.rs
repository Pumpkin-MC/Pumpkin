use criterion::{Criterion, criterion_group, criterion_main};

use pumpkin_data;
use pumpkin_data::Block;
use pumpkin_data::tag::Tagable;

fn bench_tag(c: &mut Criterion) {
    let name = "is_tagged_with";
    c.bench_function(&name, |b| b.iter(|| {
        Block::GRASS_BLOCK.is_tagged_with("minecraft:replaceable_by_mushrooms");
        Block::RAIL.is_tagged_with("minecraft:sculk_replaceable_world_gen");
        Block::COPPER_BULB.is_tagged_with("minecraft:incorrect_for_gold_tool");
        Block::BEDROCK.is_tagged_with("minecraft:incorrect_for_gold_tool");
        Block::BEDROCK.is_tagged_with("c:ore_bearing_ground/netherrack");
    }));
    let name = "get_tag_values";
    c.bench_function(&name, |b| b.iter(|| {
        Block::get_tag_values("minecraft:replaceable_by_mushrooms");
        Block::get_tag_values("minecraft:sculk_replaceable_world_gen");
        Block::get_tag_values("minecraft:incorrect_for_gold_tool");
        Block::get_tag_values("minecraft:incorrect_for_gold_tool");
        Block::get_tag_values("c:ore_bearing_ground/netherrack");
    }));
}


criterion_group!(benches, bench_tag);
criterion_main!(benches);