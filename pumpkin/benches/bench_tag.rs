use criterion::{Criterion, criterion_group, criterion_main};

use pumpkin_data::tag::Tagable;
use pumpkin_data::{Block, tag};

fn bench_tag(c: &mut Criterion) {
    let name = "is_tagged_with";
    c.bench_function(name, |b| {
        b.iter(|| {
            Block::GRASS_BLOCK
                .is_tagged_with_by_tag(&tag::Block::MINECRAFT_REPLACEABLE_BY_MUSHROOMS);
            Block::RAIL.is_tagged_with_by_tag(&tag::Block::MINECRAFT_SCULK_REPLACEABLE_WORLD_GEN);
            Block::COPPER_BULB
                .is_tagged_with_by_tag(&tag::Block::MINECRAFT_INCORRECT_FOR_GOLD_TOOL);
            Block::BEDROCK.is_tagged_with_by_tag(&tag::Block::MINECRAFT_INCORRECT_FOR_GOLD_TOOL);
            Block::BEDROCK.is_tagged_with_by_tag(&tag::Block::C_ORE_BEARING_GROUND_NETHERRACK);
        })
    });
    let name = "get_tag_values";
    c.bench_function(name, |b| {
        b.iter(|| {
            Block::get_tag_values("minecraft:replaceable_by_mushrooms");
            Block::get_tag_values("minecraft:sculk_replaceable_world_gen");
            Block::get_tag_values("minecraft:incorrect_for_gold_tool");
            Block::get_tag_values("minecraft:incorrect_for_gold_tool");
            Block::get_tag_values("c:ore_bearing_ground/netherrack");
        })
    });
}

criterion_group!(benches, bench_tag);
criterion_main!(benches);
