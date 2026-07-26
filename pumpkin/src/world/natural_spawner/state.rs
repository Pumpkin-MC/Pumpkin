use crate::entity::{Entity, EntityBase};
use crate::world::World;
use crossbeam::atomic::AtomicCell;
use dashmap::DashMap;
use pumpkin_data::entity::{EntityType, MobCategory};
use pumpkin_util::GameMode;
use pumpkin_util::math::get_section_cord;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering::Relaxed};
use uuid::Uuid;

use super::SPAWN_DISTANCE_BLOCK_SQ;

const MAGIC_NUMBER: i32 = 17 * 17;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::natural_spawner as public_api;

    // Compile-time assertions that the public paths and signatures survived the
    // module split (re-exported through `crate::world::natural_spawner`).
    const _: fn() -> public_api::SpawnState = public_api::SpawnState::empty;
    const _: fn(&public_api::SpawnState, &World, &dyn EntityBase) =
        public_api::SpawnState::add_entity;
    const _: fn(&public_api::SpawnState, &World, &dyn EntityBase) =
        public_api::SpawnState::remove_entity;
    const _: fn(&public_api::SpawnState, &dyn EntityBase, &Arc<World>) =
        public_api::SpawnState::after_spawn;
    const _: fn(&public_api::SpawnState, &'static MobCategory) -> bool =
        public_api::SpawnState::can_spawn_for_category_global;
    const _: fn(&public_api::SpawnState) -> i32 = public_api::SpawnState::spawnable_chunk_count;
    const _: fn(&public_api::SpawnState, &'static MobCategory) -> i32 =
        public_api::SpawnState::category_count;
    const _: fn(&public_api::MobCounts, &'static MobCategory) -> bool =
        public_api::MobCounts::can_spawn;
    const _: fn(
        &public_api::LocalMobCapCalculator,
        &'static MobCategory,
        &World,
        Vector2<i32>,
    ) -> bool = public_api::LocalMobCapCalculator::can_spawn;

    #[test]
    fn mob_counts_enforce_the_per_category_cap() {
        let counts = MobCounts::default();
        let category = &MobCategory::CREATURE;
        for _ in 0..category.max {
            assert!(counts.can_spawn(category));
            counts.add(category);
        }
        assert!(!counts.can_spawn(category));
        counts.remove(category);
        assert!(counts.can_spawn(category));
    }

    #[test]
    fn global_cap_scales_with_spawnable_chunk_count() {
        let mut state = SpawnState::empty();
        // With zero spawnable chunks every derived category limit is zero.
        assert!(!state.can_spawn_for_category_global(&MobCategory::MONSTER));

        // 289 spawnable chunks (17 * 17) makes the limit exactly `category.max`.
        state.set_spawnable_chunk_count(MAGIC_NUMBER);
        let category = &MobCategory::MONSTER;
        for _ in 0..category.max {
            assert!(state.can_spawn_for_category_global(category));
            state.mob_category_counts.add(category);
        }
        assert!(!state.can_spawn_for_category_global(category));
        assert_eq!(state.category_count(category), category.max);
        assert_eq!(state.spawnable_chunk_count(), MAGIC_NUMBER);
    }

    #[test]
    fn potential_energy_change_follows_inverse_distance() {
        let calculator = PotentialCalculator::default();
        let origin = BlockPos::new(0, 0, 0);
        calculator.add_charge(&origin, 2.0);

        // Probe 4 blocks away: energy change = (2.0 / 4.0) * probe charge.
        let probe = BlockPos::new(0, 4, 0);
        let change = calculator.get_potential_energy_change(&probe, 1.0);
        assert!((change - 0.5).abs() < 1e-9);

        // A zero probe charge never contributes energy.
        assert!(calculator.get_potential_energy_change(&probe, 0.0).abs() < f64::EPSILON);

        calculator.remove_charge(&origin, 2.0);
        assert!(calculator.get_potential_energy_change(&probe, 1.0).abs() < f64::EPSILON);
    }
}
