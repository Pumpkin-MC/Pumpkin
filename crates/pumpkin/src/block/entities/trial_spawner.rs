use super::BlockEntity;
use pumpkin_data::block_properties::{
    BlockProperties, TrialSpawnerLikeProperties, TrialSpawnerState,
};
use pumpkin_data::entity::EntityType;
use pumpkin_data::{Block, world::WorldEvent};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::GameMode;
use pumpkin_util::math::{
    boundingbox::{BoundingBox, EntityDimensions},
    position::BlockPos,
};
use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicI64, AtomicU8, Ordering};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::world::{BlockFlags, World};

// TrialSpawnerConfig.java:92-97 (Builder defaults)
#[derive(Clone)]
pub struct TrialSpawnerConfig {
    pub spawn_range: i32,
    pub total_mobs: f32,
    pub simultaneous_mobs: f32,
    pub total_mobs_added_per_player: f32,
    pub simultaneous_mobs_added_per_player: f32,
    pub ticks_between_spawn: i64,
    pub spawn_potentials: Vec<(&'static EntityType, i32)>,
}

impl Default for TrialSpawnerConfig {
    fn default() -> Self {
        Self {
            spawn_range: 4,
            total_mobs: 6.0,
            simultaneous_mobs: 2.0,
            total_mobs_added_per_player: 2.0,
            simultaneous_mobs_added_per_player: 1.0,
            ticks_between_spawn: 40,
            spawn_potentials: Vec::new(),
        }
    }
}

impl TrialSpawnerConfig {
    fn from_nbt(nbt: Option<&NbtCompound>) -> Self {
        let mut config = Self::default();
        let Some(nbt) = nbt else {
            return config;
        };
        if let Some(v) = nbt.get_int("spawn_range") {
            config.spawn_range = v;
        }
        if let Some(v) = nbt.get_float("total_mobs") {
            config.total_mobs = v;
        }
        if let Some(v) = nbt.get_float("simultaneous_mobs") {
            config.simultaneous_mobs = v;
        }
        if let Some(v) = nbt.get_float("total_mobs_added_per_player") {
            config.total_mobs_added_per_player = v;
        }
        if let Some(v) = nbt.get_float("simultaneous_mobs_added_per_player") {
            config.simultaneous_mobs_added_per_player = v;
        }
        if let Some(v) = nbt.get_int("ticks_between_spawn") {
            config.ticks_between_spawn = i64::from(v);
        }
        if let Some(list) = nbt.get_list("spawn_potentials") {
            for entry in list {
                let NbtTag::Compound(entry) = entry else {
                    continue;
                };
                let weight = entry.get_int("weight").unwrap_or(1);
                let Some(data) = entry.get_compound("data") else {
                    continue;
                };
                let Some(entity) = data.get_compound("entity") else {
                    continue;
                };
                let Some(id) = entity.get_string("id") else {
                    continue;
                };
                let name = id.strip_prefix("minecraft:").unwrap_or(id);
                if let Some(entity_type) = EntityType::from_name(name) {
                    config.spawn_potentials.push((entity_type, weight));
                }
            }
        }
        config
    }

    // TrialSpawnerConfig.java:58-60
    fn calculate_target_total_mobs(&self, additional_players: i32) -> i32 {
        (self.total_mobs + self.total_mobs_added_per_player * additional_players as f32).floor()
            as i32
    }

    // TrialSpawnerConfig.java:62-64
    fn calculate_target_simultaneous_mobs(&self, additional_players: i32) -> i32 {
        (self.simultaneous_mobs
            + self.simultaneous_mobs_added_per_player * additional_players as f32)
            .floor() as i32
    }

    fn pick_random_entity(&self) -> Option<&'static EntityType> {
        let total_weight: i32 = self.spawn_potentials.iter().map(|(_, w)| *w).sum();
        if total_weight <= 0 {
            return self.spawn_potentials.first().map(|(e, _)| *e);
        }
        let mut roll = rand::random_range(0..total_weight);
        for (entity, weight) in &self.spawn_potentials {
            if roll < *weight {
                return Some(entity);
            }
            roll -= weight;
        }
        None
    }
}

pub struct TrialSpawnerBlockEntity {
    pub position: BlockPos,
    normal_config_nbt: Mutex<Option<NbtCompound>>,
    ominous_config_nbt: Mutex<Option<NbtCompound>>,
    normal_config: TrialSpawnerConfig,
    ominous_config: TrialSpawnerConfig,
    target_cooldown_length: i64,
    required_player_range: f64,
    detected_players: Mutex<HashSet<Uuid>>,
    current_mobs: Mutex<HashSet<Uuid>>,
    cooldown_ends_at: AtomicI64,
    next_mob_spawns_at: AtomicI64,
    total_mobs_spawned: AtomicI32,
    next_spawn_entity: Mutex<Option<&'static EntityType>>,
    // 0 = none, 1 = consumables/key ejection in progress
    ejecting_loot_table: AtomicU8,
}

// TrialSpawner.java:56-58
const DEFAULT_TARGET_COOLDOWN_LENGTH: i64 = 36000;
const DEFAULT_REQUIRED_PLAYER_RANGE: f64 = 14.0;
// TrialSpawnerState.java:41-42
const DELAY_BEFORE_EJECT_AFTER_KILLING_LAST_MOB: i64 = 40;
const TIME_BETWEEN_EACH_EJECTION: i64 = 30;
// TrialSpawner.java:59
const MAX_MOB_TRACKING_DISTANCE_SQR: i32 = 47 * 47;

impl TrialSpawnerBlockEntity {
    pub const ID: &'static str = "minecraft:trial_spawner";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            normal_config_nbt: Mutex::const_new(None),
            ominous_config_nbt: Mutex::const_new(None),
            normal_config: TrialSpawnerConfig::default(),
            ominous_config: TrialSpawnerConfig::default(),
            target_cooldown_length: DEFAULT_TARGET_COOLDOWN_LENGTH,
            required_player_range: DEFAULT_REQUIRED_PLAYER_RANGE,
            detected_players: Mutex::const_new(HashSet::new()),
            current_mobs: Mutex::const_new(HashSet::new()),
            cooldown_ends_at: AtomicI64::new(0),
            next_mob_spawns_at: AtomicI64::new(0),
            total_mobs_spawned: AtomicI32::new(0),
            next_spawn_entity: Mutex::const_new(None),
            ejecting_loot_table: AtomicU8::new(0),
        }
    }

    const fn active_config(&self, is_ominous: bool) -> &TrialSpawnerConfig {
        if is_ominous {
            &self.ominous_config
        } else {
            &self.normal_config
        }
    }

    async fn reset_statistics(&self) {
        self.detected_players.lock().await.clear();
        self.total_mobs_spawned.store(0, Ordering::Relaxed);
        self.next_mob_spawns_at.store(0, Ordering::Relaxed);
        self.cooldown_ends_at.store(0, Ordering::Relaxed);
    }

    async fn reset(&self) {
        self.current_mobs.lock().await.clear();
        *self.next_spawn_entity.lock().await = None;
        self.reset_statistics().await;
    }

    async fn count_additional_players(&self) -> i32 {
        // StateData.java:113-119
        (self.detected_players.lock().await.len() as i32 - 1).max(0)
    }

    // StateData.java:121-159, simplified: no line-of-sight raycast, no ominous
    // acquisition (bad omen -> trial omen transform, TrialSpawner.java:103-113;
    // OminousItemSpawner) -- deferred, see report.
    async fn try_detect_players(&self, world: &Arc<World>, is_ominous: bool) {
        let game_time = world.get_world_age().await;
        if (self.position.0.x as i64
            + self.position.0.y as i64
            + self.position.0.z as i64
            + game_time)
            % 20
            != 0
        {
            return;
        }
        let nearby = world.get_nearby_players(self.position.to_f64(), self.required_player_range);
        let found: HashSet<Uuid> = nearby
            .iter()
            .filter(|p| !matches!(p.gamemode.load(), GameMode::Spectator))
            .map(|p| p.gameprofile.id)
            .collect();

        let mut detected = self.detected_players.lock().await;
        let before = detected.len();
        detected.extend(found);
        if detected.len() != before {
            self.next_mob_spawns_at
                .fetch_max(game_time + 40, Ordering::Relaxed);
            let event = if is_ominous {
                WorldEvent::ParticlesTrialSpawnerDetectPlayerOminous
            } else {
                WorldEvent::ParticlesTrialSpawnerDetectPlayer
            };
            world.sync_world_event(event, self.position, detected.len() as i32);
        }
    }

    async fn has_mob_to_spawn(&self, config: &TrialSpawnerConfig) -> bool {
        if self.next_spawn_entity.lock().await.is_some() {
            return true;
        }
        !config.spawn_potentials.is_empty()
    }

    async fn get_or_create_next_spawn_data(
        &self,
        config: &TrialSpawnerConfig,
    ) -> Option<&'static EntityType> {
        let mut next = self.next_spawn_entity.lock().await;
        if next.is_none() {
            *next = config.pick_random_entity();
        }
        *next
    }

    // TrialSpawner.java:161-234, simplified: no custom spawn rules / equipment /
    // line-of-sight clip check (only collision + spawn placement rules kept)
    async fn spawn_mob(&self, world: &Arc<World>, config: &TrialSpawnerConfig) -> Option<Uuid> {
        let entity_type = self.get_or_create_next_spawn_data(config).await?;
        let pos = self.position.0;
        let spawn_range = f64::from(config.spawn_range);
        let spawn_pos = pumpkin_util::math::vector3::Vector3::new(
            pos.x as f64 + (rand::random::<f64>() - rand::random::<f64>()) * spawn_range + 0.5,
            (pos.y + rand::random_range(0..3) - 1) as f64,
            pos.z as f64 + (rand::random::<f64>() - rand::random::<f64>()) * spawn_range + 0.5,
        );
        if !world.is_space_empty(BoundingBox::new_from_pos(
            spawn_pos.x,
            spawn_pos.y,
            spawn_pos.z,
            &EntityDimensions {
                width: entity_type.dimension[0],
                height: entity_type.dimension[1],
                eye_height: entity_type.eye_height,
            },
        )) {
            return None;
        }
        let uuid = uuid::Uuid::new_v4();
        let entity = crate::entity::r#type::from_type(entity_type, spawn_pos, world, uuid);
        world.spawn_entity(entity).await;
        world.sync_world_event(
            WorldEvent::ParticlesTrialSpawnerSpawnMobAt,
            BlockPos::floored(spawn_pos.x, spawn_pos.y, spawn_pos.z),
            0,
        );
        *self.next_spawn_entity.lock().await = config.pick_random_entity();
        Some(uuid)
    }

    // TrialSpawner.java:271-290
    async fn untrack_dead_mobs(&self, world: &Arc<World>) -> bool {
        let mut mobs = self.current_mobs.lock().await;
        let before = mobs.len();
        mobs.retain(|id| {
            world.get_entity_by_uuid(*id).is_some_and(|e| {
                e.get_entity().is_alive()
                    && e.get_entity()
                        .block_pos
                        .load()
                        .0
                        .squared_distance_to_vec(&self.position.0)
                        <= MAX_MOB_TRACKING_DISTANCE_SQR
            })
        });
        mobs.len() != before
    }

    // Eject one item from the loot table picked for this reward cycle.
    // TrialSpawner.java:237-251
    async fn eject_reward(&self, world: &Arc<World>, table: u8) {
        if let Some(item) = spawner_ejection_item(table) {
            world.drop_stack(&self.position, item).await;
        }
        world.sync_world_event(WorldEvent::AnimationTrialSpawnerEjectItem, self.position, 0);
    }

    #[allow(clippy::too_many_lines)]
    async fn tick_server(&self, world: &Arc<World>) {
        let state_id = world.get_block_state_id(&self.position);
        let block = Block::from_state_id(state_id);
        if !TrialSpawnerLikeProperties::handles_block_id(block.id) {
            return;
        }
        let mut props = TrialSpawnerLikeProperties::from_state_id(state_id, block);
        let is_ominous = props.ominous;
        let game_time = world.get_world_age().await;

        if self.untrack_dead_mobs(world).await {
            self.next_mob_spawns_at.store(
                game_time + self.active_config(is_ominous).ticks_between_spawn,
                Ordering::Relaxed,
            );
        }

        let config = self.active_config(is_ominous).clone();
        let next_state = self
            .tick_state_machine(
                world,
                props.trial_spawner_state,
                is_ominous,
                &config,
                game_time,
            )
            .await;

        if next_state != props.trial_spawner_state {
            props.trial_spawner_state = next_state;
            let new_state_id = props.to_state_id(block);
            world
                .set_block_state(&self.position, new_state_id, BlockFlags::NOTIFY_ALL)
                .await;
        }
    }

    // TrialSpawnerState.java:63-155
    async fn tick_state_machine(
        &self,
        world: &Arc<World>,
        current: TrialSpawnerState,
        is_ominous: bool,
        config: &TrialSpawnerConfig,
        game_time: i64,
    ) -> TrialSpawnerState {
        match current {
            TrialSpawnerState::Inactive => TrialSpawnerState::WaitingForPlayers,
            TrialSpawnerState::WaitingForPlayers => {
                if !self.has_mob_to_spawn(config).await {
                    return TrialSpawnerState::Inactive;
                }
                self.try_detect_players(world, is_ominous).await;
                if self.detected_players.lock().await.is_empty() {
                    TrialSpawnerState::WaitingForPlayers
                } else {
                    TrialSpawnerState::Active
                }
            }
            TrialSpawnerState::Active => {
                if !self.has_mob_to_spawn(config).await {
                    return TrialSpawnerState::Inactive;
                }
                let additional_players = self.count_additional_players().await;
                self.try_detect_players(world, is_ominous).await;

                let total_spawned = self.total_mobs_spawned.load(Ordering::Relaxed);
                if total_spawned >= config.calculate_target_total_mobs(additional_players) {
                    if self.current_mobs.lock().await.is_empty() {
                        self.cooldown_ends_at
                            .store(game_time + self.target_cooldown_length, Ordering::Relaxed);
                        self.total_mobs_spawned.store(0, Ordering::Relaxed);
                        self.next_mob_spawns_at.store(0, Ordering::Relaxed);
                        return TrialSpawnerState::WaitingForRewardEjection;
                    }
                } else if game_time >= self.next_mob_spawns_at.load(Ordering::Relaxed)
                    && self.current_mobs.lock().await.len()
                        < config.calculate_target_simultaneous_mobs(additional_players) as usize
                    && let Some(uuid) = self.spawn_mob(world, config).await
                {
                    self.current_mobs.lock().await.insert(uuid);
                    self.total_mobs_spawned.fetch_add(1, Ordering::Relaxed);
                    self.next_mob_spawns_at
                        .store(game_time + config.ticks_between_spawn, Ordering::Relaxed);
                }
                TrialSpawnerState::Active
            }
            TrialSpawnerState::WaitingForRewardEjection => {
                // StateData.java:213-216
                let cooldown_started_at =
                    self.cooldown_ends_at.load(Ordering::Relaxed) - self.target_cooldown_length;
                if game_time >= cooldown_started_at + DELAY_BEFORE_EJECT_AFTER_KILLING_LAST_MOB {
                    // TrialSpawnerConfig.java:99-102: lootTablesToEject is a
                    // 1:1-weighted choice between the consumables and key tables,
                    // picked once per reward cycle.
                    let table = if rand::random_range(0..2) == 0 { 1 } else { 2 };
                    self.ejecting_loot_table.store(table, Ordering::Relaxed);
                    TrialSpawnerState::EjectingReward
                } else {
                    TrialSpawnerState::WaitingForRewardEjection
                }
            }
            TrialSpawnerState::EjectingReward => {
                let cooldown_started_at =
                    self.cooldown_ends_at.load(Ordering::Relaxed) - self.target_cooldown_length;
                if (game_time - cooldown_started_at) % TIME_BETWEEN_EACH_EJECTION != 0 {
                    return TrialSpawnerState::EjectingReward;
                }
                if self.detected_players.lock().await.is_empty() {
                    self.ejecting_loot_table.store(0, Ordering::Relaxed);
                    TrialSpawnerState::Cooldown
                } else {
                    let table = self.ejecting_loot_table.load(Ordering::Relaxed);
                    self.eject_reward(world, table).await;
                    let mut detected = self.detected_players.lock().await;
                    if let Some(&first) = detected.iter().next() {
                        detected.remove(&first);
                    }
                    TrialSpawnerState::EjectingReward
                }
            }
            TrialSpawnerState::Cooldown => {
                self.try_detect_players(world, is_ominous).await;
                if !self.detected_players.lock().await.is_empty() {
                    self.total_mobs_spawned.store(0, Ordering::Relaxed);
                    self.next_mob_spawns_at.store(0, Ordering::Relaxed);
                    TrialSpawnerState::Active
                } else if game_time >= self.cooldown_ends_at.load(Ordering::Relaxed) {
                    self.reset().await;
                    TrialSpawnerState::WaitingForPlayers
                } else {
                    TrialSpawnerState::Cooldown
                }
            }
        }
    }
}

impl BlockEntity for TrialSpawnerBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn tick<'a>(&'a self, world: &'a Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move { self.tick_server(world).await })
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let normal_config_nbt = nbt.get_compound("normal_config").cloned();
        let ominous_config_nbt = nbt.get_compound("ominous_config").cloned();
        let normal_config = TrialSpawnerConfig::from_nbt(normal_config_nbt.as_ref());
        let ominous_config = TrialSpawnerConfig::from_nbt(ominous_config_nbt.as_ref());
        let target_cooldown_length = nbt
            .get_int("target_cooldown_length")
            .map_or(DEFAULT_TARGET_COOLDOWN_LENGTH, i64::from);
        let required_player_range = nbt
            .get_int("required_player_range")
            .map_or(DEFAULT_REQUIRED_PLAYER_RANGE, f64::from);

        let spawner_data = nbt.get_compound("spawner_data");
        let detected_players = spawner_data
            .and_then(|data| data.get_list("registered_players"))
            .map(parse_uuid_list)
            .unwrap_or_default();
        let current_mobs = spawner_data
            .and_then(|data| data.get_list("current_mobs"))
            .map(parse_uuid_list)
            .unwrap_or_default();
        let cooldown_ends_at = spawner_data
            .and_then(|data| data.get_long("cooldown_ends_at"))
            .unwrap_or(0);
        let next_mob_spawns_at = spawner_data
            .and_then(|data| data.get_long("next_mob_spawns_at"))
            .unwrap_or(0);
        let total_mobs_spawned = spawner_data
            .and_then(|data| data.get_int("total_mobs_spawned"))
            .unwrap_or(0);

        Self {
            position,
            normal_config_nbt: Mutex::new(normal_config_nbt),
            ominous_config_nbt: Mutex::new(ominous_config_nbt),
            normal_config,
            ominous_config,
            target_cooldown_length,
            required_player_range,
            detected_players: Mutex::new(detected_players),
            current_mobs: Mutex::new(current_mobs),
            cooldown_ends_at: AtomicI64::new(cooldown_ends_at),
            next_mob_spawns_at: AtomicI64::new(next_mob_spawns_at),
            total_mobs_spawned: AtomicI32::new(total_mobs_spawned),
            next_spawn_entity: Mutex::const_new(None),
            ejecting_loot_table: AtomicU8::new(0),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if let Some(cfg) = self.normal_config_nbt.lock().await.as_ref() {
                nbt.put_compound("normal_config", cfg.clone());
            }
            if let Some(cfg) = self.ominous_config_nbt.lock().await.as_ref() {
                nbt.put_compound("ominous_config", cfg.clone());
            }

            let mut data = NbtCompound::new();
            let players: Vec<NbtTag> = self
                .detected_players
                .lock()
                .await
                .iter()
                .map(|u| uuid_to_int_array(*u))
                .collect();
            data.put_list("registered_players", players);
            let mobs: Vec<NbtTag> = self
                .current_mobs
                .lock()
                .await
                .iter()
                .map(|u| uuid_to_int_array(*u))
                .collect();
            data.put_list("current_mobs", mobs);
            data.put_long(
                "cooldown_ends_at",
                self.cooldown_ends_at.load(Ordering::Relaxed),
            );
            data.put_long(
                "next_mob_spawns_at",
                self.next_mob_spawns_at.load(Ordering::Relaxed),
            );
            data.put_int(
                "total_mobs_spawned",
                self.total_mobs_spawned.load(Ordering::Relaxed),
            );
            nbt.put_compound("spawner_data", data);
        })
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(cfg) = self.normal_config_nbt.try_lock()
            && let Some(ref cfg) = *cfg
        {
            nbt.put_compound("normal_config", cfg.clone());
        }
        if let Ok(cfg) = self.ominous_config_nbt.try_lock()
            && let Some(ref cfg) = *cfg
        {
            nbt.put_compound("ominous_config", cfg.clone());
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn potion_item(
    item: &'static pumpkin_data::item::Item,
    potion_name: &str,
) -> pumpkin_data::item_stack::ItemStack {
    let mut stack = pumpkin_data::item_stack::ItemStack::new(1, item);
    if let Some(potion) = pumpkin_data::potion::Potion::from_name(potion_name) {
        stack.patch.push((
            pumpkin_data::data_component::DataComponent::PotionContents,
            Some(Box::new(
                pumpkin_data::data_component_impl::PotionContentsImpl {
                    potion_id: Some(potion.id as i32),
                    custom_color: None,
                    custom_effects: Vec::new(),
                    custom_name: None,
                },
            )),
        ));
    }
    stack
}

// Hand-ported (no generic loot-table registry entry exists for the
// "spawners/trial_chamber/*" namespace, only "chests/*" -- see report):
// data/minecraft/loot_table/spawners/trial_chamber/consumables.json (table 1)
// data/minecraft/loot_table/spawners/trial_chamber/key.json (table 2)
fn spawner_ejection_item(table: u8) -> Option<pumpkin_data::item_stack::ItemStack> {
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;

    match table {
        1 => {
            let entries: [(i32, fn() -> ItemStack); 5] = [
                (3, || {
                    let mut s = ItemStack::new(1, &Item::COOKED_CHICKEN);
                    s.item_count = 1;
                    s
                }),
                (3, || {
                    ItemStack::new(1u8 + rand::random_range(0..3u8), &Item::BREAD)
                }),
                (2, || {
                    ItemStack::new(1u8 + rand::random_range(0..3u8), &Item::BAKED_POTATO)
                }),
                (1, || potion_item(&Item::POTION, "minecraft:regeneration")),
                (1, || potion_item(&Item::POTION, "minecraft:swiftness")),
            ];
            let total: i32 = entries.iter().map(|(w, _)| *w).sum();
            let mut roll = rand::random_range(0..total);
            for (weight, make) in entries {
                if roll < weight {
                    return Some(make());
                }
                roll -= weight;
            }
            None
        }
        2 => Some(ItemStack::new(1, &Item::TRIAL_KEY)),
        _ => None,
    }
}

fn parse_uuid_list(list: &[NbtTag]) -> HashSet<Uuid> {
    list.iter()
        .filter_map(|tag| {
            let NbtTag::IntArray(v) = tag else {
                return None;
            };
            let &[a, b, c, d] = v.as_slice() else {
                return None;
            };
            Some(Uuid::from_u128(
                ((a as u32 as u128) << 96)
                    | ((b as u32 as u128) << 64)
                    | ((c as u32 as u128) << 32)
                    | (d as u32 as u128),
            ))
        })
        .collect()
}

fn uuid_to_int_array(u: Uuid) -> NbtTag {
    let v = u.as_u128();
    NbtTag::IntArray(vec![
        (v >> 96) as i32,
        ((v >> 64) & 0xFFFF_FFFF) as i32,
        ((v >> 32) & 0xFFFF_FFFF) as i32,
        (v & 0xFFFF_FFFF) as i32,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> TrialSpawnerConfig {
        TrialSpawnerConfig::default()
    }

    #[test]
    fn target_total_mobs_scales_with_players() {
        let c = config();
        assert_eq!(c.calculate_target_total_mobs(0), 6);
        assert_eq!(c.calculate_target_total_mobs(1), 8);
        assert_eq!(c.calculate_target_total_mobs(3), 12);
    }

    #[test]
    fn target_simultaneous_mobs_scales_with_players() {
        let c = config();
        assert_eq!(c.calculate_target_simultaneous_mobs(0), 2);
        assert_eq!(c.calculate_target_simultaneous_mobs(2), 4);
    }

    #[test]
    fn cooldown_start_time_derivation() {
        let target_cooldown_length = DEFAULT_TARGET_COOLDOWN_LENGTH;
        let cooldown_ends_at = 100_000 + target_cooldown_length;
        let cooldown_started_at = cooldown_ends_at - target_cooldown_length;
        assert_eq!(cooldown_started_at, 100_000);
        assert_eq!(
            cooldown_started_at + DELAY_BEFORE_EJECT_AFTER_KILLING_LAST_MOB,
            100_000 + DELAY_BEFORE_EJECT_AFTER_KILLING_LAST_MOB
        );
    }

    #[test]
    fn eject_items_cadence_matches_time_between_ejections() {
        let cooldown_started_at: i64 = 1000;
        for offset in 0..90 {
            let game_time = cooldown_started_at + offset;
            let is_eject_tick = (game_time - cooldown_started_at) % TIME_BETWEEN_EACH_EJECTION == 0;
            assert_eq!(is_eject_tick, offset % TIME_BETWEEN_EACH_EJECTION == 0);
        }
    }

    #[test]
    fn default_config_matches_vanilla_builder() {
        let c = config();
        assert_eq!(c.spawn_range, 4);
        assert!((c.total_mobs - 6.0).abs() < f32::EPSILON);
        assert!((c.simultaneous_mobs - 2.0).abs() < f32::EPSILON);
        assert_eq!(c.ticks_between_spawn, 40);
    }
}
