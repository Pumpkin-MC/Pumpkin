use crate::entity::r#type::{check_spawn_rules, from_type};
use crate::entity::{Entity, EntityBase};
use crate::world::World;
use pumpkin_data::biome::Spawner;
use pumpkin_data::chunk::Biome;
use pumpkin_data::entity::{EntityType, MobCategory, SpawnLocation};
use pumpkin_data::tag::Block::{
    MINECRAFT_BUTTONS, MINECRAFT_FIRE, MINECRAFT_FOX_IMMUNE_TO, MINECRAFT_POLAR_BEAR_IMMUNE_TO,
    MINECRAFT_PRESSURE_PLATES, MINECRAFT_PREVENT_MOB_SPAWNING_INSIDE,
    MINECRAFT_SNOW_GOLEM_IMMUNE_TO, MINECRAFT_STRAY_IMMUNE_TO, MINECRAFT_WITHER_IMMUNE_TO,
    MINECRAFT_WITHER_SKELETON_IMMUNE_TO,
};
use pumpkin_data::tag::Fluid::{MINECRAFT_LAVA, MINECRAFT_WATER};
use pumpkin_data::tag::Taggable;
use pumpkin_data::tag::WorldgenBiome::MINECRAFT_REDUCE_WATER_AMBIENT_SPAWNS;
use pumpkin_data::{Block, BlockDirection, BlockId, BlockState};
use pumpkin_util::GameMode;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::get_section_cord;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomImpl, legacy_rand::LegacyRand};
use pumpkin_world::chunk::{ChunkData, ChunkHeightmapType};
use pumpkin_world::generation::proto_chunk::GenerationCache;
use rand::{Rng, RngExt, rng};
use rustc_hash::FxHashSet;
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

const MAGIC_NUMBER: i32 = 17 * 17;

/// Vanilla `DistanceManager` natural-spawn tracker radius
/// (`FixedPlayerDistanceChunkTracker(8)`). Independent of simulation distance.
pub const NATURAL_SPAWN_CHUNK_RANGE: i32 = 8;

/// Vanilla `NaturalSpawner.SPAWN_DISTANCE_BLOCK` squared
/// (`ChunkMap.playerIsCloseEnoughForSpawning` uses 128² = 16384).
pub const SPAWN_DISTANCE_BLOCK_SQ: f64 = 128.0 * 128.0;

/// Java's `BlockState.isRedstoneConductor` is represented by this cached block-state flag.
#[must_use]
fn is_redstone_conductor(state: &BlockState) -> bool {
    state.is_solid_block()
}

/// Vanilla `Mob.isPersistenceRequired() || Mob.requiresCustomPersistence()`.
///
/// These locks must remain non-blocking: `SpawnState` is rebuilt before entity
/// ticks, and a missed lock only makes a cap conservative for one tick.
#[must_use]
fn requires_custom_persistence(entity: &Entity) -> bool {
    entity.custom_name.load().is_some()
        || entity
            .leashed_to
            .try_lock()
            .is_ok_and(|holder| holder.is_some())
        || entity
            .vehicle
            .try_lock()
            .is_ok_and(|vehicle| vehicle.is_some())
}

/// Whether this entity contributes to the current natural-spawn cap state.
///
/// Vanilla builds `SpawnState` from every mob in a loaded full chunk, not only
/// chunks within simulation distance. Keep dynamic additions on the same rule;
/// removals are instead gated by the per-state accounting set so a mob that
/// becomes persistent (or unloads) during the tick is still removed exactly
/// once.
#[must_use]
fn counts_towards_spawn_cap(entity: &Entity, world: &World) -> bool {
    let entity_type = entity.entity_type;
    entity_type.mob
        && entity_type.category != &MobCategory::MISC
        && !requires_custom_persistence(entity)
        && world.level.is_chunk_loaded(&entity.chunk_pos.load())
}

/// Vanilla `ChunkMap.anyPlayerCloseEnoughForSpawningInternal`.
#[must_use]
pub fn any_player_close_enough_for_spawning(world: &World, chunk_pos: Vector2<i32>) -> bool {
    let chunk_center_x = ((chunk_pos.x << 4) + 8) as f64;
    let chunk_center_z = ((chunk_pos.y << 4) + 8) as f64;
    for player in world.players.load().iter() {
        if player.gamemode.load() == GameMode::Spectator {
            continue;
        }
        let pos = player.position();
        let dx = chunk_center_x - pos.x;
        let dz = chunk_center_z - pos.z;
        if dx * dx + dz * dz < SPAWN_DISTANCE_BLOCK_SQ {
            return true;
        }
    }
    false
}

/// Whether `chunk_pos` is inside vanilla natural-spawn candidate range
/// (`FixedPlayerDistanceChunkTracker` radius 8 around any player chunk).
#[must_use]
pub fn is_natural_spawn_candidate(world: &World, chunk_pos: Vector2<i32>) -> bool {
    let spectators_generate_chunks = world
        .level_info
        .load()
        .game_rules
        .spectators_generate_chunks;
    for player in world.players.load().iter() {
        if player.gamemode.load() == GameMode::Spectator && !spectators_generate_chunks {
            continue;
        }
        let center = player.get_entity().chunk_pos.load();
        if (chunk_pos.x - center.x)
            .abs()
            .max((chunk_pos.y - center.y).abs())
            <= NATURAL_SPAWN_CHUNK_RANGE
        {
            return true;
        }
    }
    false
}

use dashmap::DashMap;
use std::sync::atomic::{AtomicI32, Ordering::Relaxed};

pub struct MobCounts([AtomicI32; 8]);

impl Default for MobCounts {
    fn default() -> Self {
        Self(std::array::from_fn(|_| AtomicI32::new(0)))
    }
}

impl fmt::Debug for MobCounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|a| a.load(Relaxed)))
            .finish()
    }
}

impl Clone for MobCounts {
    fn clone(&self) -> Self {
        Self(std::array::from_fn(|i| {
            AtomicI32::new(self.0[i].load(Relaxed))
        }))
    }
}

impl MobCounts {
    #[inline]
    pub fn add(&self, category: &'static MobCategory) {
        self.0[category.id].fetch_add(1, Relaxed);
    }

    #[inline]
    pub fn remove(&self, category: &'static MobCategory) {
        self.0[category.id].fetch_sub(1, Relaxed);
    }
    #[inline]
    pub fn can_spawn(&self, category: &'static MobCategory) -> bool {
        self.0[category.id].load(Relaxed) < category.max
    }
}

pub struct LocalMobCapCalculator {
    player_mob_counts: DashMap<i32, MobCounts>,
    players_near_chunk: DashMap<Vector2<i32>, Vec<i32>>,
}

impl Clone for LocalMobCapCalculator {
    fn clone(&self) -> Self {
        let player_mob_counts = DashMap::new();
        for r in &self.player_mob_counts {
            player_mob_counts.insert(*r.key(), r.value().clone());
        }
        let players_near_chunk = DashMap::new();
        for r in &self.players_near_chunk {
            players_near_chunk.insert(*r.key(), r.value().clone());
        }
        Self {
            player_mob_counts,
            players_near_chunk,
        }
    }
}

impl Default for LocalMobCapCalculator {
    fn default() -> Self {
        Self {
            player_mob_counts: DashMap::new(),
            players_near_chunk: DashMap::new(),
        }
    }
}

impl fmt::Debug for LocalMobCapCalculator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LocalMobCapCalculator")
            .field("world", &"<skipped>")
            .finish()
    }
}

impl LocalMobCapCalculator {
    const fn calc_distance(chunk_pos: Vector2<i32>, player_pos: &Vector3<f64>) -> f64 {
        let dx = ((chunk_pos.x << 4) + 8) as f64 - player_pos.x;
        let dy = ((chunk_pos.y << 4) + 8) as f64 - player_pos.z;
        dx * dx + dy * dy
    }

    fn get_players_near(&self, world: &World, chunk_pos: Vector2<i32>) -> Vec<i32> {
        if let Some(players) = self.players_near_chunk.get(&chunk_pos) {
            return players.value().clone();
        }

        let mut players = Vec::new();
        for player in world.players.load().iter() {
            if player.gamemode.load() == GameMode::Spectator {
                continue;
            }
            // Vanilla ChunkMap.playerIsCloseEnoughForSpawning: < 128 blocks.
            if Self::calc_distance(chunk_pos, &player.position()) < SPAWN_DISTANCE_BLOCK_SQ {
                players.push(player.entity_id());
            }
        }
        self.players_near_chunk.insert(chunk_pos, players.clone());
        players
    }

    pub fn add_mob(&self, chunk_pos: Vector2<i32>, world: &World, category: &'static MobCategory) {
        let players = self.get_players_near(world, chunk_pos);
        for player in players {
            self.player_mob_counts
                .entry(player)
                .or_default()
                .add(category);
        }
    }

    pub fn remove_mob(
        &self,
        chunk_pos: Vector2<i32>,
        world: &World,
        category: &'static MobCategory,
    ) {
        let players = self.get_players_near(world, chunk_pos);
        for player in players {
            if let Some(count) = self.player_mob_counts.get(&player) {
                count.remove(category);
            }
        }
    }

    pub fn can_spawn(
        &self,
        category: &'static MobCategory,
        world: &World,
        chunk_pos: Vector2<i32>,
    ) -> bool {
        let players = self.get_players_near(world, chunk_pos);
        for player in players {
            if let Some(count) = self.player_mob_counts.get(&player) {
                if count.can_spawn(category) {
                    return true;
                }
            } else {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
struct PointCharge(Vector3<f64>, f64);

impl PointCharge {
    fn get_potential_change(&self, pos: &BlockPos) -> f64 {
        let dst = self.0.sub(&pos.to_f64()).length();
        self.1 / dst
    }
}

#[derive(Default, Debug)]
struct PotentialCalculator(std::sync::Mutex<Vec<PointCharge>>);

impl Clone for PotentialCalculator {
    fn clone(&self) -> Self {
        Self(std::sync::Mutex::new(self.0.lock().unwrap().clone()))
    }
}

impl PotentialCalculator {
    pub fn add_charge(&self, pos: &BlockPos, charge: f64) {
        if charge != 0. {
            self.0
                .lock()
                .unwrap()
                .push(PointCharge(pos.to_f64(), charge));
        }
    }

    pub fn remove_charge(&self, pos: &BlockPos, charge: f64) {
        if charge != 0. {
            let mut charges = self.0.lock().unwrap();
            let pos_f64 = pos.to_f64();
            if let Some(idx) = charges.iter().position(|c| c.0 == pos_f64 && c.1 == charge) {
                charges.swap_remove(idx);
            }
        }
    }
    pub fn get_potential_energy_change(&self, pos: &BlockPos, charge: f64) -> f64 {
        if charge == 0. {
            return 0.;
        }
        let mut sum: f64 = 0.;
        let charges = self.0.lock().unwrap();
        for i in charges.iter() {
            sum += i.get_potential_change(pos);
        }
        sum * charge
    }
}

use crossbeam::atomic::AtomicCell;

pub struct SpawnState {
    spawnable_chunk_count: i32,
    pub mob_category_counts: MobCounts,
    spawn_potential: PotentialCalculator,
    local_mob_cap_calculator: LocalMobCapCalculator,
    // Entities represented by the cap counters in this state. This prevents
    // removal of persistent/non-active mobs from decrementing a count they
    // never contributed to during construction.
    accounted_mobs: DashMap<Uuid, ()>,
    // unmodifiable_mob_category_counts: MobCounts, seems only for debug
    last_checked: AtomicCell<Option<(BlockPos, &'static EntityType, f64)>>,
}

impl Clone for SpawnState {
    fn clone(&self) -> Self {
        let accounted_mobs = DashMap::new();
        for entry in &self.accounted_mobs {
            accounted_mobs.insert(*entry.key(), ());
        }
        Self {
            spawnable_chunk_count: self.spawnable_chunk_count,
            mob_category_counts: self.mob_category_counts.clone(),
            spawn_potential: self.spawn_potential.clone(),
            local_mob_cap_calculator: self.local_mob_cap_calculator.clone(),
            accounted_mobs,
            last_checked: AtomicCell::new(self.last_checked.load()),
        }
    }
}

impl fmt::Debug for SpawnState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SpawnState")
            .field("spawnable_chunk_count", &self.spawnable_chunk_count)
            .field("mob_category_counts", &self.mob_category_counts)
            .field("spawn_potential", &self.spawn_potential)
            .field("local_mob_cap_calculator", &self.local_mob_cap_calculator)
            .field("accounted_mob_count", &self.accounted_mobs.len())
            .field("last_checked", &self.last_checked)
            .finish()
    }
}

impl SpawnState {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            spawnable_chunk_count: 0,
            mob_category_counts: MobCounts::default(),
            spawn_potential: PotentialCalculator::default(),
            local_mob_cap_calculator: LocalMobCapCalculator::default(),
            accounted_mobs: DashMap::new(),
            last_checked: AtomicCell::new(None),
        }
    }

    pub const fn set_spawnable_chunk_count(&mut self, count: i32) {
        self.spawnable_chunk_count = count;
    }

    pub fn add_entity(&self, world: &World, entity: &dyn EntityBase) {
        let base_entity = entity.get_entity();
        if !counts_towards_spawn_cap(base_entity, world)
            || self
                .accounted_mobs
                .insert(base_entity.entity_uuid, ())
                .is_some()
        {
            return;
        }
        let entity_type = base_entity.entity_type;
        let entity_pos = base_entity.block_pos.load();
        let biome = base_entity.current_biome.load();
        if let Some(cost) = biome.spawn_costs.get(entity_type.resource_name) {
            self.spawn_potential.add_charge(&entity_pos, cost.charge);
        }
        self.local_mob_cap_calculator.add_mob(
            base_entity.chunk_pos.load(),
            world,
            entity_type.category,
        );
        self.mob_category_counts.add(entity_type.category);
    }

    pub fn remove_entity(&self, world: &World, entity: &dyn EntityBase) {
        let base_entity = entity.get_entity();
        let entity_type = base_entity.entity_type;
        if !entity_type.mob
            || entity_type.category == &MobCategory::MISC
            || self
                .accounted_mobs
                .remove(&base_entity.entity_uuid)
                .is_none()
        {
            return;
        }
        let entity_pos = base_entity.block_pos.load();
        let biome = base_entity.current_biome.load();
        if let Some(cost) = biome.spawn_costs.get(entity_type.resource_name) {
            self.spawn_potential.remove_charge(&entity_pos, cost.charge);
        }
        self.local_mob_cap_calculator.remove_mob(
            base_entity.chunk_pos.load(),
            world,
            entity_type.category,
        );
        self.mob_category_counts.remove(entity_type.category);
    }

    pub fn new(
        chunk_count: i32,
        entities: &crate::world::entity_lookup::EntityLookup,
        world: &Arc<World>,
    ) -> Self {
        let potential = PotentialCalculator::default();
        let local_mob_cap = LocalMobCapCalculator::default();
        let counter = MobCounts::default();
        let accounted_mobs = DashMap::new();
        for entity in entities.load().iter() {
            let entity = entity.get_entity();
            if !counts_towards_spawn_cap(entity, world)
                || accounted_mobs.insert(entity.entity_uuid, ()).is_some()
            {
                continue;
            }
            let entity_type = entity.entity_type;
            let chunk_pos = entity.chunk_pos.load();
            let entity_pos = entity.block_pos.load();
            let biome = entity.current_biome.load();
            if let Some(cost) = biome.spawn_costs.get(entity_type.resource_name) {
                potential.add_charge(&entity_pos, cost.charge);
            }
            if entity_type.mob {
                local_mob_cap.add_mob(chunk_pos, world, entity_type.category);
            }
            counter.add(entity_type.category);
        }
        Self {
            spawnable_chunk_count: chunk_count,
            mob_category_counts: counter,
            spawn_potential: potential,
            local_mob_cap_calculator: local_mob_cap,
            accounted_mobs,
            last_checked: AtomicCell::new(None),
        }
    }
    #[inline]
    pub fn can_spawn_for_category_global(&self, category: &'static MobCategory) -> bool {
        // Vanilla SpawnState.canSpawnForCategoryGlobal:
        // maxInstancesPerChunk * spawnableChunkCount / 289  (integer floor).
        let limit = category.max * self.spawnable_chunk_count / MAGIC_NUMBER;
        self.mob_category_counts.0[category.id].load(Relaxed) < limit
    }

    #[must_use]
    pub fn spawnable_chunk_count(&self) -> i32 {
        self.spawnable_chunk_count
    }

    #[must_use]
    pub fn category_count(&self, category: &'static MobCategory) -> i32 {
        self.mob_category_counts.0[category.id].load(Relaxed)
    }
    pub fn can_spawn_for_category_local(
        &self,
        world: &Arc<World>,
        category: &'static MobCategory,
        chunk_pos: Vector2<i32>,
    ) -> bool {
        self.local_mob_cap_calculator
            .can_spawn(category, world, chunk_pos)
    }
    pub fn can_spawn(
        &self,
        entity_type: &'static EntityType,
        pos: &BlockPos,
        world: &Arc<World>,
    ) -> bool {
        // TODO get biome
        let biome = world.level.get_rough_biome(pos);
        biome
            .spawn_costs
            .get(entity_type.resource_name)
            .map_or_else(
                || {
                    self.last_checked.store(Some((*pos, entity_type, 0.)));
                    true
                },
                |cost| {
                    self.last_checked
                        .store(Some((*pos, entity_type, cost.charge)));
                    self.spawn_potential
                        .get_potential_energy_change(pos, cost.charge)
                        <= cost.energy_budget
                },
            )
    }
    pub fn after_spawn(&self, entity: &dyn EntityBase, world: &Arc<World>) {
        let base_entity = entity.get_entity();
        if !counts_towards_spawn_cap(base_entity, world)
            || self
                .accounted_mobs
                .insert(base_entity.entity_uuid, ())
                .is_some()
        {
            return;
        }

        let entity_type = base_entity.entity_type;
        let pos = base_entity.block_pos.load();
        let charge = if let Some((l_pos, l_type, l_charge)) = self.last_checked.load()
            && l_pos.eq(&pos)
            && l_type == entity_type
        {
            Some(l_charge)
        } else {
            None
        };

        let charge = charge.unwrap_or_else(|| {
            // TODO get biome
            let biome = world.level.get_rough_biome(&pos);
            biome
                .spawn_costs
                .get(entity_type.resource_name)
                .map_or(0., |cost| cost.charge)
        });

        self.spawn_potential.add_charge(&pos, charge);
        self.mob_category_counts.add(entity_type.category);
        self.local_mob_cap_calculator.add_mob(
            Vector2::<i32>::new(get_section_cord(pos.0.x), get_section_cord(pos.0.z)),
            world,
            entity_type.category,
        );
    }
}

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

#[must_use]
pub fn get_nearest_player(pos: &Vector3<f64>, player_positions: &[Vector3<f64>]) -> f64 {
    let mut min_dst_sq = f64::MAX;

    for player_pos in player_positions {
        let cur_dst_sq = player_pos.squared_distance_to_vec(pos);
        if cur_dst_sq < min_dst_sq {
            min_dst_sq = cur_dst_sq;
        }
    }
    min_dst_sq
}

#[must_use]
pub fn is_right_distance_to_player_and_spawn_point(
    world: &World,
    pos: &BlockPos,
    distance: f64,
    chunk_pos: &Vector2<i32>,
    world_spawn: Vector3<f64>,
) -> bool {
    // Vanilla: must be > 24 blocks from any player.
    if distance <= 24. * 24. {
        return false;
    }
    // Vanilla: must be > 24 blocks from world spawn (not world origin).
    if pos
        .to_centered_f64()
        .squared_distance_to(world_spawn.x, world_spawn.y, world_spawn.z)
        <= 24. * 24.
    {
        return false;
    }
    let target_chunk = Vector2::new(get_section_cord(pos.0.x), get_section_cord(pos.0.z));
    can_spawn_entities_in_chunk(*chunk_pos, target_chunk, &world.active_chunks.load())
}

/// Vanilla permits a pack to spill from its origin chunk into another entity-ticking chunk.
#[must_use]
fn can_spawn_entities_in_chunk(
    origin_chunk: Vector2<i32>,
    target_chunk: Vector2<i32>,
    active_chunks: &FxHashSet<Vector2<i32>>,
) -> bool {
    origin_chunk == target_chunk || active_chunks.contains(&target_chunk)
}

#[must_use]
pub fn get_random_spawn_mob_at(
    world: &Arc<World>,
    category: &'static MobCategory,
    block_pos: &BlockPos,
) -> Option<&'static Spawner> {
    let mut random = rng();
    get_random_spawn_mob_at_with_random(world, category, block_pos, &mut random)
}

fn get_random_spawn_mob_at_with_random<R: Rng + ?Sized>(
    world: &Arc<World>,
    category: &'static MobCategory,
    block_pos: &BlockPos,
    random: &mut R,
) -> Option<&'static Spawner> {
    // TODO Holder<Biome> holder = level.getBiome(pos);
    let biome = world.level.get_rough_biome(block_pos);
    if category == &MobCategory::WATER_AMBIENT
        && biome.has_tag(&MINECRAFT_REDUCE_WATER_AMBIENT_SPAWNS)
        && random.random::<f32>() < 0.98f32
    {
        return None;
    }

    // TODO isInNetherFortressBounds(pos, level, cetagory, structureManager) then NetherFortressStructure.FORTRESS_ENEMIES
    // TODO structureManager.getAllStructuresAt(pos); ChunkGenerator::getMobsAt
    choose_weighted_spawner(spawn_pool_for_category(biome, category), random)
}

fn spawn_pool_for_category(biome: &Biome, category: &'static MobCategory) -> &'static [Spawner] {
    match category.id {
        id if id == MobCategory::MONSTER.id => biome.spawners.monster,
        id if id == MobCategory::CREATURE.id => biome.spawners.creature,
        id if id == MobCategory::AMBIENT.id => biome.spawners.ambient,
        id if id == MobCategory::AXOLOTLS.id => biome.spawners.axolotls,
        id if id == MobCategory::UNDERGROUND_WATER_CREATURE.id => {
            biome.spawners.underground_water_creature
        }
        id if id == MobCategory::WATER_CREATURE.id => biome.spawners.water_creature,
        id if id == MobCategory::WATER_AMBIENT.id => biome.spawners.water_ambient,
        id if id == MobCategory::MISC.id => biome.spawners.misc,
        _ => panic!(),
    }
}

fn spawner_is_in_biome_pool(
    biome: &Biome,
    category: &'static MobCategory,
    spawner: &Spawner,
) -> bool {
    spawn_pool_for_category(biome, category)
        .iter()
        .any(|candidate| {
            candidate.r#type == spawner.r#type
                && candidate.min_count == spawner.min_count
                && candidate.max_count == spawner.max_count
        })
}

fn total_spawner_weight(spawners: &[Spawner]) -> Option<u64> {
    let total = spawners.iter().try_fold(0_u64, |total, spawner| {
        total.checked_add(u64::from(spawner.weight))
    })?;
    (total != 0).then_some(total)
}

#[must_use]
fn select_weighted_spawner(spawners: &[Spawner], mut roll: u64) -> Option<&Spawner> {
    if roll >= total_spawner_weight(spawners)? {
        return None;
    }

    for spawner in spawners {
        let weight = u64::from(spawner.weight);
        if roll < weight {
            return Some(spawner);
        }
        roll -= weight;
    }

    None
}

fn choose_weighted_spawner<'a, R: Rng + ?Sized>(
    spawners: &'a [Spawner],
    random: &mut R,
) -> Option<&'a Spawner> {
    let total = total_spawner_weight(spawners)?;
    select_weighted_spawner(spawners, random.random_range(0..total))
}

fn choose_weighted_spawner_with_random_impl<'a, R: RandomImpl>(
    spawners: &'a [Spawner],
    random: &mut R,
) -> Option<&'a Spawner> {
    let total = i32::try_from(total_spawner_weight(spawners)?).ok()?;
    select_weighted_spawner(spawners, random.next_bounded_i32(total) as u64)
}

pub fn is_valid_spawn_position_for_type(
    world: &Arc<World>,
    block_pos: &BlockPos,
    category: &'static MobCategory,
    entity_type: &'static EntityType,
    distance: f64,
    is_thundering: bool,
) -> bool {
    // SpawnPlacements.checkSpawnRules → is_spawn_position_ok + check_spawn_rules below
    // (vanilla EntitySpawnReason.NATURAL light / biome rules).
    if category == &MobCategory::MISC {
        return false;
    }
    if !entity_type.can_spawn_far_from_player
        && distance
            > f64::from(entity_type.category.despawn_distance)
                * f64::from(entity_type.category.despawn_distance)
    {
        return false;
    }
    if !entity_type.summonable {
        return false;
    }
    if !is_spawn_position_ok(world, block_pos, entity_type) {
        return false;
    }
    if !check_spawn_rules(entity_type, world, block_pos, is_thundering) {
        return false;
    }
    // TODO: we should use getSpawnBox, but this is only modified for slimes and magma slimes
    if !world.is_space_empty(BoundingBox::new_from_pos(
        f64::from(block_pos.0.x) + 0.5,
        f64::from(block_pos.0.y),
        f64::from(block_pos.0.z) + 0.5,
        &EntityDimensions {
            width: entity_type.dimension[0],
            height: entity_type.dimension[1],
            eye_height: entity_type.eye_height,
        },
    )) {
        return false;
    }
    true
}

pub fn is_spawn_position_ok(
    world: &Arc<World>,
    block_pos: &BlockPos,
    entity_type: &'static EntityType,
) -> bool {
    match entity_type.spawn_restriction.location {
        SpawnLocation::InLava => world.get_fluid(block_pos).has_tag(&MINECRAFT_LAVA),
        SpawnLocation::InWater => {
            let above_state = world.get_block_state(&block_pos.up());
            world.get_fluid(block_pos).has_tag(&MINECRAFT_WATER)
                && !is_redstone_conductor(above_state)
        }
        SpawnLocation::OnGround => {
            let down = world.get_block_state(&block_pos.down());
            let up = world.get_block_state(&block_pos.up());
            let cur = world.get_block_state(block_pos);
            // TODO: blockState.allowsSpawning
            let is_valid_spawn_below =
                down.is_side_solid(BlockDirection::Up) && down.luminance < 14;

            if is_valid_spawn_below {
                is_valid_empty_spawn_block(cur, entity_type)
                    && is_valid_empty_spawn_block(up, entity_type)
            } else {
                false
            }
        }
        SpawnLocation::Unrestricted => true,
    }
}

/// Cache-based version of `is_spawn_position_ok` used during world generation.
pub fn is_spawn_position_ok_cache(
    cache: &dyn GenerationCache,
    block_pos: &BlockPos,
    entity_type: &'static EntityType,
) -> bool {
    let pos_vec = block_pos.0;
    let state = GenerationCache::get_block_state(cache, &pos_vec).to_state();

    match entity_type.spawn_restriction.location {
        SpawnLocation::InLava => {
            // During generation, we check the block state's liquid property and tag
            state.is_liquid() && Block::from_state_id(state.id).has_tag(&MINECRAFT_LAVA)
        }
        SpawnLocation::InWater => {
            let above_pos = block_pos.up().0;
            let above_state = GenerationCache::get_block_state(cache, &above_pos).to_state();

            state.is_liquid()
                && Block::from_state_id(state.id).has_tag(&MINECRAFT_WATER)
                && !is_redstone_conductor(above_state)
        }
        SpawnLocation::OnGround => {
            let down_pos = block_pos.down().0;
            let up_pos = block_pos.up().0;

            let down = GenerationCache::get_block_state(cache, &down_pos).to_state();
            let up = GenerationCache::get_block_state(cache, &up_pos).to_state();

            // Logic: solid surface below and low enough light level (if applicable in generation)
            let is_valid_spawn_below =
                down.is_side_solid(BlockDirection::Up) && down.luminance < 14;

            if is_valid_spawn_below {
                is_valid_empty_spawn_block(state, entity_type)
                    && is_valid_empty_spawn_block(up, entity_type)
            } else {
                false
            }
        }
        SpawnLocation::Unrestricted => true,
    }
}

/// Cache-based version of `adjust_spawn_position` used during world generation.
pub fn adjust_spawn_position_cache(
    cache: &dyn GenerationCache,
    pos: BlockPos,
    entity_type: &'static EntityType,
) -> BlockPos {
    if matches!(
        entity_type.spawn_restriction.location,
        SpawnLocation::OnGround
    ) {
        let below = pos.down();
        let state = GenerationCache::get_block_state(cache, &below.0).to_state();

        if !state.is_full_cube() && !state.is_liquid() {
            return below;
        }
    }
    pos
}

pub fn adjust_spawn_position(
    world: &World,
    pos: BlockPos,
    entity_type: &'static EntityType,
) -> BlockPos {
    if matches!(
        entity_type.spawn_restriction.location,
        SpawnLocation::OnGround
    ) {
        let below = pos.down();
        let state = world.get_block_state(&below);
        // Approximation of isPathfindable(LAND)
        if !state.is_full_cube() && !state.is_liquid() {
            return below;
        }
    }
    pos
}

/// Vanilla `BlockState.isSignalSource` for the blocks whose non-full collision
/// shapes would otherwise let a mob spawn inside them.
#[must_use]
fn is_signal_source(state: &BlockState) -> bool {
    let block = Block::from_state_id(state.id);

    block.has_tag(&MINECRAFT_BUTTONS)
        || block.has_tag(&MINECRAFT_PRESSURE_PLATES)
        || matches!(
            block.id,
            BlockId::REPEATER
                | BlockId::COMPARATOR
                | BlockId::DAYLIGHT_DETECTOR
                | BlockId::LEVER
                | BlockId::SCULK_SENSOR
                | BlockId::CALIBRATED_SCULK_SENSOR
                | BlockId::LIGHTNING_ROD
                | BlockId::LECTERN
                | BlockId::REDSTONE_BLOCK
                | BlockId::JUKEBOX
                | BlockId::REDSTONE_TORCH
                | BlockId::REDSTONE_WALL_TORCH
                | BlockId::TRAPPED_CHEST
                | BlockId::OBSERVER
                | BlockId::TARGET
                | BlockId::REDSTONE_WIRE
                | BlockId::TRIPWIRE_HOOK
                | BlockId::DETECTOR_RAIL
        )
}

/// Vanilla `EntityType.isBlockDangerous` immunity tags.
#[must_use]
fn is_immune_to_block(entity_type: &EntityType, block: &Block) -> bool {
    let id = entity_type.id;
    (id == EntityType::FOX.id && block.has_tag(&MINECRAFT_FOX_IMMUNE_TO))
        || (id == EntityType::POLAR_BEAR.id && block.has_tag(&MINECRAFT_POLAR_BEAR_IMMUNE_TO))
        || (id == EntityType::SNOW_GOLEM.id && block.has_tag(&MINECRAFT_SNOW_GOLEM_IMMUNE_TO))
        || (id == EntityType::STRAY.id && block.has_tag(&MINECRAFT_STRAY_IMMUNE_TO))
        || (id == EntityType::WITHER.id && block.has_tag(&MINECRAFT_WITHER_IMMUNE_TO))
        || (id == EntityType::WITHER_SKELETON.id
            && block.has_tag(&MINECRAFT_WITHER_SKELETON_IMMUNE_TO))
}

/// Vanilla `NodeEvaluator.isBurningBlock`.
#[must_use]
fn is_burning_block(state: &BlockState, block: &Block) -> bool {
    block.has_tag(&MINECRAFT_FIRE)
        || matches!(
            block.id,
            BlockId::LAVA | BlockId::MAGMA_BLOCK | BlockId::LAVA_CAULDRON
        )
        || (matches!(block.id, BlockId::CAMPFIRE | BlockId::SOUL_CAMPFIRE) && state.luminance > 0)
}

/// Vanilla `EntityType.isBlockDangerous`.
#[must_use]
fn is_block_dangerous(state: &BlockState, entity_type: &EntityType) -> bool {
    let block = Block::from_state_id(state.id);
    if is_immune_to_block(entity_type, block) {
        return false;
    }

    if !entity_type.fire_immune && is_burning_block(state, block) {
        return true;
    }

    matches!(
        block.id,
        BlockId::WITHER_ROSE | BlockId::SWEET_BERRY_BUSH | BlockId::CACTUS | BlockId::POWDER_SNOW
    )
}

/// Vanilla `NaturalSpawner.isValidEmptySpawnBlock`.
#[must_use]
pub fn is_valid_empty_spawn_block(state: &BlockState, entity_type: &EntityType) -> bool {
    if state.is_full_cube() {
        return false;
    }
    if is_signal_source(state) {
        return false;
    }
    if state.is_liquid() {
        return false;
    }
    if Block::from_state_id(state.id).has_tag(&MINECRAFT_PREVENT_MOB_SPAWNING_INSIDE) {
        return false;
    }

    !is_block_dangerous(state, entity_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redstone_conductors_are_not_limited_to_full_cubes() {
        let soul_sand = Block::SOUL_SAND.default_state;
        assert!(is_redstone_conductor(soul_sand));
        assert!(!soul_sand.is_full_cube());
        assert!(!is_redstone_conductor(Block::REDSTONE_BLOCK.default_state));
    }

    #[test]
    fn empty_spawn_blocks_reject_signal_sources() {
        let lever = Block::LEVER.default_state;
        assert!(!lever.is_full_cube());
        assert!(is_signal_source(lever));
        assert!(!is_valid_empty_spawn_block(lever, &EntityType::ZOMBIE));
    }

    #[test]
    fn dangerous_spawn_blocks_honor_entity_immunities() {
        assert!(is_block_dangerous(
            Block::FIRE.default_state,
            &EntityType::ZOMBIE,
        ));
        assert!(!is_block_dangerous(
            Block::FIRE.default_state,
            &EntityType::BLAZE,
        ));
        assert!(is_block_dangerous(
            Block::SWEET_BERRY_BUSH.default_state,
            &EntityType::ZOMBIE,
        ));
        assert!(!is_block_dangerous(
            Block::SWEET_BERRY_BUSH.default_state,
            &EntityType::FOX,
        ));
        assert!(is_block_dangerous(
            Block::POWDER_SNOW.default_state,
            &EntityType::ZOMBIE,
        ));
        assert!(!is_block_dangerous(
            Block::POWDER_SNOW.default_state,
            &EntityType::POLAR_BEAR,
        ));
    }

    #[test]
    fn packs_can_cross_into_an_active_neighboring_chunk() {
        let origin = Vector2::new(0, 0);
        let neighbor = Vector2::new(1, 0);
        let mut active_chunks = FxHashSet::default();
        active_chunks.insert(neighbor);

        assert!(can_spawn_entities_in_chunk(origin, origin, &active_chunks,));
        assert!(can_spawn_entities_in_chunk(
            origin,
            neighbor,
            &active_chunks,
        ));
        assert!(!can_spawn_entities_in_chunk(
            origin,
            Vector2::new(2, 0),
            &active_chunks,
        ));
    }

    #[test]
    fn generated_biome_spawner_weights_are_preserved() {
        let zombie = Biome::BADLANDS
            .spawners
            .monster
            .iter()
            .find(|spawner| spawner.r#type == "minecraft:zombie")
            .expect("badlands has a zombie spawner");
        assert_eq!(zombie.weight, 95);
    }

    #[test]
    fn pack_spawner_must_exist_in_the_destination_biome_pool() {
        let zombie = Biome::BADLANDS
            .spawners
            .monster
            .iter()
            .find(|spawner| spawner.r#type == "minecraft:zombie")
            .expect("badlands has a zombie spawner");

        assert!(spawner_is_in_biome_pool(
            &Biome::BADLANDS,
            &MobCategory::MONSTER,
            zombie,
        ));
        assert!(!spawner_is_in_biome_pool(
            &Biome::WARPED_FOREST,
            &MobCategory::MONSTER,
            zombie,
        ));
    }

    #[test]
    fn selects_biome_spawners_by_weight_boundaries() {
        let spawners = [
            Spawner {
                r#type: "minecraft:first",
                min_count: 1,
                max_count: 1,
                weight: 2,
            },
            Spawner {
                r#type: "minecraft:second",
                min_count: 1,
                max_count: 1,
                weight: 3,
            },
            Spawner {
                r#type: "minecraft:third",
                min_count: 1,
                max_count: 1,
                weight: 1,
            },
        ];

        assert_eq!(
            select_weighted_spawner(&spawners, 0).map(|spawner| spawner.r#type),
            Some("minecraft:first")
        );
        assert_eq!(
            select_weighted_spawner(&spawners, 1).map(|spawner| spawner.r#type),
            Some("minecraft:first")
        );
        assert_eq!(
            select_weighted_spawner(&spawners, 2).map(|spawner| spawner.r#type),
            Some("minecraft:second")
        );
        assert_eq!(
            select_weighted_spawner(&spawners, 4).map(|spawner| spawner.r#type),
            Some("minecraft:second")
        );
        assert_eq!(
            select_weighted_spawner(&spawners, 5).map(|spawner| spawner.r#type),
            Some("minecraft:third")
        );
        assert!(select_weighted_spawner(&spawners, 6).is_none());
    }

    #[test]
    fn ignores_zero_weight_biome_spawners() {
        let spawners = [
            Spawner {
                r#type: "minecraft:ignored",
                min_count: 1,
                max_count: 1,
                weight: 0,
            },
            Spawner {
                r#type: "minecraft:selected",
                min_count: 1,
                max_count: 1,
                weight: 1,
            },
        ];

        assert_eq!(
            select_weighted_spawner(&spawners, 0).map(|spawner| spawner.r#type),
            Some("minecraft:selected")
        );
    }
}
