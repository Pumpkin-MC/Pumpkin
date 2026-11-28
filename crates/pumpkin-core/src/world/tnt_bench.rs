//! Chain-reaction benchmark: a cube of ~10 000 TNT blocks set off by one explosion, then the
//! primed TNT ticked in entity-list order (the serial path of `World::tick`) until none is left.
//!
//! Run it with optimisations, it is ignored by default:
//! `cargo test --release -p pumpkin-core tnt_chain_bench -- --ignored --nocapture`
//! `PUMPKIN_TNT_BENCH_SIDE` changes the cube's side (default 22, 22×21×22 = 10 164 TNT).

use std::{sync::Arc, time::Instant};

use pumpkin_data::{Block, entity::EntityType};
use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};

use crate::world::{BlockFlags, ExplosionInteraction, World};

/// Half-width, in chunks, of the empty area loaded around the cube.
const CHUNK_RADIUS: i32 = 5;
const MAX_TICKS: u32 = 600;

async fn test_server(world_dir: &std::path::Path) -> Arc<crate::server::Server> {
    use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
    use pumpkin_util::world_seed::Seed;

    let basic_config = BasicConfiguration {
        seed: Seed(123),
        default_level_name: world_dir.to_string_lossy().into_owned(),
        allow_nether: false,
        allow_end: false,
        ..BasicConfiguration::default()
    };
    let mut advanced_config = AdvancedConfiguration::default();
    advanced_config.networking.bedrock.online_mode = false;
    advanced_config.networking.bedrock.authentication.enabled = false;
    let vanilla_data = crate::data::VanillaData {
        banned_ip_list: std::sync::RwLock::new(crate::data::banned_ip::BannedIpList::default()),
        banned_player_list: std::sync::RwLock::new(
            crate::data::banned_player::BannedPlayerList::default(),
        ),
        operator_config: std::sync::RwLock::new(crate::data::op::OperatorConfig::default()),
        user_cache: std::sync::RwLock::new(crate::data::usercache::UserCache::default()),
        whitelist_config: std::sync::RwLock::new(crate::data::whitelist::WhitelistConfig::default()),
    };
    let telemetry_config = TelemetryConfig {
        enabled: false,
        ..TelemetryConfig::default()
    };
    crate::server::Server::new(
        basic_config,
        advanced_config,
        telemetry_config,
        vanilla_data,
        Vec::new(),
    )
    .await
    .expect("the test server must start")
}

/// Loads empty chunks around the origin, marks them ticking (as a player nearby would) and
/// fills a `side`×(`side`-1)×`side` cube of TNT from y = 64. Returns the cube's centre.
fn build_tnt_cube(world: &Arc<World>, side: i32) -> Vector3<f64> {
    {
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
    let tnt = Block::TNT.default_state.id;
    let half = side / 2;
    for x in -half..side - half {
        for y in 64..64 + side - 1 {
            for z in -half..side - half {
                world.set_block_state(&BlockPos::new(x, y, z), tnt, BlockFlags::NOTIFY_LISTENERS);
            }
        }
    }
    world.flush_block_updates();
    Vector3::new(0.5, f64::from(64 + (side - 1) / 2), 0.5)
}

fn live_tnt(world: &World) -> usize {
    world
        .entities
        .load()
        .iter()
        .filter(|e| e.get_entity().entity_type == &EntityType::TNT && !e.get_entity().is_removed())
        .count()
}

/// Ticks every live entity once, in list order, like the serial path of `World::tick`.
fn tick_entities(world: &Arc<World>, server: &crate::server::Server) {
    let entities = world.entities.load_full();
    for entity in entities.iter() {
        let base = entity.get_entity();
        if base.is_removed() {
            continue;
        }
        base.tick_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        entity.tick(entity.as_ref(), server);
    }
    world.flush_block_updates();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "benchmark, run with --release --ignored --nocapture"]
#[expect(clippy::print_stderr, reason = "the benchmark reports its timings")]
async fn tnt_chain_bench() -> Result<(), Box<dyn std::error::Error>> {
    let side: i32 = std::env::var("PUMPKIN_TNT_BENCH_SIDE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(22);
    // A side of 1 makes a 1x0x1 cube: no TNT, no tick to time.
    assert!(
        side >= 2,
        "PUMPKIN_TNT_BENCH_SIDE must be at least 2, got {side}"
    );
    let server_dir = tempfile::tempdir()?;
    let server = test_server(server_dir.path()).await;
    let folder = tempfile::tempdir()?;
    let world = crate::entity::ai::test_support::world(&folder);

    let setup = Instant::now();
    let center = build_tnt_cube(&world, side);
    let blocks = side * (side - 1) * side;
    eprintln!(
        "[tnt-bench] cube {side}x{}x{side} = {blocks} TNT built in {:.2?}",
        side - 1,
        setup.elapsed()
    );

    let start = Instant::now();
    world.explode(center, 4.0, ExplosionInteraction::Tnt);
    let first = start.elapsed();

    let mut tick_ms = Vec::new();
    let mut peak_live = live_tnt(&world);
    for _ in 0..MAX_TICKS {
        if live_tnt(&world) == 0 {
            break;
        }
        let t = Instant::now();
        tick_entities(&world, &server);
        tick_ms.push(t.elapsed().as_secs_f64() * 1000.0);
        peak_live = peak_live.max(live_tnt(&world));
    }
    let total = start.elapsed();
    if tick_ms.is_empty() {
        eprintln!("[tnt-bench] first explosion {first:.2?}; no primed TNT left to tick");
        world.level.shutdown().await;
        return Ok(());
    }
    let mut sorted = tick_ms.clone();
    sorted.sort_by(f64::total_cmp);
    let p = |q: f64| sorted[((sorted.len() - 1) as f64 * q).round() as usize];
    let remaining_tnt_blocks = (-side / 2..side - side / 2)
        .flat_map(|x| (64..64 + side - 1).map(move |y| (x, y)))
        .flat_map(|(x, y)| (-side / 2..side - side / 2).map(move |z| BlockPos::new(x, y, z)))
        .filter(|p| world.get_block(p) == &Block::TNT)
        .count();
    eprintln!(
        "[tnt-bench] first explosion {first:.2?}; {} ticks, total {total:.2?}; tick ms: mean {:.1} p50 {:.1} p95 {:.1} max {:.1}; peak primed {peak_live}; TNT blocks left {remaining_tnt_blocks}; entities left {}",
        tick_ms.len(),
        tick_ms.iter().sum::<f64>() / tick_ms.len().max(1) as f64,
        p(0.5),
        p(0.95),
        p(1.0),
        world.entities.load().len(),
    );
    world.level.shutdown().await;
    Ok(())
}

/// A small cube goes off completely: every TNT block primes, every primed TNT explodes and
/// leaves the entity list and the index.
#[tokio::test(flavor = "multi_thread")]
async fn tnt_cube_chain_reaction_clears_every_block() -> Result<(), Box<dyn std::error::Error>> {
    let side = 6;
    let server_dir = tempfile::tempdir()?;
    let server = test_server(server_dir.path()).await;
    let folder = tempfile::tempdir()?;
    let world = crate::entity::ai::test_support::world(&folder);
    let center = build_tnt_cube(&world, side);
    world.explode(center, 4.0, ExplosionInteraction::Tnt);
    assert!(live_tnt(&world) > 0, "the blast must prime the cube");
    let mut ticks = 0;
    while live_tnt(&world) > 0 {
        tick_entities(&world, &server);
        ticks += 1;
        assert!(ticks < MAX_TICKS, "primed TNT never went off");
    }
    let half = side / 2;
    for x in -half..side - half {
        for y in 64..64 + side - 1 {
            for z in -half..side - half {
                assert_ne!(world.get_block(&BlockPos::new(x, y, z)), &Block::TNT);
            }
        }
    }
    let everything = pumpkin_util::math::boundingbox::BoundingBox::new(
        Vector3::new(-1e6, -1e6, -1e6),
        Vector3::new(1e6, 1e6, 1e6),
    );
    assert!(
        world
            .get_entities_at_box(&everything)
            .iter()
            .all(|e| e.get_entity().entity_type != &EntityType::TNT)
    );
    world.level.shutdown().await;
    Ok(())
}
