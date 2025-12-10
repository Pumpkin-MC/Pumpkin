use crate::entity::EntityBase;
use crate::entity::r#type::from_type;
use crate::world::World;
use pumpkin_data::biome::Spawner;
use pumpkin_data::entity::{EntityType, MobCategory, SpawnLocation};
use pumpkin_data::tag::Block::MINECRAFT_PREVENT_MOB_SPAWNING_INSIDE;
use pumpkin_data::tag::Fluid::{MINECRAFT_LAVA, MINECRAFT_WATER};
use pumpkin_data::tag::Taggable;
use pumpkin_data::tag::WorldgenBiome::MINECRAFT_REDUCE_WATER_AMBIENT_SPAWNS;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_util::GameMode;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::get_section_cord;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::chunk::{ChunkData, ChunkHeightmapType};
use rand::seq::IndexedRandom;
use rand::{Rng, rng};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

const MAGIC_NUMBER: i32 = 17 * 17;

#[derive(Default, Debug)]
pub struct MobCounts([i32; 8]);

impl MobCounts {
    #[inline]
    const fn add(&mut self, category: &'static MobCategory) {
        self.0[category.id] += 1;
    }
    #[inline]
    const fn can_spawn(&self, category: &'static MobCategory) -> bool {
        self.0[category.id] < category.max
    }
}

pub struct LocalMobCapCalculator {
    world: Arc<World>,
    player_mob_counts: HashMap<i32, MobCounts>,
    players_near_chunk: HashMap<Vector2<i32>, Vec<i32>>,
}

impl fmt::Debug for LocalMobCapCalculator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LocalMobCapCalculator")
            .field("world", &"<skipped>")
            .field("player_mob_counts", &self.player_mob_counts)
            .field("players_near_chunk", &self.players_near_chunk)
            .finish()
    }
}

impl LocalMobCapCalculator {
    pub fn new(world: &Arc<World>) -> Self {
        Self {
            world: world.clone(), // can anybody get rid of this clone?
            player_mob_counts: HashMap::new(),
            players_near_chunk: HashMap::new(),
        }
    }

    const fn calc_distance(chunk_pos: Vector2<i32>, player_pos: &Vector3<f64>) -> f64 {
        let dx = ((chunk_pos.x << 4) + 8) as f64 - player_pos.x;
        let dy = ((chunk_pos.y << 4) + 8) as f64 - player_pos.z;
        dx * dx + dy * dy
    }

    async fn get_players_near<'b>(
        players_near_chunk: &'b mut HashMap<Vector2<i32>, Vec<i32>>,
        world: &Arc<World>,
        chunk_pos: &Vector2<i32>,
    ) -> &'b Vec<i32> {
        match players_near_chunk.entry(*chunk_pos) {
            Entry::Occupied(value) => {
                // debug!("chunk {chunk_pos:?} near player {:?}", value.get());
                value.into_mut()
            }
            Entry::Vacant(entry) => {
                let mut players = Vec::new();
                for (_uuid, player) in world.players.read().await.iter() {
                    if player.gamemode.load() == GameMode::Spectator {
                        continue;
                    }
                    if Self::calc_distance(*chunk_pos, &player.position()) < 16384. {
                        players.push(player.entity_id());
                    }
                }
                // debug!("chunk {chunk_pos:?} near player {:?}", players);
                entry.insert(players)
            }
        }
    }
    pub async fn add_mob(&mut self, chunk_pos: &Vector2<i32>, category: &'static MobCategory) {
        let players =
            Self::get_players_near(&mut self.players_near_chunk, &self.world, chunk_pos).await;
        for player in players {
            self.player_mob_counts
                .entry(*player)
                .or_default()
                .add(category);
        }
        // debug!("player_mob_counts {:?}", self.player_mob_counts);
        // debug!("players_near_chunk {:?}", self.players_near_chunk);
        // debug!("chunk_pos {:?}", chunk_pos);
        // debug!("players {:?}", players);
    }
    pub async fn can_spawn(
        &mut self,
        category: &'static MobCategory,
        chunk_pos: &Vector2<i32>,
    ) -> bool {
        let players =
            Self::get_players_near(&mut self.players_near_chunk, &self.world, chunk_pos).await;
        for player in players {
            if let Some(count) = self.player_mob_counts.get(player) {
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

#[derive(Debug)]
struct PointCharge(Vector3<f64>, f64); // pos charge

impl PointCharge {
    fn get_potential_change(&self, pos: &BlockPos) -> f64 {
        let dst = self.0.sub(&pos.to_f64()).length();
        self.1 / dst
    }
}

#[derive(Default, Debug)]
struct PotentialCalculator(Vec<PointCharge>);

impl PotentialCalculator {
    pub fn add_charge(&mut self, pos: &BlockPos, charge: f64) {
        if charge != 0. {
            self.0.push(PointCharge(pos.to_f64(), charge));
        }
    }
    pub fn get_potential_energy_change(&self, pos: &BlockPos, charge: f64) -> f64 {
        if charge == 0. {
            return 0.;
        }
        let mut sum: f64 = 0.;
        for i in &self.0 {
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
    // unmodifiable_mob_category_counts: MobCounts, seems only for debug
    last_checked_pos: BlockPos,
    last_checked_type: &'static EntityType,
    last_charge: f64,
}

impl fmt::Debug for SpawnState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SpawnState")
            .field("spawnable_chunk_count", &self.spawnable_chunk_count)
            .field("mob_category_counts", &self.mob_category_counts)
            .field("spawn_potential", &self.spawn_potential)
            .field("local_mob_cap_calculator", &self.local_mob_cap_calculator)
            .field("last_checked_pos", &self.last_checked_pos)
            .field("last_checked_type", &self.last_checked_type.resource_name)
            .field("last_charge", &self.last_charge)
            .finish()
    }
}

impl SpawnState {
    pub async fn new(
        chunk_count: i32,
        entities: &Arc<RwLock<HashMap<Uuid, Arc<dyn EntityBase>>>>,
        world: &Arc<World>,
    ) -> Self {
        let mut potential = PotentialCalculator::default();
        let mut local_mob_cap = LocalMobCapCalculator::new(world);
        let mut counter = MobCounts::default();
        for entity in entities.read().await.values() {
            let entity = entity.get_entity();
            let entity_type = entity.entity_type;
            if !entity_type.mob || entity_type.category == &MobCategory::MISC {
                // TODO (mob.isPersistenceRequired() || mob.requiresCustomPersistence())
                continue;
            }
            let entity_pos = entity.block_pos.load();
            let biome = world.level.get_rough_biome(&entity_pos).await;
            if let Some(cost) = biome.spawn_costs.get(entity_type.resource_name) {
                potential.add_charge(&entity_pos, cost.charge);
            }
            if entity_type.mob {
                local_mob_cap
                    .add_mob(&entity.chunk_pos.load(), entity_type.category)
                    .await;
            }
            counter.add(entity_type.category);
        }
        Self {
            spawnable_chunk_count: chunk_count,
            mob_category_counts: counter,
            spawn_potential: potential,
            local_mob_cap_calculator: local_mob_cap,
            // unmodifiable_mob_category_counts: counter,
            last_checked_pos: BlockPos::new(i32::MAX, i32::MAX, i32::MAX),
            last_checked_type: &EntityType::PLAYER,
            last_charge: 0.,
        }
    }
    #[inline]
    fn can_spawn_for_category_global(&self, category: &'static MobCategory) -> bool {
        self.mob_category_counts.0[category.id]
            < category.max * self.spawnable_chunk_count / MAGIC_NUMBER
    }
    async fn can_spawn_for_category_local(
        &mut self,
        category: &'static MobCategory,
        chunk_pos: &Vector2<i32>,
    ) -> bool {
        self.local_mob_cap_calculator
            .can_spawn(category, chunk_pos)
            .await
    }
    async fn can_spawn(
        &mut self,
        entity_type: &'static EntityType,
        pos: &BlockPos,
        world: &Arc<World>,
    ) -> bool {
        self.last_checked_pos = *pos;
        self.last_checked_type = entity_type;
        // TODO get biome
        let biome = world.level.get_rough_biome(pos).await;
        if let Some(cost) = biome.spawn_costs.get(entity_type.resource_name) {
            self.last_charge = cost.charge;
            self.spawn_potential
                .get_potential_energy_change(pos, cost.charge)
                <= cost.energy_budget
        } else {
            self.last_charge = 0.;
            true
        }
    }
    async fn after_spawn(
        &mut self,
        entity_type: &'static EntityType,
        pos: &BlockPos,
        world: &Arc<World>,
    ) {
        let charge;
        if self.last_checked_pos.eq(pos) && self.last_checked_type == entity_type {
            charge = self.last_charge;
        } else {
            // TODO get biome
            let biome = world.level.get_rough_biome(pos).await;
            if let Some(cost) = biome.spawn_costs.get(entity_type.resource_name) {
                charge = cost.charge;
            } else {
                charge = 0.;
            }
        }

        self.spawn_potential.add_charge(pos, charge);
        self.mob_category_counts.add(entity_type.category);
        self.local_mob_cap_calculator
            .add_mob(
                &Vector2::<i32>::new(get_section_cord(pos.0.x), get_section_cord(pos.0.z)),
                entity_type.category,
            )
            .await;
    }
}

#[must_use]
pub fn get_filtered_spawning_categories(
    state: &SpawnState,
    spawn_friendlies: bool,
    spawn_enemies: bool,
    spawn_passives: bool,
) -> Vec<&'static MobCategory> {
    let mut ret = Vec::with_capacity(8);
    for category in MobCategory::SPAWNING_CATEGORIES {
        if (spawn_friendlies || !category.is_friendly)
            && (spawn_enemies || category.is_friendly)
            && (spawn_passives || !category.is_persistent)
            && state.can_spawn_for_category_global(category)
        {
            ret.push(category);
        }
    }
    ret
}

pub async fn spawn_for_chunk(
    world: &Arc<World>,
    chunk_pos: &Vector2<i32>,
    chunk: &Arc<RwLock<ChunkData>>,
    spawn_state: &mut SpawnState,
    spawn_list: &Vec<&'static MobCategory>,
) {
    // debug!("spawn for chunk {:?}", chunk_pos);
    for category in spawn_list {
        if spawn_state
            .can_spawn_for_category_local(category, chunk_pos)
            .await
        {
            let random_pos = get_random_pos_within(world.min_y, chunk_pos, chunk).await;
            // debug!("try random pos: {:?}", random_pos);
            if random_pos.0.y > world.min_y {
                spawn_category_for_position(category, world, random_pos, chunk_pos, spawn_state)
                    .await;
            }
        }
    }
}
pub async fn get_random_pos_within(
    min_y: i32,
    chunk_pos: &Vector2<i32>,
    chunk: &Arc<RwLock<ChunkData>>,
) -> BlockPos {
    let x = (chunk_pos.x << 4) + rng().random_range(0..16);
    let z = (chunk_pos.y << 4) + rng().random_range(0..16);
    let temp_y =
        chunk
            .read()
            .await
            .heightmap
            .get_height(ChunkHeightmapType::WorldSurface, x, z, min_y)
            + 1;
    let y = rng().random_range(min_y..=temp_y);
    BlockPos::new(x, y, z)
}

pub async fn spawn_category_for_position(
    category: &'static MobCategory,
    world: &Arc<World>,
    pos: BlockPos,
    chunk_pos: &Vector2<i32>,
    spawn_state: &mut SpawnState,
) {
    // TODO StructureManager structureManager = level.structureManager();
    // TODO blockState.isRedstoneConductor(chunk, pos) is true then return
    let mut spawn_cluster_size = 0;
    let mut new_pos = pos;
    for _ in 0..3 {
        let mut new_x = new_pos.0.x;
        let mut new_z = new_pos.0.z;
        let mut random_group_size = (rng().random::<f32>() * 4.).ceil() as i32;
        let mut inc = 0;
        #[allow(unused_variables)]
        let group_size = 0;
        'outer: while inc < random_group_size {
            new_x += rng().random_range(0..6) - rng().random_range(0..6);
            new_z += rng().random_range(0..6) - rng().random_range(0..6);
            new_pos = BlockPos::new(new_x, new_pos.0.y, new_z);
            let new_pos_center = new_pos.to_centered_f64();
            let player_distance = get_nearest_player(&new_pos_center, world).await;
            if !is_right_distance_to_player_and_spawn_point(
                &new_pos,
                player_distance,
                world,
                chunk_pos,
            ) {
                // debug!("{new_pos:?} failed, too near to player or spawn point dst: {player_distance}");
                inc += 1;
                continue;
            }
            let Some(spawner) = get_random_spawn_mob_at(world, category, &new_pos).await else {
                // debug!("{new_pos:?} failed, no random spawn mob at category: {category:?}");
                break 'outer;
            };
            random_group_size = rng().random_range(spawner.min_count..=spawner.max_count);
            let entity_type =
                &EntityType::from_name(spawner.r#type.strip_prefix("minecraft:").unwrap()).unwrap();
            if !is_valid_spawn_position_for_type(
                world,
                &new_pos,
                category,
                entity_type,
                player_distance,
            )
            .await
            {
                // debug!("{new_pos:?} failed, not valid spawn position");
                inc += 1;
                continue;
            }
            if !spawn_state.can_spawn(entity_type, &new_pos, world).await {
                // debug!("{new_pos:?} failed, can't spawn at");
                inc += 1;
                continue;
            }
            let entity = from_type(entity_type, new_pos_center, world, Uuid::new_v4()).await;
            entity
                .get_entity()
                .set_rotation(rng().random::<f32>() * 360., 0.);
            // TODO isValidPositionForMob(level, mob, f)
            // TODO spawnGroupData = mob.finalizeSpawn(level, level.getCurrentDifficultyAt(mob.blockPosition()), EntitySpawnReason.NATURAL, spawnGroupData);
            spawn_cluster_size += 1;
            //group_size += 1;
            world.spawn_entity(entity).await;
            spawn_state.after_spawn(entity_type, &new_pos, world).await;
            if spawn_cluster_size >= entity_type.limit_per_chunk {
                return;
            }

            //TODO mob.isMaxGroupSizeReached(p)
            inc += 1;
        }
    }
}

pub async fn get_nearest_player(pos: &Vector3<f64>, world: &Arc<World>) -> f64 {
    let mut dst = f64::MAX;
    for (_uuid, player) in world.players.read().await.iter() {
        if player.gamemode.load() == GameMode::Spectator {
            continue;
        }
        let cur_dst = player.position().squared_distance_to_vec(*pos);
        if cur_dst < dst {
            dst = cur_dst;
        }
    }
    dst
}
pub fn is_right_distance_to_player_and_spawn_point(
    pos: &BlockPos,
    distance: f64,
    _world: &Arc<World>,
    chunk_pos: &Vector2<i32>,
) -> bool {
    if distance <= 24. * 24. {
        return false;
    }
    // TODO getSharedSpawnPos/WorldSpawnPoint
    if pos.to_centered_f64().squared_distance_to(0., 0., 0.) <= 24. * 24. {
        return false;
    }
    #[allow(clippy::overly_complex_bool_expr)]
    #[allow(clippy::nonminimal_bool)]
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
    // TODO: we should use getSpawnBox, but this is only modified for slimes and magma slimes
    world
        .is_space_empty(BoundingBox::new_from_pos(
            f64::from(block_pos.0.x) + 0.5,
            f64::from(block_pos.0.y),
            f64::from(block_pos.0.z) + 0.5,
            &EntityDimensions {
                width: entity_type.dimension[0],
                height: entity_type.dimension[1],
            },
        ))
        .await
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
            world.get_fluid(block_pos).await.has_tag(&MINECRAFT_WATER)
        }
        SpawnLocation::OnGround => {
            let down = world.get_block_state(&block_pos.down()).await;
            let up = world.get_block_state(&block_pos.up()).await;
            let cur = world.get_block_state(block_pos).await;
            // TODO: blockState.allowsSpawning
            if down.is_side_solid(BlockDirection::Up) {
                is_valid_empty_spawn_block(cur) && is_valid_empty_spawn_block(up)
            } else {
                false
            }
        }
        SpawnLocation::Unrestricted => true,
    }
}

#[must_use]
pub fn is_valid_empty_spawn_block(state: &'static BlockState) -> bool {
    // TODO: emitsRedstonePower
    if state.is_full_cube() || state.is_liquid() {
        return false;
    }
    // TODO !entityType.isBlockDangerous(blockState);
    !Block::from_state_id(state.id).has_tag(&MINECRAFT_PREVENT_MOB_SPAWNING_INSIDE)
}
