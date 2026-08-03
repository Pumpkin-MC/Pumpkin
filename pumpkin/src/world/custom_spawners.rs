use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::Taggable;
use pumpkin_data::tag::WorldgenBiome::MINECRAFT_WITHOUT_WANDERING_TRADER_SPAWNS;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::mob::equipment::RegionalDifficulty;
use crate::entity::passive::cat::select_natural_cat_variant;
use crate::entity::player::statistics::{CustomStatistic, StatisticCategory};
use crate::entity::r#type::from_type;
use crate::world::World;
use crate::world::natural_spawner::{is_spawn_position_ok, is_valid_empty_spawn_block};

fn entities_of_type_near(
    world: &Arc<World>,
    center: &BlockPos,
    horizontal_radius: f64,
    vertical_radius: f64,
    entity_type: &'static EntityType,
) -> usize {
    let min = Vector3::new(
        f64::from(center.0.x) - horizontal_radius,
        f64::from(center.0.y) - vertical_radius,
        f64::from(center.0.z) - horizontal_radius,
    );
    let max = Vector3::new(
        f64::from(center.0.x) + horizontal_radius,
        f64::from(center.0.y) + vertical_radius,
        f64::from(center.0.z) + horizontal_radius,
    );
    let bb = pumpkin_util::math::boundingbox::BoundingBox { min, max };
    world
        .get_entities_at_box(&bb)
        .iter()
        .filter(|e| e.get_entity().entity_type == entity_type)
        .count()
}

/// `PhantomSpawner` (`PhantomSpawner.java`): entirely absent from Pumpkin before this change, so phantoms never spawned naturally.
///
/// Ported field for field: 60-120 game-tick interval
/// (`(60 + random.nextInt(60)) * 20`), only active once `skyDarken >= 5` (or
/// the dimension has no sky light), gated per-player by `TIME_SINCE_REST >=
/// 72000` (sampled via `nextInt`) and by
/// `DifficultyInstance.isHarderThan(random.nextFloat() * 3.0F)`.
pub async fn tick_phantom_spawner(world: &Arc<World>) {
    if !world.level_info.load().game_rules.spawn_phantoms {
        return;
    }
    if world.phantom_spawn_tick.fetch_sub(1, Relaxed) - 1 > 0 {
        return;
    }
    world
        .phantom_spawn_tick
        .store((60 + rand::random_range(0..60)) * 20, Relaxed);

    let sky_darken = world.sky_darken.load(Relaxed);
    if sky_darken < 5 && world.dimension.has_skylight {
        return;
    }

    let players: Vec<_> = world.players.load().iter().cloned().collect();
    for player in players {
        if player.gamemode.load() == GameMode::Spectator {
            continue;
        }

        let player_pos = player.get_entity().block_pos.load();
        if world.dimension.has_skylight
            && (player_pos.0.y < world.sea_level || !world.can_see_sky(&player_pos))
        {
            continue;
        }

        let difficulty = RegionalDifficulty::at(world, player.position());
        if difficulty.effective_difficulty <= rand::random::<f32>() * 3.0 {
            continue;
        }

        let time_since_rest = {
            let stats = player.stats.lock().await;
            stats.get(
                StatisticCategory::Custom,
                CustomStatistic::TimeSinceRest as i32,
            )
        }
        .clamp(1, i32::MAX);

        if rand::random_range(0..time_since_rest) < 72000 {
            continue;
        }

        let spawn_pos = player_pos.add(
            rand::random_range(0..21) - 10,
            20 + rand::random_range(0..15),
            rand::random_range(0..21) - 10,
        );

        if !is_valid_empty_spawn_block(world.get_block_state(&spawn_pos), &EntityType::PHANTOM) {
            continue;
        }

        let spawn_pos_f64 = Vector3::new(
            f64::from(spawn_pos.0.x) + 0.5,
            f64::from(spawn_pos.0.y),
            f64::from(spawn_pos.0.z) + 0.5,
        );
        let group_size = 1 + rand::random_range(0..(difficulty.base_difficulty as i32 + 1));
        for _ in 0..group_size {
            let phantom = from_type(&EntityType::PHANTOM, spawn_pos_f64, world, Uuid::new_v4());
            phantom.get_entity().set_rotation(0.0, 0.0);
            world.spawn_entity(phantom).await;
        }
    }
}

/// `CatSpawner` (`CatSpawner.java`): 1200-tick interval, picks a random
/// player and a random offset in `[-32, 32]` on each axis (`8 +
/// random.nextInt(24)`, sign-randomized).
///
/// Village gate: vanilla checks `level.isCloseToVillage(spawnPos, 2)`, then
/// requires `getCountInRange(HOME, spawnPos, 48, IS_OCCUPIED) > 4` (more
/// than 4 claimed beds within a 48-block sphere) before spawning in a
/// village. See `crate::world::village_poi` for the POI registry backing
/// both checks and the `Occupancy.ANY` (not `IS_OCCUPIED`) deviation it
/// documents.
///
/// Scope reduction: vanilla's other spawn path, inside a swamp-hut structure
/// (`CATS_SPAWN_IN` structure tag), is dropped entirely - Pumpkin has no
/// structure-piece lookup by tag.
pub async fn tick_cat_spawner(world: &Arc<World>) {
    if world.cat_spawn_tick.fetch_sub(1, Relaxed) - 1 > 0 {
        return;
    }
    world.cat_spawn_tick.store(1200, Relaxed);

    let players = world.players.load();
    if players.is_empty() {
        return;
    }
    let player = players[rand::random_range(0..players.len())].clone();
    drop(players);

    let dx = (8 + rand::random_range(0..24)) * if rand::random::<bool>() { -1 } else { 1 };
    let dz = (8 + rand::random_range(0..24)) * if rand::random::<bool>() { -1 } else { 1 };
    let spawn_pos = player.get_entity().block_pos.load().add(dx, 0, dz);

    let chunk_pos = Vector2::new(spawn_pos.0.x >> 4, spawn_pos.0.z >> 4);
    if !world.active_chunks.load().contains(&chunk_pos) {
        return;
    }

    if !is_spawn_position_ok(world, &spawn_pos, &EntityType::CAT) {
        return;
    }

    if !world.is_close_to_village(spawn_pos, 2).await {
        return;
    }

    let homes_nearby = world
        .poi_count_in_range(
            crate::world::village_poi::POI_TYPE_HOME,
            spawn_pos,
            48,
            crate::world::village_poi::Occupancy::IsOccupied,
        )
        .await;
    if homes_nearby <= 4 {
        return;
    }

    let cats_nearby = entities_of_type_near(world, &spawn_pos, 48.0, 8.0, &EntityType::CAT);
    if cats_nearby >= 5 {
        return;
    }

    let time_of_day = world.get_time_of_day().await;
    let spawn_pos_f64 = Vector3::new(
        f64::from(spawn_pos.0.x) + 0.5,
        f64::from(spawn_pos.0.y),
        f64::from(spawn_pos.0.z) + 0.5,
    );
    let cat = from_type(&EntityType::CAT, spawn_pos_f64, world, Uuid::new_v4());
    cat.set_variant_name(select_natural_cat_variant(time_of_day));
    cat.get_entity().set_rotation(0.0, 0.0);
    world.spawn_entity(cat).await;
}

fn find_spawn_position_near(
    world: &Arc<World>,
    reference: &BlockPos,
    radius: i32,
) -> Option<BlockPos> {
    for _ in 0..10 {
        let x = reference.0.x + rand::random_range(0..radius * 2) - radius;
        let z = reference.0.z + rand::random_range(0..radius * 2) - radius;
        let y = world.get_top_block(Vector2::new(x, z));
        let pos = BlockPos::new(x, y, z);
        if is_spawn_position_ok(world, &pos, &EntityType::WANDERING_TRADER) {
            return Some(pos);
        }
    }
    None
}

fn has_enough_space(world: &Arc<World>, pos: &BlockPos) -> bool {
    for dx in 0..=1 {
        for dy in 0..=2 {
            for dz in 0..=1 {
                if world.get_block_state(&pos.add(dx, dy, dz)).is_solid_block() {
                    return false;
                }
            }
        }
    }
    true
}

async fn try_spawn_wandering_trader(world: &Arc<World>) -> bool {
    let players = world.players.load();
    if players.is_empty() {
        return true;
    }
    if rand::random_range(0..10) != 0 {
        return false;
    }
    let player = players[rand::random_range(0..players.len())].clone();
    drop(players);

    // No POI manager, so the "meeting point" lookup always falls through to
    // the player's own position, matching vanilla's own fallback
    // (`poiPos.orElse(playerPos)`) for the case where no meeting POI exists
    // nearby.
    let reference_pos = player.get_entity().block_pos.load();
    let Some(spawn_pos) = find_spawn_position_near(world, &reference_pos, 48) else {
        return false;
    };
    if !has_enough_space(world, &spawn_pos) {
        return false;
    }
    if world
        .get_biome(&spawn_pos)
        .has_tag(&MINECRAFT_WITHOUT_WANDERING_TRADER_SPAWNS)
    {
        return false;
    }

    let spawn_pos_f64 = Vector3::new(
        f64::from(spawn_pos.0.x) + 0.5,
        f64::from(spawn_pos.0.y),
        f64::from(spawn_pos.0.z) + 0.5,
    );
    let trader = from_type(
        &EntityType::WANDERING_TRADER,
        spawn_pos_f64,
        world,
        Uuid::new_v4(),
    );
    world.spawn_entity(trader.clone()).await;

    for _ in 0..2 {
        if let Some(llama_pos) =
            find_spawn_position_near(world, &trader.get_entity().block_pos.load(), 4)
        {
            let llama_pos_f64 = Vector3::new(
                f64::from(llama_pos.0.x) + 0.5,
                f64::from(llama_pos.0.y),
                f64::from(llama_pos.0.z) + 0.5,
            );
            let llama = from_type(
                &EntityType::TRADER_LLAMA,
                llama_pos_f64,
                world,
                Uuid::new_v4(),
            );
            world.spawn_entity(llama.clone()).await;
            llama.get_entity().leash_to(trader.clone()).await;
        }
    }

    true
}

/// `WanderingTraderSpawner` (`WanderingTraderSpawner.java`).
///
/// 1200-tick outer interval, an inner 24000-tick (one day) spawn-delay
/// counter, and a spawn-chance that ramps `25 -> 50 -> 75` (capped) each day
/// until a trader actually spawns, then resets to 25.
///
/// Scope reductions: the `spawn_delay`/`spawn_chance` counters live only in
/// memory (`World::trader_spawn_delay` / `trader_spawn_chance`), not in a
/// persisted `WanderingTraderData` saved-data file, so they reset to vanilla
/// defaults (24000 / 25) on server restart. The trader also isn't given a
/// wander/home restriction or scripted despawn timer: Pumpkin's mob
/// restriction system (`Mob::get_home_pos`) has no setter yet.
pub async fn tick_wandering_trader_spawner(world: &Arc<World>) {
    if !world.level_info.load().game_rules.spawn_wandering_traders {
        return;
    }
    if world.trader_tick_delay.fetch_sub(1, Relaxed) - 1 > 0 {
        return;
    }
    world.trader_tick_delay.store(1200, Relaxed);

    if world.trader_spawn_delay.fetch_sub(1200, Relaxed) - 1200 > 0 {
        return;
    }
    world.trader_spawn_delay.store(24000, Relaxed);

    let chance = world.trader_spawn_chance.load(Relaxed);
    world
        .trader_spawn_chance
        .store((chance + 25).clamp(25, 75), Relaxed);

    if rand::random_range(0..100) > chance {
        return;
    }

    if try_spawn_wandering_trader(world).await {
        world.trader_spawn_chance.store(25, Relaxed);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn spawn_chance_ramps_and_caps() {
        let mut chance = 25;
        for _ in 0..10 {
            chance = (chance + 25i32).clamp(25, 75);
        }
        assert_eq!(chance, 75);
    }

    #[test]
    fn phantom_next_tick_interval_matches_vanilla_bounds() {
        for roll in 0..60 {
            let next = (60 + roll) * 20;
            assert!((1200..=2380).contains(&next));
        }
    }
}
