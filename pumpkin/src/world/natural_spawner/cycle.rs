use crate::entity::EntityBase;
use crate::entity::r#type::from_type;
use crate::world::World;
use pumpkin_data::chunk::Biome;
use pumpkin_data::entity::{EntityType, MobCategory};
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomImpl, legacy_rand::LegacyRand};
use pumpkin_world::chunk::{ChunkData, ChunkHeightmapType};
use pumpkin_world::generation::proto_chunk::GenerationCache;
use rand::{RngExt, rng};
use std::sync::Arc;
use uuid::Uuid;

use super::rules::{
    adjust_spawn_position_cache, choose_weighted_spawner_with_random_impl, get_nearest_player,
    get_random_spawn_mob_at_with_random, is_redstone_conductor,
    is_right_distance_to_player_and_spawn_point, is_spawn_position_ok_cache,
    is_valid_spawn_position_for_type, spawner_is_in_biome_pool,
};
use super::state::SpawnState;

/// Vanilla `NaturalSpawner.getFilteredSpawningCategories(state, spawnEnemies, spawnPersistent)`.
///
/// - Hostile categories need `spawn_enemies`
/// - Persistent categories (CREATURE, WATER_CREATURE, …) need `spawn_persistent` (~every 400 ticks)
/// - Friendly non-persistent always allowed when global cap allows
#[must_use]
pub fn get_filtered_spawning_categories(
    state: &SpawnState,
    spawn_enemies: bool,
    spawn_persistent: bool,
) -> Vec<&'static MobCategory> {
    let mut ret = Vec::with_capacity(MobCategory::SPAWNING_CATEGORIES.len());
    for category in MobCategory::SPAWNING_CATEGORIES {
        // if (!spawnEnemies && !isFriendly) continue
        if !spawn_enemies && !category.is_friendly {
            continue;
        }
        // if (!spawnPersistent && isPersistent) continue
        if !spawn_persistent && category.is_persistent {
            continue;
        }
        if state.can_spawn_for_category_global(category) {
            ret.push(category);
        }
    }
    ret
}

/// Vanilla `NaturalSpawner.spawnForChunk` — one pack attempt per category per chunk.
pub fn spawn_for_chunk(
    world: &Arc<World>,
    chunk_pos: Vector2<i32>,
    chunk: &Arc<ChunkData>,
    spawn_state: &SpawnState,
    spawn_list: &Vec<&'static MobCategory>,
    is_thundering: bool,
) -> Vec<Arc<dyn EntityBase>> {
    let mut entities = Vec::new();
    for category in spawn_list {
        if !spawn_state.can_spawn_for_category_local(world, category, chunk_pos) {
            continue;
        }
        // Vanilla spawnCategoryForChunk: single getRandomPosWithin + spawnCategoryForPosition.
        let start = get_random_pos_within(world.min_y, &chunk_pos, chunk);
        if start.0.y < world.min_y + 1 {
            continue;
        }
        let batch = spawn_category_for_position(
            category,
            world,
            start,
            &chunk_pos,
            spawn_state,
            is_thundering,
        );
        if pumpkin_config::development_mode()
            && category.id == MobCategory::CREATURE.id
            && !batch.is_empty()
        {
            tracing::info!(
                "natural creature spawn: {} at {:?} chunk {:?}",
                batch.len(),
                start,
                chunk_pos
            );
        }
        entities.extend(batch);
    }
    entities
}

/// Vanilla `NaturalSpawner.getRandomPosWithin`:
/// `x,z` random in chunk; `y = randomInclusive(minY, WORLD_SURFACE+1)`.
pub fn get_random_pos_within(
    min_y: i32,
    chunk_pos: &Vector2<i32>,
    chunk: &Arc<ChunkData>,
) -> BlockPos {
    let mut rng = rng();

    let x = (chunk_pos.x << 4) + rng.random_range(0..16);
    let z = (chunk_pos.y << 4) + rng.random_range(0..16);
    let top_empty_y = chunk.heightmap.lock().unwrap().get(
        ChunkHeightmapType::WorldSurface,
        x,
        z,
        chunk.section.min_y,
    ) + 1;
    let top = top_empty_y.max(min_y + 1);
    let y = rng.random_range(min_y..=top);
    BlockPos::new(x, y, z)
}

pub fn spawn_mobs_for_chunk_generation(
    world: &Arc<World>,
    cache: &mut dyn GenerationCache,
    biome: &'static Biome,
    chunk_x: i32,
    chunk_z: i32,
) {
    let mob_settings = &biome.spawners;
    let creatures = &mob_settings.creature;

    if creatures.is_empty() || !world.level_info.load().game_rules.spawn_mobs {
        return;
    }

    let xo = chunk_x << 4;
    let zo = chunk_z << 4;

    // Vanilla `WorldgenRandom#setDecorationSeed` uses the legacy RNG, seeded
    // from the world seed and chunk block origin. Generation must not depend on
    // process-local entropy or chunk scheduling order.
    let mut random =
        LegacyRand::from_seed(LegacyRand::get_population_seed(world.level.seed.0, xo, zo));
    while random.next_f32() < biome.creature_spawn_probability {
        let Some(spawner_data) = choose_weighted_spawner_with_random_impl(creatures, &mut random)
        else {
            continue;
        };

        let count = random.next_inbetween_i32(spawner_data.min_count, spawner_data.max_count);
        let Some(entity_type) = EntityType::from_name(
            spawner_data
                .r#type
                .strip_prefix("minecraft:")
                .unwrap_or(spawner_data.r#type),
        ) else {
            continue;
        };

        let mut x = xo + random.next_bounded_i32(16);
        let mut z = zo + random.next_bounded_i32(16);
        let start_x = x;
        let start_z = z;

        for _ in 0..count {
            let mut success = false;

            // Try 4 times to find a valid spot in the immediate area
            for _ in 0..4 {
                if success {
                    break;
                }

                let pos = get_top_non_colliding_pos(world, cache, entity_type, x, z);

                if is_spawn_position_ok_cache(cache, &pos, entity_type) {
                    let spawn_pos_f64 = Vector3::new(
                        f64::from(pos.0.x) + 0.5,
                        f64::from(pos.0.y),
                        f64::from(pos.0.z) + 0.5,
                    );

                    let entity = from_type(entity_type, spawn_pos_f64, world, Uuid::new_v4());
                    entity
                        .get_entity()
                        .set_rotation(random.next_f32() * 360., 0.);
                    world.spawn_entity_non_save(&entity);
                    success = true;
                }

                // Random jitter for the next mob in the group
                x += random.next_bounded_i32(5) - random.next_bounded_i32(5);
                z += random.next_bounded_i32(5) - random.next_bounded_i32(5);

                // Vanilla retries the jitter from the group's origin until it lands in chunk.
                while x < xo || x >= xo + 16 || z < zo || z >= zo + 16 {
                    x = start_x + random.next_bounded_i32(5) - random.next_bounded_i32(5);
                    z = start_z + random.next_bounded_i32(5) - random.next_bounded_i32(5);
                }
            }
        }
    }
}

pub fn get_top_non_colliding_pos(
    world: &World,
    cache: &dyn GenerationCache,
    entity_type: &'static EntityType,
    x: i32,
    z: i32,
) -> BlockPos {
    let mut y = cache.get_top_y(&entity_type.spawn_restriction.heightmap, x, z);
    let mut pos_vec = Vector3::new(x, y, z);
    let min_y = world.min_y;

    if world.dimension.has_ceiling {
        loop {
            y -= 1;
            pos_vec.y = y;
            // Use UFCS to avoid the ambiguity error from earlier
            if GenerationCache::get_block_state(cache, &pos_vec)
                .to_state()
                .is_air()
                || y <= min_y
            {
                break;
            }
        }

        loop {
            y -= 1;
            pos_vec.y = y;
            if !GenerationCache::get_block_state(cache, &pos_vec)
                .to_state()
                .is_air()
                || y <= min_y
            {
                break;
            }
        }
    }

    let pos = BlockPos::new(x, y, z);

    adjust_spawn_position_cache(cache, pos, entity_type)
}

pub fn spawn_category_for_position(
    category: &'static MobCategory,
    world: &Arc<World>,
    pos: BlockPos,
    chunk_pos: &Vector2<i32>,
    spawn_state: &SpawnState,
    is_thundering: bool,
) -> Vec<Arc<dyn EntityBase>> {
    let mut batch_buffer = vec![];
    let mut spawn_cluster_size = 0;
    let player_positions: Vec<_> = world
        .players
        .load()
        .iter()
        .filter(|p| p.gamemode.load() != GameMode::Spectator)
        .map(|p| p.position())
        .collect();
    if player_positions.is_empty() {
        return batch_buffer;
    }

    let world_spawn = {
        let info = world.level_info.load();
        Vector3::new(
            f64::from(info.spawn_x) + 0.5,
            f64::from(info.spawn_y),
            f64::from(info.spawn_z) + 0.5,
        )
    };

    // Vanilla: if (chunk.getBlockState(start).isRedstoneConductor(...)) return;
    let start_state = world.get_block_state(&pos);
    if is_redstone_conductor(start_state) {
        return batch_buffer;
    }

    let mut random = rng();
    'group_loop: for _ in 0..3 {
        let mut new_x = pos.0.x;
        let mut new_z = pos.0.z;

        let mut random_group_size = (random.random::<f32>() * 4.).ceil() as i32;
        let mut inc = 0;
        let mut current_spawner = None;

        while inc < random_group_size {
            new_x += random.random_range(0..6) - random.random_range(0..6);
            new_z += random.random_range(0..6) - random.random_range(0..6);
            // Vanilla keeps pack Y at yStart (no vertical crawl).
            let new_pos = BlockPos::new(new_x, pos.0.y, new_z);
            let spawn_pos_f64 = Vector3::new(
                f64::from(new_pos.0.x) + 0.5,
                f64::from(new_pos.0.y),
                f64::from(new_pos.0.z) + 0.5,
            );
            let player_distance = get_nearest_player(&spawn_pos_f64, &player_positions);
            if !is_right_distance_to_player_and_spawn_point(
                world,
                &new_pos,
                player_distance,
                chunk_pos,
                world_spawn,
            ) {
                inc += 1;
                continue;
            }

            if current_spawner.is_none() {
                let Some(spawner) =
                    get_random_spawn_mob_at_with_random(world, category, &new_pos, &mut random)
                else {
                    continue 'group_loop;
                };
                current_spawner = Some(spawner);
                random_group_size = random.random_range(spawner.min_count..=spawner.max_count);
            }

            let spawner = current_spawner.unwrap();
            let Some(entity_type) = EntityType::from_name(
                spawner
                    .r#type
                    .strip_prefix("minecraft:")
                    .unwrap_or(spawner.r#type),
            ) else {
                inc += 1;
                continue;
            };

            if !spawner_is_in_biome_pool(world.level.get_rough_biome(&new_pos), category, spawner) {
                inc += 1;
                continue;
            }

            if !is_valid_spawn_position_for_type(
                world,
                &new_pos,
                category,
                entity_type,
                player_distance,
                is_thundering,
            ) {
                inc += 1;
                continue;
            }
            if !spawn_state.can_spawn(entity_type, &new_pos, world) {
                inc += 1;
                continue;
            }

            let entity = from_type(entity_type, spawn_pos_f64, world, Uuid::new_v4());
            entity
                .get_entity()
                .set_rotation(random.random::<f32>() * 360., 0.);

            spawn_cluster_size += 1;
            spawn_state.after_spawn(entity.as_ref(), world);
            batch_buffer.push(entity);
            if spawn_cluster_size >= entity_type.limit_per_chunk {
                break 'group_loop;
            }

            inc += 1;
        }
    }
    batch_buffer
}

/// Vanilla `Zombie.finalizeSpawn` jockey roll: 5% of baby zombies start riding
/// a chicken. Called for freshly spawned natural entities only.
pub async fn try_spawn_chicken_jockey(world: &Arc<World>, entity: &Arc<dyn EntityBase>) {
    use crate::entity::mob::zombie::ZombieEntityBase;
    use crate::entity::mob::zombie::drowned::DrownedEntity;
    use crate::entity::mob::zombie::husk::HuskEntity;
    use crate::entity::mob::zombie::zombie::ZombieEntity;
    use crate::entity::mob::zombie::zombie_villager::ZombieVillagerEntity;

    let any = entity.cast_any();
    let zombie_base: Option<&ZombieEntityBase> = any
        .downcast_ref::<ZombieEntity>()
        .map(|z| z.entity.as_ref())
        .or_else(|| any.downcast_ref::<HuskEntity>().map(|z| z.entity.as_ref()))
        .or_else(|| {
            any.downcast_ref::<DrownedEntity>()
                .map(|z| z.entity.as_ref())
        })
        .or_else(|| {
            any.downcast_ref::<ZombieVillagerEntity>()
                .map(|z| z.mob_entity.as_ref())
        });
    let Some(zombie_base) = zombie_base else {
        return;
    };
    if !zombie_base
        .is_baby
        .load(std::sync::atomic::Ordering::Relaxed)
        || rand::random::<f32>() >= 0.05
    {
        return;
    }

    let pos = entity.get_entity().pos.load();
    let chicken = crate::entity::r#type::from_type(
        &pumpkin_data::entity::EntityType::CHICKEN,
        pos,
        world,
        uuid::Uuid::new_v4(),
    );
    world.spawn_entity(chicken.clone()).await;
    chicken
        .get_entity()
        .add_passenger(chicken.clone(), entity.clone())
        .await;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::natural_spawner as public_api;

    // Compile-time assertions that the public paths and signatures survived the
    // module split (re-exported through `crate::world::natural_spawner`).
    const _: fn(&public_api::SpawnState, bool, bool) -> Vec<&'static MobCategory> =
        public_api::get_filtered_spawning_categories;
    const _: fn(i32, &Vector2<i32>, &Arc<ChunkData>) -> BlockPos =
        public_api::get_random_pos_within;
    const _: fn(&World, &dyn GenerationCache, &'static EntityType, i32, i32) -> BlockPos =
        public_api::get_top_non_colliding_pos;
    const _: fn(&Arc<World>, &mut dyn GenerationCache, &'static Biome, i32, i32) =
        public_api::spawn_mobs_for_chunk_generation;

    #[test]
    fn filtered_categories_respect_global_caps_and_flags() {
        let mut state = SpawnState::empty();
        // Zero spawnable chunks: every per-category global cap is zero.
        assert!(get_filtered_spawning_categories(&state, true, true).is_empty());

        // 17 * 17 spawnable chunks make each category limit exactly `category.max`.
        state.set_spawnable_chunk_count(17 * 17);
        let has = |list: &[&'static MobCategory], category: &'static MobCategory| {
            list.iter().any(|c| c.id == category.id)
        };

        let all = get_filtered_spawning_categories(&state, true, true);
        assert!(has(&all, &MobCategory::MONSTER));
        assert!(has(&all, &MobCategory::CREATURE));
        // MISC has a negative cap and never naturally spawns.
        assert!(!has(&all, &MobCategory::MISC));

        let peaceful = get_filtered_spawning_categories(&state, false, true);
        assert!(!has(&peaceful, &MobCategory::MONSTER));
        assert!(has(&peaceful, &MobCategory::CREATURE));

        let transient = get_filtered_spawning_categories(&state, true, false);
        assert!(has(&transient, &MobCategory::MONSTER));
        assert!(!has(&transient, &MobCategory::CREATURE));
    }
}
