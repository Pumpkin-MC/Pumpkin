use crate::entity::EntityBase;
use crate::entity::r#type::{check_spawn_rules, from_type};
use crate::world::World;
use arc_swap::ArcSwap;
use pumpkin_data::biome::Spawner;
use pumpkin_data::chunk::Biome;
use pumpkin_data::entity::{EntityType, MobCategory, SpawnLocation};
use pumpkin_data::tag::Block::MINECRAFT_PREVENT_MOB_SPAWNING_INSIDE;
use pumpkin_data::tag::Fluid::{MINECRAFT_LAVA, MINECRAFT_WATER};
use pumpkin_data::tag::Taggable;
use pumpkin_data::tag::WorldgenBiome::MINECRAFT_REDUCE_WATER_AMBIENT_SPAWNS;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_nbt::pnbt::PNbtCompound;
use pumpkin_util::GameMode;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::get_section_cord;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomImpl, get_seed};
use pumpkin_world::chunk::io::Dirtiable;
use pumpkin_world::chunk::{ChunkData, ChunkHeightmapType};
use pumpkin_world::generation::proto_chunk::GenerationCache;
use rand::seq::IndexedRandom;
use rand::{RngExt, rng};
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

const MAGIC_NUMBER: i32 = 17 * 17;

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
            if Self::calc_distance(chunk_pos, &player.position()) < 16384. {
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
    // unmodifiable_mob_category_counts: MobCounts, seems only for debug
    last_checked: AtomicCell<Option<(BlockPos, &'static EntityType, f64)>>,
}

impl Clone for SpawnState {
    fn clone(&self) -> Self {
        Self {
            spawnable_chunk_count: self.spawnable_chunk_count,
            mob_category_counts: self.mob_category_counts.clone(),
            spawn_potential: self.spawn_potential.clone(),
            local_mob_cap_calculator: self.local_mob_cap_calculator.clone(),
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
            last_checked: AtomicCell::new(None),
        }
    }

    pub const fn set_spawnable_chunk_count(&mut self, count: i32) {
        self.spawnable_chunk_count = count;
    }

    pub fn add_entity(&self, world: &World, entity: &dyn EntityBase) {
        let base_entity = entity.get_entity();
        let entity_type = base_entity.entity_type;
        if !entity_type.mob || entity_type.category == &MobCategory::MISC {
            return;
        }
        let entity_pos = base_entity.block_pos.load();
        let biome = base_entity.current_biome.load();
        if let Some(cost) = biome.spawn_costs.get(entity_type.resource_name) {
            self.spawn_potential.add_charge(&entity_pos, cost.charge);
        }
        if entity_type.mob {
            self.local_mob_cap_calculator.add_mob(
                base_entity.chunk_pos.load(),
                world,
                entity_type.category,
            );
            self.mob_category_counts.add(entity_type.category);
        }
    }

    pub fn remove_entity(&self, world: &World, entity: &dyn EntityBase) {
        let base_entity = entity.get_entity();
        let entity_type = base_entity.entity_type;
        if !entity_type.mob || entity_type.category == &MobCategory::MISC {
            return;
        }
        let entity_pos = base_entity.block_pos.load();
        let biome = base_entity.current_biome.load();
        if let Some(cost) = biome.spawn_costs.get(entity_type.resource_name) {
            self.spawn_potential.remove_charge(&entity_pos, cost.charge);
        }
        if entity_type.mob {
            self.local_mob_cap_calculator.remove_mob(
                base_entity.chunk_pos.load(),
                world,
                entity_type.category,
            );
            self.mob_category_counts.remove(entity_type.category);
        }
    }

    pub fn new(
        chunk_count: i32,
        entities: &ArcSwap<Vec<Arc<dyn EntityBase>>>,
        world: &Arc<World>,
    ) -> Self {
        let potential = PotentialCalculator::default();
        let local_mob_cap = LocalMobCapCalculator::default();
        let counter = MobCounts::default();
        let active_chunks = world.active_chunks.load();
        for entity in entities.load().iter() {
            let entity = entity.get_entity();
            let entity_type = entity.entity_type;
            if !entity_type.mob || entity_type.category == &MobCategory::MISC {
                // TODO (mob.isPersistenceRequired() || mob.requiresCustomPersistence())
                continue;
            }
            let chunk_pos = entity.chunk_pos.load();
            if !active_chunks.contains(&chunk_pos) {
                continue;
            }
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
            last_checked: AtomicCell::new(None),
        }
    }
    #[inline]
    pub fn can_spawn_for_category_global(&self, category: &'static MobCategory) -> bool {
        self.mob_category_counts.0[category.id].load(Relaxed)
            < category.max * self.spawnable_chunk_count / MAGIC_NUMBER
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
    pub async fn can_spawn(
        &self,
        entity_type: &'static EntityType,
        pos: &BlockPos,
        world: &Arc<World>,
    ) -> bool {
        // TODO get biome
        let biome = world.level.get_rough_biome(pos).await;
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
    pub async fn after_spawn(
        &self,
        entity_type: &'static EntityType,
        pos: &BlockPos,
        world: &Arc<World>,
    ) {
        let charge = if let Some((l_pos, l_type, l_charge)) = self.last_checked.load()
            && l_pos.eq(pos)
            && l_type == entity_type
        {
            Some(l_charge)
        } else {
            None
        };

        let charge = if let Some(charge) = charge {
            charge
        } else {
            // TODO get biome
            let biome = world.level.get_rough_biome(pos).await;
            biome
                .spawn_costs
                .get(entity_type.resource_name)
                .map_or(0., |cost| cost.charge)
        };

        self.spawn_potential.add_charge(pos, charge);
        self.mob_category_counts.add(entity_type.category);
        self.local_mob_cap_calculator.add_mob(
            Vector2::<i32>::new(get_section_cord(pos.0.x), get_section_cord(pos.0.z)),
            world,
            entity_type.category,
        );
    }
}

#[must_use]
pub fn get_filtered_spawning_categories(
    state: &SpawnState,
    spawn_friendlies: bool,
    spawn_enemies: bool,
    spawn_passives: bool,
) -> Vec<&'static MobCategory> {
    let mut ret = Vec::with_capacity(MobCategory::SPAWNING_CATEGORIES.len());
    for category in MobCategory::SPAWNING_CATEGORIES {
        let is_type_allowed = if category.is_friendly {
            spawn_friendlies
        } else {
            spawn_enemies
        };

        if !is_type_allowed {
            continue;
        }

        if category.is_persistent && !spawn_passives {
            continue;
        }

        if state.can_spawn_for_category_global(category) {
            ret.push(category);
        }
    }
    ret
}

pub async fn spawn_for_chunk(
    world: &Arc<World>,
    chunk_pos: Vector2<i32>,
    chunk: &Arc<ChunkData>,
    spawn_state: &SpawnState,
    spawn_list: &Vec<&'static MobCategory>,
) {
    // debug!("spawn for chunk {:?}", chunk_pos);
    for category in spawn_list {
        if spawn_state.can_spawn_for_category_local(world, category, chunk_pos) {
            let random_pos = get_random_pos_within(world.min_y, &chunk_pos, chunk);
            if random_pos.0.y > world.min_y {
                spawn_category_for_position(category, world, random_pos, &chunk_pos, spawn_state)
                    .await;
            }
        }
    }
}
pub fn get_random_pos_within(
    min_y: i32,
    chunk_pos: &Vector2<i32>,
    chunk: &Arc<ChunkData>,
) -> BlockPos {
    let mut rng = Xoroshiro::from_seed(get_seed());

    let x = (chunk_pos.x << 4) + rng.next_bounded_i32(16);
    let z = (chunk_pos.y << 4) + rng.next_bounded_i32(16);
    let temp_y = chunk.heightmap.lock().unwrap().get(
        ChunkHeightmapType::WorldSurface,
        x,
        z,
        chunk.section.min_y,
    ) + 1;
    let y = rng.next_inbetween_i32(min_y, temp_y);
    BlockPos::new(x, y, z)
}

pub async fn spawn_mobs_for_chunk_generation(
    world: &Arc<World>,
    cache: &mut dyn GenerationCache,
    biome: &'static Biome,
    chunk_x: i32,
    chunk_z: i32,
) {
    let mob_settings = &biome.spawners;
    let creatures = &mob_settings.creature;

    if creatures.is_empty() {
        return;
    }

    let xo = chunk_x << 4;
    let zo = chunk_z << 4;

    while rand::random::<f32>() < biome.creature_spawn_probability {
        let Some(spawner_data) = creatures.choose(&mut rand::rng()) else {
            continue;
        };

        let count = spawner_data.min_count
            + rand::random_range(0..(1 + spawner_data.max_count - spawner_data.min_count).max(1));
        let entity_type = EntityType::from_name(
            spawner_data
                .r#type
                .strip_prefix("minecraft:")
                .unwrap_or(spawner_data.r#type),
        )
        .unwrap();

        let mut x = xo + rand::random_range(0..16);
        let mut z = zo + rand::random_range(0..16);
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

                    let entity = from_type(entity_type, spawn_pos_f64, world, Uuid::new_v4()).await;
                    entity
                        .get_entity()
                        .set_rotation(rand::random::<f32>() * 360., 0.);
                    world.spawn_entity(entity).await;
                    success = true;
                }

                // Random jitter for the next mob in the group
                x += rand::random_range(0..5) - rand::random_range(0..5);
                z += rand::random_range(0..5) - rand::random_range(0..5);

                // Keep group within the chunk bounds
                if x < xo || x >= xo + 16 || z < zo || z >= zo + 16 {
                    x = start_x;
                    z = start_z;
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

pub async fn spawn_category_for_position(
    category: &'static MobCategory,
    world: &Arc<World>,
    pos: BlockPos,
    chunk_pos: &Vector2<i32>,
    spawn_state: &SpawnState,
) {
    let mut batch_buffer = vec![];
    let mut spawn_cluster_size = 0;
    let player_positions: Vec<_> = world.players.load().iter().map(|p| p.position()).collect();

    'group_loop: for _ in 0..3 {
        let mut new_x = pos.0.x;
        let mut new_z = pos.0.z;

        let mut random_group_size = (rng().random::<f32>() * 4.).ceil() as i32;
        let mut inc = 0;
        let mut current_spawner = None;

        'spawn_loop: while inc < random_group_size {
            new_x += rng().random_range(0..6) - rng().random_range(0..6);
            new_z += rng().random_range(0..6) - rng().random_range(0..6);
            let mut new_pos = BlockPos::new(new_x, pos.0.y, new_z);

            if current_spawner.is_none() {
                let Some(spawner) = get_random_spawn_mob_at(world, category, &new_pos).await else {
                    break 'spawn_loop;
                };
                current_spawner = Some(spawner);
                random_group_size = rng().random_range(spawner.min_count..=spawner.max_count);
            }

            let spawner = current_spawner.unwrap();
            let entity_type =
                &EntityType::from_name(spawner.r#type.strip_prefix("minecraft:").unwrap()).unwrap();

            new_pos = adjust_spawn_position(world, new_pos, entity_type).await;

            let spawn_pos_f64 = Vector3::new(
                f64::from(new_pos.0.x) + 0.5,
                f64::from(new_pos.0.y),
                f64::from(new_pos.0.z) + 0.5,
            );

            let player_distance = get_nearest_player(&spawn_pos_f64, &player_positions);
            if !is_right_distance_to_player_and_spawn_point(&new_pos, player_distance, chunk_pos) {
                inc += 1;
                continue;
            }

            if !is_valid_spawn_position_for_type(
                world,
                &new_pos,
                category,
                entity_type,
                player_distance,
            )
            .await
            {
                inc += 1;
                continue;
            }
            if !spawn_state.can_spawn(entity_type, &new_pos, world).await {
                inc += 1;
                continue;
            }

            let entity = from_type(entity_type, spawn_pos_f64, world, Uuid::new_v4()).await;
            entity
                .get_entity()
                .set_rotation(rng().random::<f32>() * 360., 0.);

            spawn_cluster_size += 1;
            batch_buffer.push(entity);
            spawn_state.after_spawn(entity_type, &new_pos, world).await;
            if spawn_cluster_size >= entity_type.limit_per_chunk {
                break 'group_loop;
            }

            inc += 1;
        }
    }

    // Spawn in batch
    if !batch_buffer.is_empty() {
        let mut prepared_data = Vec::with_capacity(batch_buffer.len());

        for entity in &batch_buffer {
            entity.init_data_tracker().await;
            let base_entity = entity.get_entity();
            let packet = base_entity.create_spawn_packet();
            let mut nbt = PNbtCompound::new();
            entity.write_nbt(&mut nbt).await;
            // Keep the entity reference here so we don't have to "find" it later
            prepared_data.push((base_entity.entity_uuid, nbt, packet, entity.clone()));
        }

        let chunk_handle = world.level.get_entity_chunk(*chunk_pos).await;
        let mut data = chunk_handle.data.lock().await;

        for (uuid, nbt, _, _) in &prepared_data {
            data.insert(*uuid, nbt.clone());
        }
        drop(data);
        chunk_handle.mark_dirty(true);

        world.entities.rcu(|current_entities| {
            let mut new_entities = (**current_entities).clone();

            for (uuid, _, _, entity_ref) in &prepared_data {
                if !new_entities
                    .iter()
                    .any(|e| e.get_entity().entity_uuid == *uuid)
                {
                    new_entities.push(entity_ref.clone());
                }
            }
            new_entities
        });

        for (_, _, packet, _) in prepared_data {
            world.broadcast_packet_all(&packet).await;
        }
    }
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
    pos: &BlockPos,
    distance: f64,
    chunk_pos: &Vector2<i32>,
) -> bool {
    if distance <= 24. * 24. {
        return false;
    }
    // TODO getSharedSpawnPos/WorldSpawnPoint
    if pos.to_centered_f64().squared_distance_to(0., 0., 0.) <= 24. * 24. {
        return false;
    }
    #[expect(clippy::nonminimal_bool)]
    {
        chunk_pos == &Vector2::new(get_section_cord(pos.0.x), get_section_cord(pos.0.z)) || false // TODO canSpawnEntitiesInChunk(ChunkPos chunkPos)
    }
}

#[must_use]
pub async fn get_random_spawn_mob_at(
    world: &Arc<World>,
    category: &'static MobCategory,
    block_pos: &BlockPos,
) -> Option<&'static Spawner> {
    // TODO Holder<Biome> holder = level.getBiome(pos);
    let biome = world.level.get_rough_biome(block_pos).await;
    if category == &MobCategory::WATER_AMBIENT
        && biome.has_tag(&MINECRAFT_REDUCE_WATER_AMBIENT_SPAWNS)
        && rng().random::<f32>() < 0.98f32
    {
        None
    } else {
        // TODO isInNetherFortressBounds(pos, level, cetagory, structureManager) then NetherFortressStructure.FORTRESS_ENEMIES
        // TODO structureManager.getAllStructuresAt(pos); ChunkGenerator::getMobsAt
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
        .choose(&mut rng())
    }
}

pub async fn is_valid_spawn_position_for_type(
    world: &Arc<World>,
    block_pos: &BlockPos,
    category: &'static MobCategory,
    entity_type: &'static EntityType,
    distance: f64,
) -> bool {
    // TODO !SpawnPlacements.checkSpawnRules(entityType, level, EntitySpawnReason.NATURAL, pos, level.random)
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
    if !is_spawn_position_ok(world, block_pos, entity_type).await {
        return false;
    }
    if !check_spawn_rules(entity_type, world, block_pos).await {
        return false;
    }
    // TODO: we should use getSpawnBox, but this is only modified for slimes and magma slimes
    if !world
        .is_space_empty(BoundingBox::new_from_pos(
            f64::from(block_pos.0.x) + 0.5,
            f64::from(block_pos.0.y),
            f64::from(block_pos.0.z) + 0.5,
            &EntityDimensions {
                width: entity_type.dimension[0],
                height: entity_type.dimension[1],
                eye_height: entity_type.eye_height,
            },
        ))
        .await
    {
        return false;
    }
    true
}

pub async fn is_spawn_position_ok(
    world: &Arc<World>,
    block_pos: &BlockPos,
    entity_type: &'static EntityType,
) -> bool {
    match entity_type.spawn_restriction.location {
        SpawnLocation::InLava => world.get_fluid(block_pos).await.has_tag(&MINECRAFT_LAVA),
        SpawnLocation::InWater => {
            // TODO !level.getBlockState(blockPos).isRedstoneConductor(level, blockPos)
            let above_state = world.get_block_state(&block_pos.up()).await;
            world.get_fluid(block_pos).await.has_tag(&MINECRAFT_WATER)
                && !above_state.is_full_cube()
        }
        SpawnLocation::OnGround => {
            let down = world.get_block_state(&block_pos.down()).await;
            let up = world.get_block_state(&block_pos.up()).await;
            let cur = world.get_block_state(block_pos).await;
            // TODO: blockState.allowsSpawning
            let is_valid_spawn_below =
                down.is_side_solid(BlockDirection::Up) && down.luminance < 14;

            if is_valid_spawn_below {
                is_valid_empty_spawn_block(cur) && is_valid_empty_spawn_block(up)
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
                && !above_state.is_full_cube()
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
                is_valid_empty_spawn_block(state) && is_valid_empty_spawn_block(up)
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

pub async fn adjust_spawn_position(
    world: &World,
    pos: BlockPos,
    entity_type: &'static EntityType,
) -> BlockPos {
    if matches!(
        entity_type.spawn_restriction.location,
        SpawnLocation::OnGround
    ) {
        let below = pos.down();
        let state = world.get_block_state(&below).await;
        // Approximation of isPathfindable(LAND)
        if !state.is_full_cube() && !state.is_liquid() {
            return below;
        }
    }
    pos
}

#[must_use]
pub fn is_valid_empty_spawn_block(state: &'static BlockState) -> bool {
    if state.is_full_cube() {
        return false;
    }
    // if state.is_signal_source() {
    //     return false;
    // }
    if state.is_liquid() {
        return false;
    }
    if Block::from_state_id(state.id).has_tag(&MINECRAFT_PREVENT_MOB_SPAWNING_INSIDE) {
        return false;
    }

    // TODO: !entityType.isBlockDangerous(blockState)
    // (e.g., preventing spawns inside Sweet Berry Bushes, Wither Roses, or Fire)
    true
}
