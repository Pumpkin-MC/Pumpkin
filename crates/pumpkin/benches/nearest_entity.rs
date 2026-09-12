#![allow(clippy::expect_used)]

#[path = "../tests/support/mod.rs"]
mod support;

use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin::entity::{Entity, EntityBase};
use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;
use std::{collections::HashMap, hint::black_box, sync::Arc};

// The master implementation is retained to measure the cost of collecting candidates.
fn collected_search(
    world: &pumpkin::world::World,
    pos: Vector3<f64>,
    radius: f64,
    entity_types: Option<&[&'static EntityType]>,
) -> Option<Arc<dyn EntityBase>> {
    let entities = world.get_nearby_entities(pos, radius);
    let filtered_entities = if let Some(types) = entity_types {
        entities
            .into_iter()
            .filter(|(_, entity)| types.contains(&entity.get_entity().entity_type))
            .collect::<HashMap<_, _>>()
    } else {
        entities
    };
    filtered_entities
        .iter()
        .min_by(|a, b| {
            a.1.get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&pos)
                .total_cmp(&b.1.get_entity().pos.load().squared_distance_to_vec(&pos))
        })
        .map(|p| p.1.clone())
}

fn nearest_entity(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let _guard = runtime.enter();
    let dir = tempfile::tempdir().expect("temporary world");
    let world = support::world(dir.path());
    let mut group = c.benchmark_group("nearest_entity");
    for (layout, spacing) in [("nearby", 1.0), ("spread", 16.0)] {
        for count in [100, 1_000, 10_000] {
            let entities: Vec<Arc<dyn EntityBase>> = (0..count)
                .map(|i| {
                    let kind = if i % 4 == 0 {
                        &EntityType::COW
                    } else {
                        &EntityType::ITEM
                    };
                    Arc::new(Entity::new(
                        world.clone(),
                        Vector3::new(
                            f64::from(i % 32) * spacing,
                            64.0,
                            f64::from(i / 32 % 32) * spacing,
                        ),
                        kind,
                    )) as Arc<dyn EntityBase>
                })
                .collect();
            world.entities.store(Arc::new(entities));
            for (name, types) in [("all", None), ("cow", Some([&EntityType::COW].as_slice()))] {
                let position = Vector3::new(16.0 * spacing, 64.0, 16.0 * spacing);
                let distance = |entity: Option<Arc<dyn EntityBase>>| {
                    entity.map(|entity| {
                        entity
                            .get_entity()
                            .pos
                            .load()
                            .squared_distance_to_vec(&position)
                    })
                };
                assert_eq!(
                    distance(collected_search(&world, position, 16.0, types)),
                    distance(world.get_closest_entity(position, 16.0, types)),
                );
                group.bench_function(format!("collected/{layout}/{name}/{count}"), |b| {
                    b.iter(|| {
                        black_box(collected_search(
                            &world,
                            black_box(Vector3::new(16.0 * spacing, 64.0, 16.0 * spacing)),
                            black_box(16.0),
                            black_box(types),
                        ))
                    });
                });
                group.bench_function(format!("streamed/{layout}/{name}/{count}"), |b| {
                    b.iter(|| {
                        black_box(world.get_closest_entity(
                            black_box(Vector3::new(16.0 * spacing, 64.0, 16.0 * spacing)),
                            black_box(16.0),
                            black_box(types),
                        ))
                    });
                });
            }
        }
    }
    group.finish();
    world.entities.store(Arc::new(Vec::new()));
    runtime.block_on(world.level.shutdown());
}

criterion_group!(benches, nearest_entity);
criterion_main!(benches);
