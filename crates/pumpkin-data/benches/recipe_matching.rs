use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_data::{
    item::Item,
    recipes::{CookingRecipeKind, get_cooking_recipe_with_ingredient},
};

fn recipe_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("furnace_recipe_lookup");
    for (name, item) in [
        ("idle_air", &Item::AIR),
        ("iron_ore", &Item::IRON_ORE),
        ("cobblestone", &Item::COBBLESTONE),
        ("invalid_input", &Item::DIAMOND),
    ] {
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(get_cooking_recipe_with_ingredient(
                    black_box(item),
                    CookingRecipeKind::Smelting,
                ))
            });
        });
    }
    group.finish();
}

criterion_group!(benches, recipe_matching);
criterion_main!(benches);
