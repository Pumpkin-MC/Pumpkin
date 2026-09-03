use rustc_hash::FxHashSet;
use std::sync::{Arc, Mutex};

use pumpkin_data::block_properties::{TrialSpawnerLikeProperties, TrialSpawnerState};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use uuid::Uuid;

use super::BlockEntity;
use crate::entity::EntityBase;
use crate::world::World;

/// Flame particle type used by trial spawners for world events.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrialSpawnerFlameParticle {
    Normal = 0,
    Ominous = 1,
}

impl TrialSpawnerFlameParticle {
    #[must_use]
    pub const fn encode(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn decode(data: i32) -> Self {
        if data == 1 {
            Self::Ominous
        } else {
            Self::Normal
        }
    }
}

/// Player detector strategies for the trial spawner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PlayerDetector {
    #[default]
    NoCreativePlayers,
    Sheep,
}

impl PlayerDetector {
    #[must_use]
    pub fn detect(&self, world: &World, spawner_pos: BlockPos, range: i32) -> Vec<Uuid> {
        let center = spawner_pos.to_centered_f64();
        let max_dist_sq = f64::from(range * range);

        match self {
            Self::NoCreativePlayers => {
                let players = world.players.load();
                players
                    .iter()
                    .filter_map(|player| {
                        let mode = player.gamemode.load();
                        if mode == pumpkin_util::GameMode::Creative
                            || mode == pumpkin_util::GameMode::Spectator
                        {
                            return None;
                        }
                        let ent = player.get_entity();
                        if !ent.is_alive() {
                            return None;
                        }
                        let pos = ent.pos.load();
                        (pos.squared_distance_to_vec(&center) <= max_dist_sq)
                            .then_some(player.gameprofile.id)
                    })
                    .collect()
            }
            Self::Sheep => {
                let entities = world.entities.load();
                entities
                    .iter()
                    .filter_map(|e| {
                        let ent = e.get_entity();
                        (ent.entity_type.id == EntityType::SHEEP.id
                            && ent.is_alive()
                            && ent.pos.load().squared_distance_to_vec(&center) <= max_dist_sq)
                            .then_some(ent.entity_uuid)
                    })
                    .collect()
            }
        }
    }
}

/// Spawn data configuration for an entity to spawn.
#[derive(Clone, Debug)]
pub struct SpawnData {
    pub entity_type: Option<&'static EntityType>,
    pub custom_spawn_rules: Option<NbtCompound>,
    pub equipment: Option<NbtCompound>,
    pub raw_entity_nbt: Option<NbtCompound>,
}

impl Default for SpawnData {
    fn default() -> Self {
        Self {
            entity_type: Some(&EntityType::ZOMBIE),
            custom_spawn_rules: None,
            equipment: None,
            raw_entity_nbt: None,
        }
    }
}

impl SpawnData {
    #[must_use]
    pub const fn from_entity_type(entity_type: &'static EntityType) -> Self {
        Self {
            entity_type: Some(entity_type),
            custom_spawn_rules: None,
            equipment: None,
            raw_entity_nbt: None,
        }
    }

    #[must_use]
    pub fn from_nbt(nbt: &NbtCompound) -> Self {
        let entity_compound = nbt.get_compound("entity").cloned();
        let entity_type = entity_compound
            .as_ref()
            .and_then(|e| e.get_string("id"))
            .or_else(|| nbt.get_string("id"))
            .and_then(|id| EntityType::from_name(id.strip_prefix("minecraft:").unwrap_or(id)));

        Self {
            entity_type,
            custom_spawn_rules: nbt.get_compound("custom_spawn_rules").cloned(),
            equipment: nbt.get_compound("equipment").cloned(),
            raw_entity_nbt: entity_compound,
        }
    }

    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(entity_type) = self.entity_type {
            let mut entity_compound = self.raw_entity_nbt.clone().unwrap_or_default();
            entity_compound.put_string("id", format!("minecraft:{}", entity_type.resource_name));
            nbt.put_compound("entity", entity_compound);
        }
        if let Some(rules) = &self.custom_spawn_rules {
            nbt.put_compound("custom_spawn_rules", rules.clone());
        }
        if let Some(equipment) = &self.equipment {
            nbt.put_compound("equipment", equipment.clone());
        }
    }
}

#[derive(Clone, Debug)]
pub struct WeightedSpawnData {
    pub data: SpawnData,
    pub weight: i32,
}

#[derive(Clone, Debug)]
pub struct WeightedLootTable {
    pub data: String,
    pub weight: i32,
}

/// Trial spawner configuration for a mode (normal or ominous).
#[derive(Clone, Debug)]
pub struct TrialSpawnerConfig {
    pub spawn_range: i32,
    pub total_mobs: f32,
    pub simultaneous_mobs: f32,
    pub total_mobs_added_per_player: f32,
    pub simultaneous_mobs_added_per_player: f32,
    pub ticks_between_spawn: i32,
    pub spawn_potentials: Vec<WeightedSpawnData>,
    pub loot_tables_to_eject: Vec<WeightedLootTable>,
    pub items_to_drop_when_ominous: Option<String>,
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
            spawn_potentials: vec![WeightedSpawnData {
                data: SpawnData::default(),
                weight: 1,
            }],
            loot_tables_to_eject: Vec::new(),
            items_to_drop_when_ominous: None,
        }
    }
}

impl TrialSpawnerConfig {
    #[must_use]
    pub fn calculate_target_total_mobs(&self, player_count: usize) -> i32 {
        let additional = player_count.saturating_sub(1) as f32 * self.total_mobs_added_per_player;
        (self.total_mobs + additional).floor() as i32
    }

    #[must_use]
    pub fn calculate_target_simultaneous_mobs(&self, player_count: usize) -> i32 {
        let additional =
            player_count.saturating_sub(1) as f32 * self.simultaneous_mobs_added_per_player;
        (self.simultaneous_mobs + additional).floor() as i32
    }

    #[must_use]
    pub fn with_spawning(&self, entity_type: &'static EntityType) -> Self {
        let mut clone = self.clone();
        clone.spawn_potentials = vec![WeightedSpawnData {
            data: SpawnData::from_entity_type(entity_type),
            weight: 1,
        }];
        clone
    }

    #[must_use]
    pub fn pick_spawn_data(&self) -> Option<SpawnData> {
        if self.spawn_potentials.is_empty() {
            return Some(SpawnData::default());
        }
        let total_weight: i32 = self.spawn_potentials.iter().map(|p| p.weight.max(1)).sum();
        if total_weight <= 0 {
            return self.spawn_potentials.first().map(|p| p.data.clone());
        }
        let mut roll = rand::rng().random_range(0..total_weight);
        for potential in &self.spawn_potentials {
            let weight = potential.weight.max(1);
            if roll < weight {
                return Some(potential.data.clone());
            }
            roll -= weight;
        }
        self.spawn_potentials.first().map(|p| p.data.clone())
    }

    #[must_use]
    pub fn pick_loot_table(&self) -> Option<String> {
        if self.loot_tables_to_eject.is_empty() {
            return None;
        }
        let total_weight: i32 = self
            .loot_tables_to_eject
            .iter()
            .map(|p| p.weight.max(1))
            .sum();
        if total_weight <= 0 {
            return self.loot_tables_to_eject.first().map(|p| p.data.clone());
        }
        let mut roll = rand::rng().random_range(0..total_weight);
        for loot in &self.loot_tables_to_eject {
            let weight = loot.weight.max(1);
            if roll < weight {
                return Some(loot.data.clone());
            }
            roll -= weight;
        }
        self.loot_tables_to_eject.first().map(|p| p.data.clone())
    }

    #[must_use]
    pub fn from_nbt(nbt: &NbtCompound) -> Self {
        let mut cfg = Self::default();

        if let Some(v) = nbt.get_int("spawn_range") {
            cfg.spawn_range = v;
        }
        if let Some(v) = nbt
            .get_float("total_mobs")
            .or_else(|| nbt.get_double("total_mobs").map(|d| d as f32))
            .or_else(|| nbt.get_int("total_mobs").map(|i| i as f32))
        {
            cfg.total_mobs = v;
        }
        if let Some(v) = nbt
            .get_float("simultaneous_mobs")
            .or_else(|| nbt.get_double("simultaneous_mobs").map(|d| d as f32))
            .or_else(|| nbt.get_int("simultaneous_mobs").map(|i| i as f32))
        {
            cfg.simultaneous_mobs = v;
        }
        if let Some(v) = nbt
            .get_float("total_mobs_added_per_player")
            .or_else(|| {
                nbt.get_double("total_mobs_added_per_player")
                    .map(|d| d as f32)
            })
            .or_else(|| nbt.get_int("total_mobs_added_per_player").map(|i| i as f32))
        {
            cfg.total_mobs_added_per_player = v;
        }
        if let Some(v) = nbt
            .get_float("simultaneous_mobs_added_per_player")
            .or_else(|| {
                nbt.get_double("simultaneous_mobs_added_per_player")
                    .map(|d| d as f32)
            })
            .or_else(|| {
                nbt.get_int("simultaneous_mobs_added_per_player")
                    .map(|i| i as f32)
            })
        {
            cfg.simultaneous_mobs_added_per_player = v;
        }
        if let Some(v) = nbt.get_int("ticks_between_spawn") {
            cfg.ticks_between_spawn = v;
        }

        if let Some(list) = nbt.get_list("spawn_potentials") {
            let potentials: Vec<WeightedSpawnData> = list
                .iter()
                .filter_map(|tag| {
                    let compound = tag.extract_compound()?;
                    let data_compound = compound.get_compound("data")?;
                    let weight = compound.get_int("weight").unwrap_or(1);
                    Some(WeightedSpawnData {
                        data: SpawnData::from_nbt(data_compound),
                        weight,
                    })
                })
                .collect();
            if !potentials.is_empty() {
                cfg.spawn_potentials = potentials;
            }
        }

        if let Some(list) = nbt.get_list("loot_tables_to_eject") {
            cfg.loot_tables_to_eject = list
                .iter()
                .filter_map(|tag| {
                    let compound = tag.extract_compound()?;
                    let data = compound.get_string("data")?.to_string();
                    let weight = compound.get_int("weight").unwrap_or(1);
                    Some(WeightedLootTable { data, weight })
                })
                .collect();
        }

        if let Some(drop) = nbt.get_string("items_to_drop_when_ominous") {
            cfg.items_to_drop_when_ominous = Some(drop.to_string());
        }

        cfg
    }

    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("spawn_range", self.spawn_range);
        nbt.put_float("total_mobs", self.total_mobs);
        nbt.put_float("simultaneous_mobs", self.simultaneous_mobs);
        nbt.put_float(
            "total_mobs_added_per_player",
            self.total_mobs_added_per_player,
        );
        nbt.put_float(
            "simultaneous_mobs_added_per_player",
            self.simultaneous_mobs_added_per_player,
        );
        nbt.put_int("ticks_between_spawn", self.ticks_between_spawn);

        let mut potentials_list = Vec::new();
        for pot in &self.spawn_potentials {
            let mut pot_cmp = NbtCompound::new();
            pot_cmp.put_int("weight", pot.weight);
            let mut data_cmp = NbtCompound::new();
            pot.data.write_nbt(&mut data_cmp);
            pot_cmp.put_compound("data", data_cmp);
            potentials_list.push(NbtTag::Compound(pot_cmp));
        }
        nbt.put("spawn_potentials", NbtTag::List(potentials_list));

        if !self.loot_tables_to_eject.is_empty() {
            let mut loot_list = Vec::new();
            for loot in &self.loot_tables_to_eject {
                let mut loot_cmp = NbtCompound::new();
                loot_cmp.put_string("data", loot.data.clone());
                loot_cmp.put_int("weight", loot.weight);
                loot_list.push(NbtTag::Compound(loot_cmp));
            }
            nbt.put("loot_tables_to_eject", NbtTag::List(loot_list));
        }

        if let Some(drop) = &self.items_to_drop_when_ominous {
            nbt.put_string("items_to_drop_when_ominous", drop.clone());
        }
    }
}

/// Combined normal and ominous configuration with shared range and cooldown parameters.
#[derive(Clone, Debug)]
pub struct TrialSpawnerFullConfig {
    pub normal: TrialSpawnerConfig,
    pub ominous: TrialSpawnerConfig,
    pub target_cooldown_length: i64,
    pub required_player_range: i32,
}

impl Default for TrialSpawnerFullConfig {
    fn default() -> Self {
        Self {
            normal: TrialSpawnerConfig::default(),
            ominous: TrialSpawnerConfig::default(),
            target_cooldown_length: 36000,
            required_player_range: 14,
        }
    }
}

impl TrialSpawnerFullConfig {
    #[must_use]
    pub fn override_entity(&self, entity_type: &'static EntityType) -> Self {
        Self {
            normal: self.normal.with_spawning(entity_type),
            ominous: self.ominous.with_spawning(entity_type),
            target_cooldown_length: self.target_cooldown_length,
            required_player_range: self.required_player_range,
        }
    }

    #[must_use]
    pub fn from_nbt(nbt: &NbtCompound) -> Self {
        let normal = nbt
            .get_compound("normal_config")
            .map(TrialSpawnerConfig::from_nbt)
            .unwrap_or_default();
        let ominous = nbt
            .get_compound("ominous_config")
            .map(TrialSpawnerConfig::from_nbt)
            .unwrap_or_default();

        let target_cooldown_length = nbt
            .get_long("target_cooldown_length")
            .or_else(|| nbt.get_int("target_cooldown_length").map(i64::from))
            .unwrap_or(36000);

        let required_player_range = nbt.get_int("required_player_range").unwrap_or(14);

        Self {
            normal,
            ominous,
            target_cooldown_length,
            required_player_range,
        }
    }

    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        let mut normal_cmp = NbtCompound::new();
        self.normal.write_nbt(&mut normal_cmp);
        nbt.put_compound("normal_config", normal_cmp);

        let mut ominous_cmp = NbtCompound::new();
        self.ominous.write_nbt(&mut ominous_cmp);
        nbt.put_compound("ominous_config", ominous_cmp);

        nbt.put_long("target_cooldown_length", self.target_cooldown_length);
        nbt.put_int("required_player_range", self.required_player_range);
    }
}

/// Dynamic state data tracking current progress of a trial spawner.
#[derive(Clone, Debug, Default)]
pub struct TrialSpawnerStateData {
    pub registered_players: FxHashSet<Uuid>,
    pub current_mobs: FxHashSet<Uuid>,
    pub cooldown_ends_at: i64,
    pub next_mob_spawns_at: i64,
    pub total_mobs_spawned: i32,
    pub next_spawn_data: Option<SpawnData>,
    pub ejecting_loot_table: Option<String>,
}

impl TrialSpawnerStateData {
    pub fn reset(&mut self) {
        self.registered_players.clear();
        self.current_mobs.clear();
        self.total_mobs_spawned = 0;
        self.next_mob_spawns_at = 0;
        self.cooldown_ends_at = 0;
        self.next_spawn_data = None;
        self.ejecting_loot_table = None;
    }

    pub fn reset_after_becoming_ominous(&mut self, game_time: i64) {
        self.current_mobs.clear();
        self.next_mob_spawns_at = game_time + TrialSpawner::DETECT_PLAYER_SPAWN_BUFFER;
        self.total_mobs_spawned = 0;
        self.next_spawn_data = None;
    }

    #[must_use]
    pub fn has_mob_to_spawn(&self, config: &TrialSpawnerConfig) -> bool {
        if self
            .next_spawn_data
            .as_ref()
            .and_then(|d| d.entity_type)
            .is_some()
        {
            return true;
        }
        !config.spawn_potentials.is_empty()
    }

    pub fn get_or_create_next_spawn_data(
        &mut self,
        config: &TrialSpawnerConfig,
    ) -> Option<&SpawnData> {
        if self.next_spawn_data.is_none() {
            self.next_spawn_data = config.pick_spawn_data();
        }
        self.next_spawn_data.as_ref()
    }

    pub fn try_detect_players(
        &mut self,
        world: &World,
        spawner_pos: BlockPos,
        range: i32,
        detector: &PlayerDetector,
    ) -> bool {
        let detected = detector.detect(world, spawner_pos, range);
        let mut newly_detected = false;
        for uuid in detected {
            if self.registered_players.insert(uuid) {
                newly_detected = true;
            }
        }
        newly_detected
    }

    #[must_use]
    pub fn from_nbt(nbt: &NbtCompound) -> Self {
        let mut data = Self::default();

        let read_uuid_list = |name: &str| -> FxHashSet<Uuid> {
            let mut set = FxHashSet::default();
            if let Some(list) = nbt.get_list(name) {
                for tag in list {
                    match tag {
                        NbtTag::IntArray(arr) if arr.len() == 4 => {
                            let value = ((arr[0] as u128) << 96)
                                | (((arr[1] as u32) as u128) << 64)
                                | (((arr[2] as u32) as u128) << 32)
                                | ((arr[3] as u32) as u128);
                            set.insert(Uuid::from_u128(value));
                        }
                        NbtTag::String(s) => {
                            if let Ok(uuid) = Uuid::parse_str(s) {
                                set.insert(uuid);
                            }
                        }
                        _ => {}
                    }
                }
            }
            set
        };

        data.registered_players = read_uuid_list("registered_players");
        data.current_mobs = read_uuid_list("current_mobs");

        if let Some(v) = nbt
            .get_long("cooldown_ends_at")
            .or_else(|| nbt.get_int("cooldown_ends_at").map(i64::from))
        {
            data.cooldown_ends_at = v;
        }

        if let Some(v) = nbt
            .get_long("next_mob_spawns_at")
            .or_else(|| nbt.get_int("next_mob_spawns_at").map(i64::from))
        {
            data.next_mob_spawns_at = v;
        }

        if let Some(v) = nbt.get_int("total_mobs_spawned") {
            data.total_mobs_spawned = v;
        }

        if let Some(spawn_data_cmp) = nbt.get_compound("spawn_data") {
            data.next_spawn_data = Some(SpawnData::from_nbt(spawn_data_cmp));
        }

        if let Some(loot) = nbt.get_string("ejecting_loot_table") {
            data.ejecting_loot_table = Some(loot.to_string());
        }

        data
    }

    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        let write_uuid_list = |set: &FxHashSet<Uuid>| -> Vec<NbtTag> {
            set.iter()
                .map(|u| {
                    let value = u.as_u128();
                    NbtTag::IntArray(vec![
                        (value >> 96) as i32,
                        ((value >> 64) & 0xFFFF_FFFF) as i32,
                        ((value >> 32) & 0xFFFF_FFFF) as i32,
                        (value & 0xFFFF_FFFF) as i32,
                    ])
                })
                .collect()
        };

        nbt.put(
            "registered_players",
            NbtTag::List(write_uuid_list(&self.registered_players)),
        );
        nbt.put(
            "current_mobs",
            NbtTag::List(write_uuid_list(&self.current_mobs)),
        );
        nbt.put_long("cooldown_ends_at", self.cooldown_ends_at);
        nbt.put_long("next_mob_spawns_at", self.next_mob_spawns_at);
        nbt.put_int("total_mobs_spawned", self.total_mobs_spawned);

        if let Some(spawn_data) = &self.next_spawn_data {
            let mut spawn_cmp = NbtCompound::new();
            spawn_data.write_nbt(&mut spawn_cmp);
            nbt.put_compound("spawn_data", spawn_cmp);
        }

        if let Some(loot) = &self.ejecting_loot_table {
            nbt.put_string("ejecting_loot_table", loot.clone());
        }
    }
}

/// Core Trial Spawner engine and state machine logic.
#[derive(Default)]
pub struct TrialSpawner {
    pub data: TrialSpawnerStateData,
    pub config: TrialSpawnerFullConfig,
    pub player_detector: PlayerDetector,
    pub override_peaceful_and_mob_spawn_rule: bool,
    pub is_ominous: bool,
}

impl TrialSpawner {
    pub const DETECT_PLAYER_SPAWN_BUFFER: i64 = 40;
    pub const DEFAULT_TARGET_COOLDOWN_LENGTH: i64 = 36000;
    pub const DEFAULT_PLAYER_SCAN_RANGE: i32 = 14;
    pub const MAX_MOB_TRACKING_DISTANCE: f64 = 47.0;
    pub const MAX_MOB_TRACKING_DISTANCE_SQR: f64 = 47.0 * 47.0;
    pub const SPAWNING_AMBIENT_SOUND_CHANCE: f32 = 0.02;

    #[must_use]
    pub fn new(config: TrialSpawnerFullConfig, player_detector: PlayerDetector) -> Self {
        Self {
            data: TrialSpawnerStateData::default(),
            config,
            player_detector,
            override_peaceful_and_mob_spawn_rule: false,
            is_ominous: false,
        }
    }

    #[must_use]
    pub const fn active_config(&self) -> &TrialSpawnerConfig {
        if self.is_ominous {
            &self.config.ominous
        } else {
            &self.config.normal
        }
    }

    #[must_use]
    pub const fn normal_config(&self) -> &TrialSpawnerConfig {
        &self.config.normal
    }

    #[must_use]
    pub const fn ominous_config(&self) -> &TrialSpawnerConfig {
        &self.config.ominous
    }

    pub fn load(&mut self, nbt: &NbtCompound) {
        if let Some(spawner_data_cmp) = nbt.get_compound("spawner_data") {
            self.data = TrialSpawnerStateData::from_nbt(spawner_data_cmp);
        } else {
            self.data = TrialSpawnerStateData::from_nbt(nbt);
        }
        self.config = TrialSpawnerFullConfig::from_nbt(nbt);
    }

    pub fn store(&self, nbt: &mut NbtCompound) {
        self.config.write_nbt(nbt);
        self.data.write_nbt(nbt);
    }

    pub fn apply_ominous(&mut self, world: &Arc<World>, spawner_pos: BlockPos) {
        self.set_ominous(world, spawner_pos, true);
        world.sync_world_event(
            WorldEvent::ParticlesTrialSpawnerBecomeOminous,
            spawner_pos,
            1,
        );
        self.is_ominous = true;
        let game_time = world
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .query_gametime();
        self.data.reset_after_becoming_ominous(game_time);
    }

    pub fn remove_ominous(&mut self, world: &Arc<World>, spawner_pos: BlockPos) {
        self.set_ominous(world, spawner_pos, false);
        self.is_ominous = false;
    }

    #[must_use]
    pub const fn is_ominous(&self) -> bool {
        self.is_ominous
    }

    #[must_use]
    pub const fn target_cooldown_length(&self) -> i64 {
        self.config.target_cooldown_length
    }

    #[must_use]
    pub const fn required_player_range(&self) -> i32 {
        self.config.required_player_range
    }

    pub fn get_state(&self, world: &World, spawner_pos: BlockPos) -> TrialSpawnerState {
        let block_id = world.get_block_state_id(&spawner_pos);
        let block = world.get_block(&spawner_pos);
        if block.id == pumpkin_data::BlockId::TRIAL_SPAWNER {
            TrialSpawnerLikeProperties::from_state_id(block_id).trial_spawner_state
        } else {
            TrialSpawnerState::Inactive
        }
    }

    pub fn set_state(&self, world: &Arc<World>, spawner_pos: BlockPos, state: TrialSpawnerState) {
        let block_id = world.get_block_state_id(&spawner_pos);
        let block = world.get_block(&spawner_pos);
        if block.id == pumpkin_data::BlockId::TRIAL_SPAWNER {
            let mut props = TrialSpawnerLikeProperties::from_state_id(block_id);
            if props.trial_spawner_state != state {
                props.trial_spawner_state = state;
                world.set_block_state(
                    &spawner_pos,
                    props.to_state_id(block),
                    pumpkin_world::world::BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }

    pub fn set_ominous(&self, world: &Arc<World>, spawner_pos: BlockPos, ominous: bool) {
        let block_id = world.get_block_state_id(&spawner_pos);
        let block = world.get_block(&spawner_pos);
        if block.id == pumpkin_data::BlockId::TRIAL_SPAWNER {
            let mut props = TrialSpawnerLikeProperties::from_state_id(block_id);
            if props.ominous != ominous {
                props.ominous = ominous;
                world.set_block_state(
                    &spawner_pos,
                    props.to_state_id(block),
                    pumpkin_world::world::BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }

    #[must_use]
    pub fn can_spawn_in_level(&self, world: &World) -> bool {
        let level_info = world.level_info.load();
        if !level_info.game_rules.spawner_blocks_work {
            return false;
        }
        if self.override_peaceful_and_mob_spawn_rule {
            return true;
        }
        if level_info.difficulty == Difficulty::Peaceful {
            return false;
        }
        level_info.game_rules.spawn_mobs
    }

    pub fn spawn_mob(&mut self, world: &Arc<World>, spawner_pos: BlockPos) -> Option<Uuid> {
        let (entity_type, spawn_range) = {
            let active_cfg = if self.is_ominous {
                &self.config.ominous
            } else {
                &self.config.normal
            };
            let spawn_data = self.data.get_or_create_next_spawn_data(active_cfg);
            let ent_type = spawn_data.and_then(|sd| sd.entity_type)?;
            (ent_type, active_cfg.spawn_range)
        };

        let mut rng = rand::rng();

        let mut chosen_pos = None;
        for _ in 0..20 {
            let spawn_pos = Vector3::new(
                f64::from(spawner_pos.0.x)
                    + (rng.random::<f64>() - rng.random::<f64>()) * f64::from(spawn_range)
                    + 0.5,
                f64::from(spawner_pos.0.y + rng.random_range(0..3) - 1),
                f64::from(spawner_pos.0.z)
                    + (rng.random::<f64>() - rng.random::<f64>()) * f64::from(spawn_range)
                    + 0.5,
            );

            let bb = entity_type.get_spawn_bounding_box(spawn_pos.x, spawn_pos.y, spawn_pos.z);
            if !world.is_space_empty(bb) {
                continue;
            }

            let center = spawner_pos.to_centered_f64();
            if !Self::in_line_of_sight(world, center, spawn_pos) {
                continue;
            }

            chosen_pos = Some(spawn_pos);
            break;
        }

        let spawn_pos = chosen_pos?;
        let entity_uuid = Uuid::new_v4();
        let entity = crate::entity::r#type::from_type(entity_type, spawn_pos, world, entity_uuid);

        let yaw = rng.random::<f32>() * 360.0;
        entity.get_entity().set_rotation(yaw, 0.0);

        let mut event =
            crate::plugin::api::events::entity::trial_spawner_spawn::TrialSpawnerSpawnEvent::new(
                entity.get_entity().entity_id,
                spawner_pos,
            );
        if let Some(server) = world.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        if event.cancelled {
            return None;
        }

        world.spawn_entity(entity);

        let flame_data = i32::from(self.is_ominous);
        world.sync_world_event(
            WorldEvent::ParticlesTrialSpawnerSpawn,
            spawner_pos,
            flame_data,
        );
        let spawn_block_pos = BlockPos::floored(spawn_pos.x, spawn_pos.y, spawn_pos.z);
        world.sync_world_event(
            WorldEvent::ParticlesTrialSpawnerSpawnMobAt,
            spawn_block_pos,
            flame_data,
        );

        Some(entity_uuid)
    }

    fn in_line_of_sight(world: &Arc<World>, origin: Vector3<f64>, dest: Vector3<f64>) -> bool {
        let origin_block = BlockPos::floored(origin.x, origin.y, origin.z);
        let hit = world.raycast(dest, origin, |pos, w| {
            let state = w.get_block_state(pos);
            !state.is_air() && !state.sided_transparency()
        });
        match hit {
            None => true,
            Some((hit_pos, _)) => hit_pos == origin_block,
        }
    }

    pub fn eject_reward(
        &self,
        world: &Arc<World>,
        spawner_pos: BlockPos,
        ejecting_loot_table: Option<&str>,
    ) {
        let mut dropped_any = false;
        if let Some(key) = ejecting_loot_table
            && let Some(loot_table) = pumpkin_data::loot_table::get_loot_table(key)
        {
            let seed = rand::random::<i64>();
            let items = crate::world::loot::generate_loot(loot_table, seed);
            for stack in items {
                world.drop_stack(&spawner_pos, stack);
                dropped_any = true;
            }
        }

        if !dropped_any {
            let key_item = if self.is_ominous {
                &Item::OMINOUS_TRIAL_KEY
            } else {
                &Item::TRIAL_KEY
            };
            world.drop_stack(&spawner_pos, ItemStack::new(1, key_item));
        }

        world.sync_world_event(WorldEvent::AnimationTrialSpawnerEjectItem, spawner_pos, 0);
    }

    pub fn tick_server(&mut self, world: &Arc<World>, spawner_pos: BlockPos, is_ominous: bool) {
        self.is_ominous = is_ominous;
        let current_state = self.get_state(world, spawner_pos);

        let max_dist_sq = Self::MAX_MOB_TRACKING_DISTANCE_SQR;
        let center = spawner_pos.to_centered_f64();
        let mut removed = false;
        self.data.current_mobs.retain(|id| {
            if let Some(entity) = world.get_entity_by_uuid(*id) {
                let ent = entity.get_entity();
                if ent.is_alive() && ent.pos.load().squared_distance_to_vec(&center) <= max_dist_sq
                {
                    return true;
                }
            }
            removed = true;
            false
        });

        if removed {
            let game_time = world
                .level_time
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .query_gametime();
            self.data.next_mob_spawns_at =
                game_time + i64::from(self.active_config().ticks_between_spawn);
        }

        let next_state = self.tick_and_get_next(world, spawner_pos, current_state);
        if next_state != current_state {
            self.set_state(world, spawner_pos, next_state);
        }
    }

    #[allow(clippy::too_many_lines)]
    fn tick_and_get_next(
        &mut self,
        world: &Arc<World>,
        spawner_pos: BlockPos,
        current_state: TrialSpawnerState,
    ) -> TrialSpawnerState {
        let game_time = world
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .query_gametime();
        let can_spawn = self.can_spawn_in_level(world);
        let has_mob = self.data.has_mob_to_spawn(self.active_config());

        match current_state {
            TrialSpawnerState::Inactive => {
                if can_spawn && has_mob {
                    TrialSpawnerState::WaitingForPlayers
                } else {
                    TrialSpawnerState::Inactive
                }
            }
            TrialSpawnerState::WaitingForPlayers => {
                if !can_spawn || !has_mob {
                    return TrialSpawnerState::Inactive;
                }

                let newly_detected = self.data.try_detect_players(
                    world,
                    spawner_pos,
                    self.config.required_player_range,
                    &self.player_detector,
                );

                if self.data.registered_players.is_empty() {
                    TrialSpawnerState::WaitingForPlayers
                } else {
                    if newly_detected {
                        let player_count = self.data.registered_players.len() as i32;
                        let event = if self.is_ominous {
                            WorldEvent::ParticlesTrialSpawnerDetectPlayerOminous
                        } else {
                            WorldEvent::ParticlesTrialSpawnerDetectPlayer
                        };
                        world.sync_world_event(event, spawner_pos, player_count);
                        world.play_sound(
                            Sound::BlockTrialSpawnerDetectPlayer,
                            SoundCategory::Blocks,
                            &spawner_pos.to_f64(),
                        );
                        world.play_sound(
                            Sound::BlockTrialSpawnerOpenShutter,
                            SoundCategory::Blocks,
                            &spawner_pos.to_f64(),
                        );
                        self.data.next_mob_spawns_at = game_time + Self::DETECT_PLAYER_SPAWN_BUFFER;
                    }
                    TrialSpawnerState::Active
                }
            }
            TrialSpawnerState::Active => {
                if !can_spawn || !has_mob {
                    return TrialSpawnerState::Inactive;
                }

                self.data.try_detect_players(
                    world,
                    spawner_pos,
                    self.config.required_player_range,
                    &self.player_detector,
                );

                let (total_mobs_needed, max_simultaneous, ticks_between_spawn) = {
                    let active_config = self.active_config();
                    (
                        active_config
                            .calculate_target_total_mobs(self.data.registered_players.len()),
                        active_config
                            .calculate_target_simultaneous_mobs(self.data.registered_players.len()),
                        active_config.ticks_between_spawn,
                    )
                };

                if self.data.total_mobs_spawned >= total_mobs_needed
                    && self.data.current_mobs.is_empty()
                {
                    self.data.cooldown_ends_at = game_time + self.config.target_cooldown_length;
                    self.data.total_mobs_spawned = 0;
                    self.data.ejecting_loot_table = self.active_config().pick_loot_table();
                    self.data.next_mob_spawns_at = game_time + 40;
                    TrialSpawnerState::WaitingForRewardEjection
                } else {
                    if self.data.total_mobs_spawned < total_mobs_needed
                        && (self.data.current_mobs.len() as i32) < max_simultaneous
                        && game_time >= self.data.next_mob_spawns_at
                        && let Some(uuid) = self.spawn_mob(world, spawner_pos)
                    {
                        self.data.current_mobs.insert(uuid);
                        self.data.total_mobs_spawned += 1;
                        self.data.next_mob_spawns_at = game_time + i64::from(ticks_between_spawn);
                    }
                    TrialSpawnerState::Active
                }
            }
            TrialSpawnerState::WaitingForRewardEjection => {
                if game_time >= self.data.next_mob_spawns_at {
                    world.play_sound(
                        Sound::BlockTrialSpawnerSpawnItemBegin,
                        SoundCategory::Blocks,
                        &spawner_pos.to_f64(),
                    );
                    self.data.next_mob_spawns_at = game_time + 20;
                    TrialSpawnerState::EjectingReward
                } else {
                    TrialSpawnerState::WaitingForRewardEjection
                }
            }
            TrialSpawnerState::EjectingReward => {
                if game_time >= self.data.next_mob_spawns_at {
                    self.eject_reward(world, spawner_pos, self.data.ejecting_loot_table.as_deref());
                    world.play_sound(
                        Sound::BlockTrialSpawnerEjectItem,
                        SoundCategory::Blocks,
                        &spawner_pos.to_f64(),
                    );
                    world.play_sound(
                        Sound::BlockTrialSpawnerCloseShutter,
                        SoundCategory::Blocks,
                        &spawner_pos.to_f64(),
                    );
                    self.data.next_mob_spawns_at = game_time + 20;
                    TrialSpawnerState::Cooldown
                } else {
                    TrialSpawnerState::EjectingReward
                }
            }
            TrialSpawnerState::Cooldown => {
                if game_time >= self.data.cooldown_ends_at {
                    self.data.reset();
                    if self.is_ominous {
                        self.remove_ominous(world, spawner_pos);
                    }
                    TrialSpawnerState::WaitingForPlayers
                } else {
                    TrialSpawnerState::Cooldown
                }
            }
        }
    }

    pub fn override_entity_to_spawn(
        &mut self,
        entity_type: &'static EntityType,
        world: &Arc<World>,
        spawner_pos: BlockPos,
    ) {
        self.data.reset();
        self.config = self.config.override_entity(entity_type);
        self.set_state(world, spawner_pos, TrialSpawnerState::Inactive);
    }

    pub const fn set_player_detector(&mut self, player_detector: PlayerDetector) {
        self.player_detector = player_detector;
    }

    pub const fn override_peaceful_and_mob_spawn_rule(&mut self) {
        self.override_peaceful_and_mob_spawn_rule = true;
    }
}

pub struct TrialSpawnerBlockEntity {
    pub position: BlockPos,
    pub trial_spawner: Mutex<TrialSpawner>,
}

impl BlockEntity for TrialSpawnerBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn tick(&self, world: &Arc<World>) {
        let block_id = world.get_block_state_id(&self.position);
        let block = world.get_block(&self.position);
        let is_ominous = if block.id == pumpkin_data::BlockId::TRIAL_SPAWNER {
            TrialSpawnerLikeProperties::from_state_id(block_id).ominous
        } else {
            false
        };

        if let Ok(mut spawner) = self.trial_spawner.lock() {
            spawner.tick_server(world, self.position, is_ominous);
        }
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let mut spawner = TrialSpawner::default();
        spawner.load(nbt);
        Self {
            position,
            trial_spawner: Mutex::new(spawner),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(spawner) = self.trial_spawner.lock() {
            spawner.store(nbt);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        self.trial_spawner.try_lock().ok().map(|spawner| {
            let mut nbt = NbtCompound::new();
            spawner.store(&mut nbt);
            nbt
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TrialSpawnerBlockEntity {
    pub const ID: &'static str = "minecraft:trial_spawner";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            trial_spawner: Mutex::new(TrialSpawner::default()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trial_spawner_config_scaling() {
        let config = TrialSpawnerConfig {
            total_mobs: 6.0,
            simultaneous_mobs: 2.0,
            total_mobs_added_per_player: 2.0,
            simultaneous_mobs_added_per_player: 1.0,
            ..Default::default()
        };

        // 1 player
        assert_eq!(config.calculate_target_total_mobs(1), 6);
        assert_eq!(config.calculate_target_simultaneous_mobs(1), 2);

        // 2 players
        assert_eq!(config.calculate_target_total_mobs(2), 8);
        assert_eq!(config.calculate_target_simultaneous_mobs(2), 3);

        // 3 players
        assert_eq!(config.calculate_target_total_mobs(3), 10);
        assert_eq!(config.calculate_target_simultaneous_mobs(3), 4);
    }

    #[test]
    fn trial_spawner_nbt_roundtrip() {
        let mut original = TrialSpawner::default();
        let test_uuid = Uuid::new_v4();
        original.data.registered_players.insert(test_uuid);
        original.data.total_mobs_spawned = 3;
        original.data.cooldown_ends_at = 12345;
        original.config.target_cooldown_length = 36000;
        original.config.required_player_range = 14;

        let mut nbt = NbtCompound::new();
        original.store(&mut nbt);

        let mut loaded = TrialSpawner::default();
        loaded.load(&nbt);

        assert_eq!(loaded.data.total_mobs_spawned, 3);
        assert_eq!(loaded.data.cooldown_ends_at, 12345);
        assert_eq!(loaded.config.target_cooldown_length, 36000);
        assert_eq!(loaded.config.required_player_range, 14);
        assert!(loaded.data.registered_players.contains(&test_uuid));
    }
}
