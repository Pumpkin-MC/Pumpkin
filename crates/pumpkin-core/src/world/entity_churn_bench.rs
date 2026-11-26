//! Spawn/removal benchmark for several entity types, on top of a resident population.
//!
//! Each scenario fills empty chunks with `PUMPKIN_CHURN_POPULATION` entities of one type
//! (default 5 000), then times:
//! - `spawn`: a burst of `PUMPKIN_CHURN_BURST` more (default 2 000), like a farm or a blast;
//! - `kill`: removing that burst, like `/kill` or items being picked up;
//! - `churn`: 50 "ticks" that each spawn and remove 100 entities and read the list once,
//!   like projectiles or drops coming and going while the world ticks.
//!
//! Run it with optimisations, it is ignored by default:
//! `cargo test --release -p pumpkin-core entity_churn_bench -- --ignored --nocapture`

use std::{sync::Arc, time::Instant};

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};
use uuid::Uuid;

use crate::entity::{EntityBase, r#type::from_type};
use crate::world::World;

const CHUNK_RADIUS: i32 = 6;
const CHURN_TICKS: usize = 50;
const CHURN_PER_TICK: usize = 100;

fn env(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn load_chunks(world: &World) {
    let mut active = world
        .active_chunks
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let active = Arc::make_mut(&mut active);
    for cx in -CHUNK_RADIUS..=CHUNK_RADIUS {
        for cz in -CHUNK_RADIUS..=CHUNK_RADIUS {
            world.level.loaded_chunks.insert(
                Vector2::new(cx, cz),
                pumpkin_world::chunk::ChunkData::empty_sync(cx, cz),
            );
            active.insert(Vector2::new(cx, cz));
        }
    }
}

struct Lcg(u64);

impl Lcg {
    fn coord(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let span = f64::from(CHUNK_RADIUS * 16 * 2 - 8);
        ((self.0 >> 33) % 10_000) as f64 / 10_000.0 * span - span / 2.0
    }
}

fn spawn(
    world: &Arc<World>,
    entity_type: &'static EntityType,
    rng: &mut Lcg,
) -> Arc<dyn EntityBase> {
    let pos = Vector3::new(rng.coord(), 64.0, rng.coord());
    let entity = from_type(entity_type, pos, world, Uuid::new_v4());
    world.spawn_entity(entity.clone());
    entity
}

fn ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "benchmark, run with --release --ignored --nocapture"]
#[expect(clippy::print_stderr, reason = "the benchmark reports its timings")]
async fn entity_churn_bench() -> Result<(), Box<dyn std::error::Error>> {
    let population = env("PUMPKIN_CHURN_POPULATION", 5_000);
    let burst = env("PUMPKIN_CHURN_BURST", 2_000);
    let types: [&'static EntityType; 7] = [
        &EntityType::ITEM,
        &EntityType::EXPERIENCE_ORB,
        &EntityType::ARROW,
        &EntityType::SNOWBALL,
        &EntityType::TNT,
        &EntityType::ZOMBIE,
        &EntityType::COW,
    ];
    eprintln!(
        "[churn-bench] population {population}, burst {burst}, churn {CHURN_TICKS}x{CHURN_PER_TICK}"
    );
    for entity_type in types {
        let folder = tempfile::tempdir()?;
        let world = crate::entity::ai::test_support::world(&folder);
        load_chunks(&world);
        let mut rng = Lcg(0x5eed);
        let residents: Vec<_> = (0..population)
            .map(|_| spawn(&world, entity_type, &mut rng))
            .collect();
        let _ = world.entities.load().len();

        let start = Instant::now();
        let spawned: Vec<_> = (0..burst)
            .map(|_| spawn(&world, entity_type, &mut rng))
            .collect();
        let listed = world.entities.load().len();
        let spawn_ms = ms(start);

        let start = Instant::now();
        for entity in &spawned {
            world.remove_entity(entity.as_ref());
        }
        let remaining = world.entities.load().len();
        let kill_ms = ms(start);

        let start = Instant::now();
        for _ in 0..CHURN_TICKS {
            let wave: Vec<_> = (0..CHURN_PER_TICK)
                .map(|_| spawn(&world, entity_type, &mut rng))
                .collect();
            for entity in &wave {
                world.remove_entity(entity.as_ref());
            }
            let _ = world.entities.load().len();
        }
        let churn_ms = ms(start) / CHURN_TICKS as f64;

        assert_eq!(listed, population + burst);
        assert_eq!(remaining, population);
        eprintln!(
            "[churn-bench] {:<15} spawn {spawn_ms:>8.1} ms | kill {kill_ms:>8.1} ms | churn {churn_ms:>6.2} ms/tick",
            entity_type.resource_name,
        );
        // Untimed: release the residents, which hold the world, before the next scenario.
        for entity in &residents {
            world.remove_entity(entity.as_ref());
        }
        assert!(world.entities.load().is_empty());
        drop(residents);
        world.level.shutdown().await;
    }
    Ok(())
}
