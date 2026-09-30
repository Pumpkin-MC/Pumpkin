//! End Dragon fight manager. Handles the dragon lifecycle, end crystal
//! spawning, boss bar, gateway spawning, respawn sequence, and exit portal.
//!
//! Matches vanilla `net.minecraft.world.level.dimension.end.EnderDragonFight` behaviour
//! and `DragonRespawnStage`.

use std::sync::{Arc, Mutex};
use tracing::{debug, info};
use uuid::Uuid;

use pumpkin_data::{Block, entity::EntityType, world::WorldEvent};
use pumpkin_util::{
    math::{position::BlockPos, vector2::Vector2, vector3::Vector3},
    text::TextComponent,
};
use pumpkin_world::world::BlockFlags;

use super::{
    World,
    bossbar::{Bossbar, BossbarColor, BossbarDivisions, BossbarFlags},
};
use crate::entity::{Entity, EntityBase, decoration::end_crystal::EndCrystalEntity};

// ── Constants (match vanilla exactly) ────────────────────────────────────────

pub const MAX_TICKS_BEFORE_DRAGON_RESPAWN: i32 = 1200;
pub const TIME_BETWEEN_CRYSTAL_SCANS: i32 = 100;
pub const TIME_BETWEEN_PLAYER_SCANS: i32 = 20;
pub const ARENA_SIZE_CHUNKS: i32 = 8;
pub const ARENA_TICKET_LEVEL: i32 = 9;
pub const GATEWAY_COUNT: usize = 20;
pub const GATEWAY_DISTANCE: f64 = 96.0;
pub const GATEWAY_Y: i32 = 75;
pub const DRAGON_SPAWN_Y: i32 = 128;
pub const ARENA_RADIUS: f64 = 192.0;
pub const EVENT_DISPLAY_NAME: &str = "entity.minecraft.ender_dragon";

// ── End Spike definition ─────────────────────────────────────────────────────

pub use pumpkin_world::generation::feature::features::end_spike::Spike as EndSpike;

// ── Respawn stage ─────────────────────────────────────────────────────────────

/// Mirrors vanilla `net.minecraft.world.level.dimension.end.DragonRespawnStage`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragonRespawnStage {
    Start,
    PreparingToSummonPillars,
    SummoningPillars,
    SummoningDragon,
    End,
}

// ── DragonFight ───────────────────────────────────────────────────────────────

pub struct DragonFight {
    // ── Persistent state (SavedData in vanilla) ───────────────────────────────
    pub needs_state_scanning: bool,
    pub dragon_killed: bool,
    pub previously_killed: bool,
    pub respawn_stage: Option<DragonRespawnStage>,
    pub respawn_time: i32,
    pub dragon_uuid: Option<Uuid>,
    pub exit_portal_location: Option<BlockPos>,
    pub origin: BlockPos,
    pub gateways: Vec<i32>,
    pub respawn_crystals: Vec<Uuid>,

    // ── Transient counters & testing flags ────────────────────────────────────
    pub skip_arena_loaded_check: bool,
    pub alive_crystals: i32,
    ticks_since_dragon_seen: i32,
    ticks_since_crystals_scanned: i32,
    ticks_since_last_player_scan: i32,

    // ── Boss bar ──────────────────────────────────────────────────────────────
    bossbar_uuid: Uuid,
    bossbar_players: Vec<Uuid>,
}

impl Default for DragonFight {
    fn default() -> Self {
        Self::new()
    }
}

impl DragonFight {
    #[must_use]
    pub fn new() -> Self {
        Self::new_with_seed(0, BlockPos::new(0, 0, 0))
    }

    #[must_use]
    pub fn new_with_seed(seed: u64, origin: BlockPos) -> Self {
        let mut gateways: Vec<i32> = (0..(GATEWAY_COUNT as i32)).collect();
        // Shuffle gateways with seed or thread-local rng
        if seed != 0 {
            use rand::SeedableRng;
            use rand::seq::SliceRandom;
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            gateways.shuffle(&mut rng);
        } else {
            use rand::seq::SliceRandom;
            let mut rng = rand::rng();
            gateways.shuffle(&mut rng);
        }

        Self {
            needs_state_scanning: true,
            dragon_killed: false,
            previously_killed: false,
            respawn_stage: None,
            respawn_time: 0,
            dragon_uuid: None,
            exit_portal_location: None,
            origin,
            gateways,
            respawn_crystals: Vec::new(),
            skip_arena_loaded_check: false,
            alive_crystals: 0,
            ticks_since_dragon_seen: 0,
            ticks_since_crystals_scanned: 0,
            // Start above threshold so first tick with players triggers a scan
            ticks_since_last_player_scan: TIME_BETWEEN_PLAYER_SCANS + 1,
            bossbar_uuid: Uuid::new_v4(),
            bossbar_players: Vec::new(),
        }
    }

    pub fn init(&mut self, seed: u64, origin: BlockPos) {
        use rand::SeedableRng;
        use rand::seq::SliceRandom;

        self.origin = origin;
        if self.gateways.is_empty() {
            let mut new_gateways: Vec<i32> = (0..(GATEWAY_COUNT as i32)).collect();
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            new_gateways.shuffle(&mut rng);
            self.gateways = new_gateways;
        }
    }

    pub const fn skip_arena_loaded_check(&mut self) {
        self.skip_arena_loaded_check = true;
    }

    #[must_use]
    pub fn get_spikes() -> [EndSpike; 10] {
        pumpkin_world::generation::feature::features::end_spike::EndSpikeFeature::get_spikes_for_seed(0)
    }

    // ── Main tick ─────────────────────────────────────────────────────────────

    pub fn tick(fight_mutex: &Mutex<Self>, world: &Arc<World>) {
        let mut fight = fight_mutex
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        // 1. Update boss-bar recipients every 20 ticks
        fight.ticks_since_last_player_scan += 1;
        if fight.ticks_since_last_player_scan >= TIME_BETWEEN_PLAYER_SCANS {
            fight.update_players(world);
            fight.ticks_since_last_player_scan = 0;
        }

        // Nothing to do without nearby players
        if fight.bossbar_players.is_empty() {
            return;
        }

        if !fight.is_arena_loaded(world) {
            return;
        }

        // 2. One-time state scan on the first populated tick
        if fight.needs_state_scanning {
            fight.scan_state(world);
            fight.needs_state_scanning = false;
        }

        // 3. Respawn sequence
        if let Some(stage) = fight.respawn_stage {
            fight.tick_respawn_stage(world, stage);
            fight.respawn_time += 1;
            return;
        }

        // 4. Normal fight ticking
        if !fight.dragon_killed {
            fight.ticks_since_dragon_seen += 1;
            if fight.dragon_uuid.is_none()
                || fight.ticks_since_dragon_seen >= MAX_TICKS_BEFORE_DRAGON_RESPAWN
            {
                fight.find_or_create_dragon(world);
                fight.ticks_since_dragon_seen = 0;
            }

            fight.ticks_since_crystals_scanned += 1;
            if fight.ticks_since_crystals_scanned >= TIME_BETWEEN_CRYSTAL_SCANS {
                fight.update_crystal_count(world);
                fight.ticks_since_crystals_scanned = 0;
            }
        }
    }

    // ── State scanning ────────────────────────────────────────────────────────

    pub fn scan_state(&mut self, world: &Arc<World>) {
        info!("Scanning for legacy world dragon fight...");

        let active_portal_exists = Self::has_active_exit_portal(world);

        if active_portal_exists {
            info!("Found that the dragon has been killed in this world already.");
            self.previously_killed = true;
        } else {
            info!("Found that the dragon has not yet been killed in this world.");
            self.previously_killed = false;
            if self.find_exit_portal(world).is_none() {
                self.spawn_exit_portal(world, false);
            }
            self.spawn_crystals(world);
        }

        // Reconcile any live dragon entity.
        let existing = {
            let entities = world.entities.load();
            entities
                .iter()
                .find(|e| e.get_entity().entity_type == &EntityType::ENDER_DRAGON)
                .map(|e| (e.get_entity().entity_uuid, e.clone()))
        };

        match existing {
            Some((uuid, _entity)) if !active_portal_exists => {
                info!("Found that there's a dragon still alive ({:?})", uuid);
                self.dragon_uuid = Some(uuid);
                self.dragon_killed = false;
            }
            Some((uuid, entity)) => {
                info!(
                    "Found that there's a dragon still alive ({:?}), but we have an active portal. Removing it.",
                    uuid
                );
                entity.get_entity().remove();
                self.dragon_uuid = None;
                self.dragon_killed = true;
            }
            None => {
                self.dragon_killed = true;
            }
        }

        if !self.previously_killed && self.dragon_killed {
            self.dragon_killed = false;
        }
    }

    /// Checks whether an active exit portal (`EndPortalBlockEntity`) exists within an 8-chunk radius
    /// of the origin, matching vanilla's `hasActiveExitPortal`.
    pub fn has_active_exit_portal(world: &Arc<World>) -> bool {
        for cx in -8i32..=8 {
            for cz in -8i32..=8 {
                let entities = world
                    .block_entities
                    .get(&Vector2::new(cx, cz))
                    .map(|m| m.values().cloned().collect::<Vec<_>>());
                if let Some(list) = entities {
                    for be in list {
                        if be
                            .as_any()
                            .is::<crate::block::entities::end_portal::EndPortalBlockEntity>()
                        {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // ── Exit portal pattern & discovery ───────────────────────────────────────

    fn matches_exit_portal_pattern(world: &Arc<World>, pos: BlockPos) -> bool {
        let check = |dx: i32, dy: i32, dz: i32| -> bool {
            let p = BlockPos::new(pos.0.x + dx, pos.0.y + dy, pos.0.z + dz);
            world.get_block(&p) == &Block::BEDROCK
        };

        // Central pillar (height 4: dy 0..=3)
        for dy in 0..=3 {
            if !check(0, dy, 0) {
                return false;
            }
        }

        // Rim at dy = 0
        for dx in -1..=1 {
            if !check(dx, 0, -3) || !check(dx, 0, 3) {
                return false;
            }
        }
        if !check(-2, 0, -2) || !check(2, 0, -2) || !check(-2, 0, 2) || !check(2, 0, 2) {
            return false;
        }
        if !check(-3, 0, -1)
            || !check(3, 0, -1)
            || !check(-3, 0, 0)
            || !check(3, 0, 0)
            || !check(-3, 0, 1)
            || !check(3, 0, 1)
        {
            return false;
        }

        // Bowl at dy = -1
        for dx in -1..=1 {
            if !check(dx, -1, -2) || !check(dx, -1, 2) {
                return false;
            }
        }
        for dz in -1..=1 {
            for dx in -2..=2 {
                if !check(dx, -1, dz) {
                    return false;
                }
            }
        }

        true
    }

    pub fn find_exit_portal(&mut self, world: &Arc<World>) -> Option<BlockPos> {
        if let Some(loc) = self.exit_portal_location
            && Self::matches_exit_portal_pattern(world, loc)
        {
            return Some(loc);
        }

        let origin_chunk_x = self.origin.0.x >> 4;
        let origin_chunk_z = self.origin.0.z >> 4;

        for cx in (-8 + origin_chunk_x)..=(8 + origin_chunk_x) {
            for cz in (-8 + origin_chunk_z)..=(8 + origin_chunk_z) {
                let entities = world
                    .block_entities
                    .get(&Vector2::new(cx, cz))
                    .map(|m| m.iter().map(|(p, be)| (*p, be.clone())).collect::<Vec<_>>());
                if let Some(list) = entities {
                    for (pos, be) in list {
                        if be
                            .as_any()
                            .is::<crate::block::entities::end_portal::EndPortalBlockEntity>()
                            || world.get_block(&pos) == &Block::END_PORTAL
                        {
                            let candidate = BlockPos::new(pos.0.x, pos.0.y, pos.0.z);
                            if Self::matches_exit_portal_pattern(world, candidate) {
                                self.exit_portal_location = Some(candidate);
                                return Some(candidate);
                            }
                        }
                    }
                }
            }
        }

        let top_y = world
            .get_top_block(Vector2::new(self.origin.0.x, self.origin.0.z))
            .max(100);
        for y in (40..=top_y).rev() {
            let candidate = BlockPos::new(self.origin.0.x, y, self.origin.0.z);
            if Self::matches_exit_portal_pattern(world, candidate) {
                self.exit_portal_location = Some(candidate);
                return Some(candidate);
            }
        }

        None
    }

    #[must_use]
    pub fn is_arena_loaded(&self, world: &Arc<World>) -> bool {
        if self.skip_arena_loaded_check {
            return true;
        }
        let center_chunk_x = self.origin.0.x >> 4;
        let center_chunk_z = self.origin.0.z >> 4;
        for dx in -3..=3 {
            for dz in -3..=3 {
                let cp = Vector2::new(center_chunk_x + dx, center_chunk_z + dz);
                if !world.level.is_chunk_loaded(&cp) {
                    return false;
                }
            }
        }
        true
    }

    // ── Dragon lifecycle ──────────────────────────────────────────────────────

    pub fn find_or_create_dragon(&mut self, world: &Arc<World>) {
        let uuid = {
            let entities = world.entities.load();
            entities
                .iter()
                .find(|e| e.get_entity().entity_type == &EntityType::ENDER_DRAGON)
                .map(|e| e.get_entity().entity_uuid)
        };

        if let Some(u) = uuid {
            debug!("Haven't seen our dragon, but found another one to use.");
            self.dragon_uuid = Some(u);
            self.ticks_since_dragon_seen = 0;
        } else {
            debug!("Haven't seen the dragon, respawning it");
            self.create_new_dragon(world);
        }
    }

    pub fn create_new_dragon(&mut self, world: &Arc<World>) -> Option<Uuid> {
        if !self.previously_killed && self.find_exit_portal(world).is_none() {
            self.spawn_exit_portal(world, false);
        }

        let uuid = Uuid::new_v4();
        let spawn_y = self.origin.0.y + DRAGON_SPAWN_Y;
        let position = Vector3::new(
            self.origin.0.x as f64 + 0.5,
            spawn_y as f64,
            self.origin.0.z as f64 + 0.5,
        );
        let dragon =
            crate::entity::r#type::from_type(&EntityType::ENDER_DRAGON, position, world, uuid);

        if let Some(dragon_entity) = dragon
            .cast_any()
            .downcast_ref::<crate::entity::boss::ender_dragon::EnderDragonEntity>(
        ) {
            dragon_entity.set_fight_origin(self.exit_portal_location.unwrap_or(self.origin));
        }

        world.spawn_entity_non_save(dragon);
        self.dragon_uuid = Some(uuid);
        self.dragon_killed = false;
        info!("Spawned ender dragon {:?}.", uuid);
        Some(uuid)
    }

    pub fn update_dragon(
        &mut self,
        world: &Arc<World>,
        dragon_uuid: Uuid,
        health: f32,
        max_health: f32,
        custom_name: Option<&TextComponent>,
    ) {
        if Some(dragon_uuid) != self.dragon_uuid {
            return;
        }

        self.ticks_since_dragon_seen = 0;
        let fraction = if max_health > 0.0 {
            (health / max_health).clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.update_bossbar_health(world, fraction);

        if let Some(name) = custom_name {
            self.update_bossbar_title(world, name);
        }

        if let Some(loc) = self.exit_portal_location
            && let Some(dragon_uuid) = self.dragon_uuid
            && let Some(e) = world
                .entities
                .load()
                .iter()
                .find(|e| e.get_entity().entity_uuid == dragon_uuid)
            && let Some(dragon) = e
                .cast_any()
                .downcast_ref::<crate::entity::boss::ender_dragon::EnderDragonEntity>()
        {
            dragon.set_fight_origin(loc);
        }
    }

    pub fn set_dragon_killed(&mut self, world: &Arc<World>, killed_uuid: Uuid) {
        if self.dragon_uuid.is_some() && Some(killed_uuid) != self.dragon_uuid {
            return;
        }

        self.update_bossbar_health(world, 0.0);
        self.remove_all_bossbar(world);

        // Activate the exit portal.
        self.spawn_exit_portal(world, true);

        // Spawn a new end gateway.
        self.spawn_new_gateway(world);

        // Place the dragon egg on the first kill.
        if !self.previously_killed {
            let podium_pos = self.exit_portal_location.unwrap_or(self.origin);
            let egg_pos = BlockPos::new(podium_pos.0.x, podium_pos.0.y + 4, podium_pos.0.z);
            world.set_block_state(
                &egg_pos,
                Block::DRAGON_EGG.default_state.id,
                BlockFlags::NOTIFY_LISTENERS,
            );
        }

        self.previously_killed = true;
        self.dragon_killed = true;
        self.dragon_uuid = None;
    }

    pub fn update_crystal_count(&mut self, world: &Arc<World>) {
        self.ticks_since_crystals_scanned = 0;
        self.alive_crystals = 0;

        let spikes = Self::get_spikes();
        let entities = world.entities.load();

        for spike in &spikes {
            let (min_x, min_y, min_z, max_x, max_y, max_z) = spike.top_bounding_box();
            for entity in entities.iter() {
                if entity.get_entity().entity_type == &EntityType::END_CRYSTAL {
                    let pos = entity.get_entity().pos.load();
                    if pos.x >= min_x
                        && pos.x <= max_x
                        && pos.y >= min_y
                        && pos.y <= max_y
                        && pos.z >= min_z
                        && pos.z <= max_z
                    {
                        self.alive_crystals += 1;
                    }
                }
            }
        }

        debug!("Found {} end crystals still alive.", self.alive_crystals);
    }

    // ── Crystal destruction ───────────────────────────────────────────────────

    pub fn on_crystal_destroyed(&mut self, world: &Arc<World>, crystal_uuid: Uuid) {
        if self.respawn_stage.is_some() && self.respawn_crystals.contains(&crystal_uuid) {
            self.abort_respawn_sequence(world);
        } else {
            self.update_crystal_count(world);
        }
    }

    pub fn abort_respawn_sequence(&mut self, world: &Arc<World>) {
        debug!("Aborting dragon respawn sequence.");
        self.respawn_stage = None;
        self.respawn_time = 0;
        self.reset_spike_crystals(world);
        self.spawn_exit_portal(world, true);
    }

    // ── Respawn sequence ──────────────────────────────────────────────────────

    pub fn try_respawn(&mut self, world: &Arc<World>) {
        if !self.dragon_killed || self.respawn_stage.is_some() {
            return;
        }

        let mut location = self.exit_portal_location;
        if location.is_none() {
            debug!("Tried to respawn, but need to find the portal first.");
            if let Some(found) = self.find_exit_portal(world) {
                debug!("Found the exit portal & saved its location for next time.");
                location = Some(found);
            } else {
                debug!("Couldn't find a portal, so we made one.");
                self.spawn_exit_portal(world, true);
                location = self.exit_portal_location;
            }
        }

        let Some(portal_loc) = location else {
            return;
        };

        let center = BlockPos::new(portal_loc.0.x, portal_loc.0.y + 1, portal_loc.0.z);
        let offsets = [
            (0, -3), // North
            (0, 3),  // South
            (-3, 0), // West
            (3, 0),  // East
        ];

        let entities = world.entities.load();
        let mut crystals = Vec::new();

        for (dx, dz) in offsets {
            let target_pos = BlockPos::new(center.0.x + dx, center.0.y, center.0.z + dz);
            let found = entities.iter().find(|e| {
                if e.get_entity().entity_type != &EntityType::END_CRYSTAL {
                    return false;
                }
                let pos = e.get_entity().pos.load();
                (pos.x - (target_pos.0.x as f64 + 0.5)).abs() < 1.5
                    && (pos.y - target_pos.0.y as f64).abs() < 1.5
                    && (pos.z - (target_pos.0.z as f64 + 0.5)).abs() < 1.5
            });

            if let Some(crystal) = found {
                crystals.push(crystal.get_entity().entity_uuid);
            } else {
                return;
            }
        }

        debug!("Found all crystals, respawning dragon.");
        self.respawn_dragon(world, crystals);
    }

    pub fn respawn_dragon(&mut self, world: &Arc<World>, crystals: Vec<Uuid>) {
        if !self.dragon_killed || self.respawn_stage.is_some() {
            return;
        }

        if let Some(loc) = self.find_exit_portal(world).or(self.exit_portal_location) {
            for dy in -1i32..=4 {
                for dx in -3i32..=3 {
                    for dz in -3i32..=3 {
                        let p = BlockPos::new(loc.0.x + dx, loc.0.y + dy, loc.0.z + dz);
                        let b = world.get_block(&p);
                        if b == &Block::BEDROCK || b == &Block::END_PORTAL {
                            world.set_block_state(
                                &p,
                                Block::END_STONE.default_state.id,
                                BlockFlags::NOTIFY_ALL,
                            );
                        }
                    }
                }
            }
        }

        self.respawn_stage = Some(DragonRespawnStage::Start);
        self.respawn_time = 0;
        self.spawn_exit_portal(world, false);
        self.respawn_crystals = crystals;
    }

    pub fn set_respawn_stage(&mut self, world: &Arc<World>, stage: DragonRespawnStage) {
        if self.respawn_stage.is_none() {
            tracing::warn!("Dragon respawn isn't in progress, can't skip ahead in the animation.");
            return;
        }

        self.respawn_time = 0;
        if stage == DragonRespawnStage::End {
            self.respawn_stage = None;
            self.dragon_killed = false;
            self.create_new_dragon(world);
        } else {
            self.respawn_stage = Some(stage);
        }
    }

    #[expect(clippy::too_many_lines)]
    fn tick_respawn_stage(&mut self, world: &Arc<World>, stage: DragonRespawnStage) {
        let time = self.respawn_time;
        let origin_y = self.origin.0.y + DRAGON_SPAWN_Y;
        let origin_target = BlockPos::new(self.origin.0.x, origin_y, self.origin.0.z);

        let entities = world.entities.load();
        let mut live_crystals = Vec::new();
        for uuid in &self.respawn_crystals {
            if let Some(e) = entities
                .iter()
                .find(|e| e.get_entity().entity_uuid == *uuid)
                && let Some(crystal) = e.cast_any().downcast_ref::<EndCrystalEntity>()
            {
                live_crystals.push(crystal);
            }
        }

        if live_crystals.is_empty() {
            self.abort_respawn_sequence(world);
            return;
        }

        match stage {
            DragonRespawnStage::Start => {
                for crystal in &live_crystals {
                    crystal.set_beam_target(Some(origin_target));
                }
                self.set_respawn_stage(world, DragonRespawnStage::PreparingToSummonPillars);
            }
            DragonRespawnStage::PreparingToSummonPillars => {
                if time < 100 {
                    if time == 0 || time == 50 || time == 51 || time == 52 || time >= 95 {
                        world.sync_world_event(
                            WorldEvent::AnimationDragonSummonRoar,
                            origin_target,
                            0,
                        );
                    }
                } else {
                    self.set_respawn_stage(world, DragonRespawnStage::SummoningPillars);
                }
            }
            DragonRespawnStage::SummoningPillars => {
                let flag = time % 40 == 0;
                let flag1 = time % 40 == 39;
                if flag || flag1 {
                    let spikes = Self::get_spikes();
                    let j = (time / 40) as usize;
                    if j < spikes.len() {
                        let spike = &spikes[j];
                        if flag {
                            let spike_target =
                                BlockPos::new(spike.center_x, spike.height + 1, spike.center_z);
                            for crystal in &live_crystals {
                                crystal.set_beam_target(Some(spike_target));
                            }
                        } else {
                            for dx in -10i32..=10 {
                                for dy in -10i32..=10 {
                                    for dz in -10i32..=10 {
                                        let pos = BlockPos::new(
                                            spike.center_x + dx,
                                            spike.height + dy,
                                            spike.center_z + dz,
                                        );
                                        let block = world.get_block(&pos);
                                        if block != &Block::BEDROCK
                                            && block != &Block::OBSIDIAN
                                            && block != &Block::AIR
                                        {
                                            world.set_block_state(
                                                &pos,
                                                Block::AIR.default_state.id,
                                                BlockFlags::NOTIFY_ALL,
                                            );
                                        }
                                    }
                                }
                            }
                            world.explode(
                                Vector3::new(
                                    spike.center_x as f64 + 0.5,
                                    spike.height as f64,
                                    spike.center_z as f64 + 0.5,
                                ),
                                5.0,
                                crate::world::ExplosionInteraction::Block,
                            );

                            Self::regenerate_spike(world, spike);
                        }
                    } else if flag {
                        self.set_respawn_stage(world, DragonRespawnStage::SummoningDragon);
                    }
                }
            }
            DragonRespawnStage::SummoningDragon => {
                if time >= 100 {
                    self.set_respawn_stage(world, DragonRespawnStage::End);
                    self.reset_spike_crystals(world);
                    for crystal in &live_crystals {
                        crystal.set_beam_target(None);
                        let pos = crystal.get_entity().pos.load();
                        world.explode(pos, 6.0, crate::world::ExplosionInteraction::None);
                        crystal.get_entity().remove();
                    }
                } else if time >= 80 {
                    world.sync_world_event(WorldEvent::AnimationDragonSummonRoar, origin_target, 0);
                } else if time == 0 {
                    let spikes = Self::get_spikes();
                    for spike in &spikes {
                        let (min_x, min_y, min_z, max_x, max_y, max_z) = spike.top_bounding_box();
                        for entity in entities.iter() {
                            if entity.get_entity().entity_type == &EntityType::END_CRYSTAL {
                                let p = entity.get_entity().pos.load();
                                if p.x >= min_x
                                    && p.x <= max_x
                                    && p.y >= min_y
                                    && p.y <= max_y
                                    && p.z >= min_z
                                    && p.z <= max_z
                                    && let Some(crystal) =
                                        entity.cast_any().downcast_ref::<EndCrystalEntity>()
                                {
                                    crystal.set_beam_target(Some(origin_target));
                                }
                            }
                        }
                    }
                } else if time < 5 {
                    world.sync_world_event(WorldEvent::AnimationDragonSummonRoar, origin_target, 0);
                }
            }
            DragonRespawnStage::End => {}
        }
    }

    fn regenerate_spike(world: &Arc<World>, spike: &EndSpike) {
        let bedrock_pos = BlockPos::new(spike.center_x, spike.height, spike.center_z);
        world.set_block_state(
            &bedrock_pos,
            Block::BEDROCK.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );

        let fire_pos = BlockPos::new(spike.center_x, spike.height + 1, spike.center_z);
        world.set_block_state(
            &fire_pos,
            Block::FIRE.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );

        if spike.guarded {
            for dy in 0i32..=3 {
                for dx in -2i32..=2 {
                    for dz in -2i32..=2 {
                        let x_wall = dx.abs() == 2;
                        let z_wall = dz.abs() == 2;
                        let on_top = dy == 3;
                        if !x_wall && !z_wall && !on_top {
                            continue;
                        }
                        let x_edge = x_wall || on_top;
                        let z_edge = z_wall || on_top;

                        let mut props =
                            pumpkin_data::block_properties::OakFenceLikeProperties::default(
                                &Block::IRON_BARS,
                            );
                        props.north = x_edge && dz != 2;
                        props.south = x_edge && dz != -2;
                        props.west = z_edge && dx != 2;
                        props.east = z_edge && dx != -2;

                        let bar_state = props.to_state_id(&Block::IRON_BARS);
                        world.set_block_state(
                            &BlockPos::new(
                                spike.center_x + dx,
                                spike.height + dy,
                                spike.center_z + dz,
                            ),
                            bar_state,
                            BlockFlags::NOTIFY_ALL,
                        );
                    }
                }
            }
        }

        let entity = Entity::new(
            world.clone(),
            Vector3::new(
                spike.center_x as f64 + 0.5,
                (spike.height + 1) as f64,
                spike.center_z as f64 + 0.5,
            ),
            &EntityType::END_CRYSTAL,
        );
        let crystal = Arc::new(EndCrystalEntity::new(entity));
        crystal.set_show_bottom(true);
        crystal.set_invulnerable(false);
        world.spawn_entity_non_save(crystal);
    }

    pub fn reset_spike_crystals(&self, world: &Arc<World>) {
        let spikes = Self::get_spikes();
        let entities = world.entities.load();

        for spike in &spikes {
            let (min_x, min_y, min_z, max_x, max_y, max_z) = spike.top_bounding_box();
            for entity in entities.iter() {
                if entity.get_entity().entity_type == &EntityType::END_CRYSTAL {
                    let p = entity.get_entity().pos.load();
                    if p.x >= min_x
                        && p.x <= max_x
                        && p.y >= min_y
                        && p.y <= max_y
                        && p.z >= min_z
                        && p.z <= max_z
                        && let Some(crystal) = entity.cast_any().downcast_ref::<EndCrystalEntity>()
                    {
                        crystal.set_invulnerable(false);
                        crystal.set_beam_target(None);
                    }
                }
            }
        }
    }

    // ── Gateway spawning ──────────────────────────────────────────────────────

    pub fn remove_all_gateways(&mut self) {
        self.gateways.clear();
    }

    pub fn spawn_new_gateway(&mut self, world: &Arc<World>) {
        use rand::seq::SliceRandom;

        if self.gateways.is_empty() {
            let mut new_gateways: Vec<i32> = (0..(GATEWAY_COUNT as i32)).collect();
            let mut rng = rand::rng();
            new_gateways.shuffle(&mut rng);
            self.gateways = new_gateways;
        }

        if let Some(gateway) = self.gateways.pop() {
            let angle =
                2.0 * (-std::f64::consts::PI + (std::f64::consts::PI / 20.0) * gateway as f64);
            let x = (GATEWAY_DISTANCE * angle.cos()).floor() as i32;
            let z = (GATEWAY_DISTANCE * angle.sin()).floor() as i32;
            let pos = BlockPos::new(x, GATEWAY_Y, z);
            info!("Spawning new end gateway at pos {:?}", pos);
            Self::spawn_new_gateway_at(world, pos);
        }
    }

    pub fn spawn_new_gateway_at(world: &Arc<World>, pos: BlockPos) {
        world.sync_world_event(WorldEvent::AnimationEndGatewaySpawn, pos, 0);

        for dx in -1i32..=1 {
            for dy in -2i32..=2 {
                for dz in -1i32..=1 {
                    let same_x = dx == 0;
                    let same_y = dy == 0;
                    let same_z = dz == 0;
                    let end = dy.abs() == 2;
                    let target = BlockPos::new(pos.0.x + dx, pos.0.y + dy, pos.0.z + dz);

                    if same_x && same_y && same_z {
                        world.set_block_state(
                            &target,
                            Block::END_GATEWAY.default_state.id,
                            BlockFlags::NOTIFY_LISTENERS,
                        );
                    } else if same_y {
                        world.set_block_state(
                            &target,
                            Block::AIR.default_state.id,
                            BlockFlags::NOTIFY_LISTENERS,
                        );
                    } else if (end && same_x && same_z) || ((same_x || same_z) && !end) {
                        world.set_block_state(
                            &target,
                            Block::BEDROCK.default_state.id,
                            BlockFlags::NOTIFY_LISTENERS,
                        );
                    } else {
                        world.set_block_state(
                            &target,
                            Block::AIR.default_state.id,
                            BlockFlags::NOTIFY_LISTENERS,
                        );
                    }
                }
            }
        }
    }

    // ── Crystal spawning ──────────────────────────────────────────────────────

    pub fn spawn_crystals(&mut self, world: &Arc<World>) {
        let spikes = Self::get_spikes();
        let entities = world.entities.load();
        for spike in &spikes {
            let bb = spike.top_bounding_box();
            let already_has_crystal = entities.iter().any(|e| {
                if e.get_entity().entity_type == &EntityType::END_CRYSTAL {
                    let pos = e.get_entity().pos.load();
                    pos.x >= bb.0
                        && pos.x <= bb.3
                        && pos.y >= bb.1
                        && pos.y <= bb.4
                        && pos.z >= bb.2
                        && pos.z <= bb.5
                } else {
                    false
                }
            });

            if already_has_crystal {
                continue;
            }

            let crystal_entity = Entity::new(
                world.clone(),
                Vector3::new(
                    spike.center_x as f64 + 0.5,
                    (spike.height + 1) as f64,
                    spike.center_z as f64 + 0.5,
                ),
                &EntityType::END_CRYSTAL,
            );
            let crystal = Arc::new(EndCrystalEntity::new(crystal_entity));
            crystal.set_show_bottom(spike.guarded);
            crystal.set_invulnerable(false);
            world.spawn_entity(crystal);
        }
    }

    // ── Exit portal ───────────────────────────────────────────────────────────

    pub fn spawn_exit_portal(&mut self, world: &Arc<World>, active: bool) {
        if self.exit_portal_location.is_none() {
            let mut portal_y = 65;
            for y in (50..=100).rev() {
                let b = world.get_block(&BlockPos::new(self.origin.0.x, y, self.origin.0.z));
                if b == &Block::END_STONE || b == &Block::BEDROCK {
                    portal_y = y;
                    break;
                }
            }

            self.exit_portal_location =
                Some(BlockPos::new(self.origin.0.x, portal_y, self.origin.0.z));
        }

        if let Some(loc) = self.exit_portal_location {
            super::end_podium::place(world, loc, active);
        }
    }

    // ── Boss-bar management ───────────────────────────────────────────────────

    fn make_bossbar(&self) -> Bossbar {
        Bossbar {
            uuid: self.bossbar_uuid,
            title: TextComponent::translate(
                "entity.minecraft.ender_dragon",
                Vec::<TextComponent>::new(),
            ),
            health: 1.0,
            color: BossbarColor::Pink,
            division: BossbarDivisions::NoDivision,
            flags: BossbarFlags::DRAGON_BAR | BossbarFlags::CREATE_FOG | BossbarFlags::DARKEN_SKY,
        }
    }

    fn update_bossbar_health(&self, world: &Arc<World>, health: f32) {
        for player in world.players.load().iter() {
            if self.bossbar_players.contains(&player.gameprofile.id) {
                player.update_bossbar_health(&self.bossbar_uuid, health);
            }
        }
    }

    fn update_bossbar_title(&self, world: &Arc<World>, title: &TextComponent) {
        for player in world.players.load().iter() {
            if self.bossbar_players.contains(&player.gameprofile.id) {
                player.update_bossbar_title(&self.bossbar_uuid, title.clone());
            }
        }
    }

    fn remove_all_bossbar(&mut self, world: &Arc<World>) {
        for player in world.players.load().iter() {
            if self.bossbar_players.contains(&player.gameprofile.id) {
                player.remove_bossbar(self.bossbar_uuid);
            }
        }
        self.bossbar_players.clear();
    }

    fn update_players(&mut self, world: &Arc<World>) {
        let players = world.players.load();
        let target_y = self.origin.0.y as f64 + DRAGON_SPAWN_Y as f64;
        let origin_x = self.origin.0.x as f64;
        let origin_z = self.origin.0.z as f64;

        let current: Vec<Uuid> = players
            .iter()
            .filter(|p| {
                let pos = p.living_entity.entity.pos.load();
                let dx = pos.x - origin_x;
                let dy = pos.y - target_y;
                let dz = pos.z - origin_z;
                dx * dx + dy * dy + dz * dz < ARENA_RADIUS * ARENA_RADIUS
            })
            .map(|p| p.gameprofile.id)
            .collect();

        // Add newly-in-range players
        for &uid in &current {
            if !self.bossbar_players.contains(&uid) {
                if !self.dragon_killed
                    && let Some(p) = players.iter().find(|p| p.gameprofile.id == uid)
                {
                    p.send_bossbar(&self.make_bossbar());
                }
                self.bossbar_players.push(uid);
            }
        }

        // Remove out-of-range players
        let to_remove: Vec<Uuid> = self
            .bossbar_players
            .iter()
            .filter(|uid| !current.contains(uid))
            .copied()
            .collect();

        for uid in &to_remove {
            if let Some(player) = players.iter().find(|player| &player.gameprofile.id == uid) {
                player.remove_bossbar(self.bossbar_uuid);
            }
            self.bossbar_players.retain(|u| u != uid);
        }
    }

    // ── Public queries ────────────────────────────────────────────────────────

    #[must_use]
    pub const fn alive_crystals(&self) -> i32 {
        self.alive_crystals
    }

    #[must_use]
    pub const fn has_previously_killed_dragon(&self) -> bool {
        self.previously_killed
    }

    #[must_use]
    pub const fn is_respawning(&self) -> bool {
        self.respawn_stage.is_some()
    }

    #[must_use]
    pub const fn dragon_uuid(&self) -> Option<Uuid> {
        self.dragon_uuid
    }
}
