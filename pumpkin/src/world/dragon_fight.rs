//! End Dragon fight manager. Handles the dragon lifecycle, end crystal
//! spawning, boss bar, gateway spawning, respawn sequence, and exit portal.
//!
//! Matches vanilla `EnderDragonFight` behaviour as closely as `PumpkinMC`'s
//! current API allows.

use std::sync::Arc;

use tokio::sync::Mutex;
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
use crate::entity::{Entity, decoration::end_crystal::EndCrystalEntity};

// ── Constants (match vanilla exactly) ────────────────────────────────────────

const MAX_TICKS_BEFORE_DRAGON_RESPAWN: i32 = 1200;
const CRYSTAL_SCAN_INTERVAL: i32 = 100;
const PLAYER_SCAN_INTERVAL: i32 = 20;
const DRAGON_SPAWN_Y: f64 = 128.0;
const ARENA_RADIUS: f64 = 192.0;

/// Number of end gateways that can be unlocked (one per dragon kill after the first).
const GATEWAY_COUNT: usize = 20;
/// Radius at which gateways are placed (blocks from origin).
const GATEWAY_DISTANCE: f64 = 96.0;
/// Y level at which gateways are placed.
const GATEWAY_Y: i32 = 75;

// ── Respawn stage ─────────────────────────────────────────────────────────────

/// Mirrors vanilla `DragonRespawnStage`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragonRespawnStage {
    /// Crystals are charging up (0-99 ticks).
    Start,
    /// Summoning beam is active (100-179 ticks).
    Preparing,
    /// Dragon is being summoned (180-199 ticks).
    End,
}

impl DragonRespawnStage {
    /// Advance through the respawn animation.  Returns `true` when the stage
    /// has completed and the caller should advance to the next one.
    #[must_use]
    pub const fn tick(&self, time: i32) -> bool {
        match self {
            Self::Start => time >= 100,
            Self::Preparing => time >= 80,
            Self::End => time >= 20,
        }
    }
}

// ── DragonFight ───────────────────────────────────────────────────────────────

pub struct DragonFight {
    // ── Persistent state (would be saved to disk in vanilla) ──────────────────
    pub dragon_killed: bool,
    pub previously_killed: bool,
    pub needs_state_scanning: bool,
    pub dragon_uuid: Option<Uuid>,
    pub portal_location: Option<BlockPos>,
    pub crystals_alive: u32,

    /// Gateways not yet unlocked, stored as angle indices 0-19.
    /// Vanilla shuffles these at world creation and pops one per dragon kill.
    pub pending_gateways: Vec<usize>,

    /// Active respawn stage, or `None` when not respawning.
    pub respawn_stage: Option<DragonRespawnStage>,
    /// Ticks elapsed in the current respawn stage.
    pub respawn_time: i32,
    /// UUIDs of the four crystals being used for the respawn ritual.
    pub respawn_crystal_uuids: Vec<Uuid>,

    // ── Transient counters ────────────────────────────────────────────────────
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
        // Build the gateway queue: indices 0-19 in a fixed (non-shuffled for
        // now) order.  Vanilla shuffles by world seed — shuffle here if your
        // server exposes a seed.
        let pending_gateways: Vec<usize> = (0..GATEWAY_COUNT).collect();

        Self {
            dragon_killed: false,
            previously_killed: false,
            needs_state_scanning: true,
            dragon_uuid: None,
            portal_location: None,
            crystals_alive: 0,
            pending_gateways,
            respawn_stage: None,
            respawn_time: 0,
            respawn_crystal_uuids: Vec::new(),
            ticks_since_dragon_seen: 0,
            ticks_since_crystals_scanned: 0,
            // Start above the threshold so the first tick with players triggers a scan.
            ticks_since_last_player_scan: PLAYER_SCAN_INTERVAL + 1,
            bossbar_uuid: Uuid::new_v4(),
            bossbar_players: Vec::new(),
        }
    }

    // ── Main tick ─────────────────────────────────────────────────────────────

    pub async fn tick(fight_mutex: &Mutex<Self>, world: &Arc<World>) {
        let (
            ticks_since_last_player_scan,
            needs_state_scanning,
            respawn_stage,
            dragon_killed,
            dragon_uuid,
        ) = {
            let mut fight = fight_mutex.lock().await;
            fight.ticks_since_last_player_scan += 1;
            (
                fight.ticks_since_last_player_scan,
                fight.needs_state_scanning,
                fight.respawn_stage,
                fight.dragon_killed,
                fight.dragon_uuid,
            )
        };

        // 1. Update boss-bar recipients every 20 ticks.
        if ticks_since_last_player_scan >= PLAYER_SCAN_INTERVAL {
            let mut fight = fight_mutex.lock().await;
            fight.update_players(world).await;
            fight.ticks_since_last_player_scan = 0;
        }

        let is_empty = { fight_mutex.lock().await.bossbar_players.is_empty() };
        // Nothing to do without nearby players.
        if is_empty {
            return;
        }

        // 2. One-time state scan on the first populated tick.
        if needs_state_scanning {
            let mut fight = fight_mutex.lock().await;
            fight.scan_state(world).await;
            fight.needs_state_scanning = false;
        }

        // 3. Respawn sequence (takes priority over normal dragon-missing logic).
        if respawn_stage.is_some() {
            let mut fight = fight_mutex.lock().await;
            fight.tick_respawn(world).await;
            return;
        }

        // 4. Normal fight ticking.
        if !dragon_killed {
            let mut fight = fight_mutex.lock().await;
            fight.ticks_since_dragon_seen += 1;
            if dragon_uuid.is_none()
                || fight.ticks_since_dragon_seen >= MAX_TICKS_BEFORE_DRAGON_RESPAWN
            {
                fight.find_or_create_dragon(world).await;
                fight.ticks_since_dragon_seen = 0;
            }

            fight.ticks_since_crystals_scanned += 1;
            if fight.ticks_since_crystals_scanned >= CRYSTAL_SCAN_INTERVAL {
                fight.update_crystal_count(world);
                fight.ticks_since_crystals_scanned = 0;
            }
        }
    }

    // ── State scanning ────────────────────────────────────────────────────────

    /// Runs once on the first tick with nearby players.  Determines whether
    /// this is a fresh fight or a resumed one and reconciles the entity list.
    async fn scan_state(&mut self, world: &Arc<World>) {
        info!("Scanning End fight state...");

        let has_active_portal = self.has_active_exit_portal(world).await;

        if has_active_portal {
            info!("Exit portal found – dragon has been killed previously.");
            self.previously_killed = true;
        } else {
            info!("No exit portal – fight is fresh or in progress.");
            self.previously_killed = false;
            if self.portal_location.is_none() {
                self.spawn_exit_portal(world, false).await;
            }
            self.spawn_crystals(world).await;
        }

        // Reconcile any live dragon entity.
        let existing = {
            let entities = world.entities.load();
            entities
                .iter()
                .find(|e| e.get_entity().entity_type == &EntityType::ENDER_DRAGON)
                .map(|e| e.get_entity().entity_uuid)
        };

        match existing {
            Some(uuid) if has_active_portal => {
                // Stale dragon after a completed fight — remove it.
                if let Some(e) = world
                    .entities
                    .load()
                    .iter()
                    .find(|e| e.get_entity().entity_uuid == uuid)
                {
                    e.get_entity().remove().await;
                }
                self.dragon_uuid = None;
                self.dragon_killed = true;
            }
            Some(uuid) => {
                info!("Found existing dragon entity {:?}.", uuid);
                self.dragon_uuid = Some(uuid);
                self.dragon_killed = false;
            }
            None => {
                self.dragon_killed = true;
            }
        }

        // Fresh world where the dragon appears dead → force a spawn next tick.
        if !self.previously_killed && self.dragon_killed {
            self.dragon_killed = false;
        }
    }

    /// Checks whether a `END_PORTAL` block exists within an 8-chunk radius
    /// of the origin, matching vanilla's `hasActiveExitPortal`.
    async fn has_active_exit_portal(&self, world: &Arc<World>) -> bool {
        for cx in -8i32..=8 {
            for cz in -8i32..=8 {
                let bx = cx * 16;
                let bz = cz * 16;
                for y in 30i32..=80 {
                    if world.get_block(&BlockPos::new(bx, y, bz)).await == &Block::END_PORTAL {
                        return true;
                    }
                }
            }
        }
        false
    }

    // ── Dragon lifecycle ──────────────────────────────────────────────────────

    async fn find_or_create_dragon(&mut self, world: &Arc<World>) {
        let uuid = {
            let entities = world.entities.load();
            entities
                .iter()
                .find(|e| e.get_entity().entity_type == &EntityType::ENDER_DRAGON)
                .map(|e| e.get_entity().entity_uuid)
        };

        if let Some(u) = uuid {
            debug!("Re-acquired existing dragon {:?}.", u);
            self.dragon_uuid = Some(u);
            self.ticks_since_dragon_seen = 0;
        } else {
            debug!("No dragon found – spawning one.");
            self.create_new_dragon(world).await;
        }
    }

    async fn create_new_dragon(&mut self, world: &Arc<World>) {
        let uuid = Uuid::new_v4();
        let position = Vector3::new(0.5, DRAGON_SPAWN_Y, 0.5);
        let dragon =
            crate::entity::r#type::from_type(&EntityType::ENDER_DRAGON, position, world, uuid)
                .await;

        world.spawn_entity(dragon).await;
        self.dragon_uuid = Some(uuid);
        info!("Spawned ender dragon {:?}.", uuid);
    }

    /// Called every tick while the dragon is alive.  Updates the boss-bar
    /// health fraction, matching vanilla `EnderDragonFight.updateDragon`.
    pub async fn update_dragon(&mut self, world: &Arc<World>, health: f32, max_health: f32) {
        self.ticks_since_dragon_seen = 0;
        let fraction = if max_health > 0.0 {
            (health / max_health).clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.update_bossbar_health(world, fraction).await;

        // Sync fight origin to the dragon so its pathfinding nodes are correctly placed.
        if let Some(loc) = self.portal_location
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
            dragon.set_fight_origin(loc).await;
        }
    }

    /// Called by the dragon entity when it dies.  Activates the portal, places
    /// the egg on a first kill, spawns a gateway, and hides the boss bar.
    /// Matches vanilla `EnderDragonFight.setDragonKilled`.
    pub async fn set_dragon_killed(&mut self, world: &Arc<World>, killed_uuid: Uuid) {
        if Some(killed_uuid) != self.dragon_uuid {
            return;
        }

        self.update_bossbar_health(world, 0.0).await;
        self.remove_all_bossbar(world).await;

        // Activate the exit portal.
        self.spawn_exit_portal(world, true).await;

        // Place the dragon egg on the first kill.
        if !self.previously_killed
            && let Some(loc) = self.portal_location
        {
            let egg_pos = BlockPos::new(loc.0.x, loc.0.y + 4, loc.0.z);
            world
                .set_block_state(
                    &egg_pos,
                    Block::DRAGON_EGG.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        }

        // Spawn a new end gateway.
        self.spawn_new_gateway(world).await;

        self.previously_killed = true;
        self.dragon_killed = true;
        self.dragon_uuid = None;
    }

    fn update_crystal_count(&mut self, world: &Arc<World>) {
        self.crystals_alive = world
            .entities
            .load()
            .iter()
            .filter(|e| e.get_entity().entity_type == &EntityType::END_CRYSTAL)
            .count() as u32;
        debug!("Found {} end crystals still alive.", self.crystals_alive);
    }

    // ── Crystal destruction ───────────────────────────────────────────────────

    /// Called when an end crystal is destroyed.  If a respawn is in progress
    /// and this was one of the ritual crystals, the respawn is aborted.
    /// Matches vanilla `EnderDragonFight.onCrystalDestroyed`.
    pub async fn on_crystal_destroyed(&mut self, world: &Arc<World>, crystal_uuid: Uuid) {
        if self.respawn_stage.is_some() && self.respawn_crystal_uuids.contains(&crystal_uuid) {
            self.abort_respawn(world).await;
        } else {
            self.update_crystal_count(world);
            // The dragon entity itself handles the visual beam-break logic;
            // we just keep the count accurate here.
        }
    }

    // ── Respawn sequence ──────────────────────────────────────────────────────

    /// Attempt to begin a respawn.  Requires four end crystals placed on the
    /// cardinal sides of the portal, exactly as in vanilla `tryRespawn`.
    pub async fn try_respawn(&mut self, world: &Arc<World>) {
        if !self.dragon_killed || self.respawn_stage.is_some() {
            return;
        }

        // Ensure we know where the portal is.
        if self.portal_location.is_none() {
            info!("Tried to respawn but no portal location – placing one.");
            self.spawn_exit_portal(world, true).await;
        }

        let Some(portal_loc) = self.portal_location else {
            return;
        };

        // Scan the four cardinal directions one block above the portal centre
        // for end crystals, matching vanilla's cardinal check.
        let centre = BlockPos::new(portal_loc.0.x, portal_loc.0.y + 1, portal_loc.0.z);
        let offsets: [(i32, i32); 4] = [(3, 0), (-3, 0), (0, 3), (0, -3)];
        let entities = world.entities.load();

        let mut ritual_uuids = Vec::new();
        for (dx, dz) in offsets {
            let check = BlockPos::new(centre.0.x + dx, centre.0.y, centre.0.z + dz);
            // Find a crystal near this position (within 2 blocks).
            let found = entities.iter().find(|e| {
                if e.get_entity().entity_type != &EntityType::END_CRYSTAL {
                    return false;
                }
                let p = e.get_entity().pos.load();
                let cx = check.0.x as f64 + 0.5;
                let cz = check.0.z as f64 + 0.5;
                (p.x - cx).abs() < 2.0 && (p.z - cz).abs() < 2.0
            });
            if let Some(e) = found {
                ritual_uuids.push(e.get_entity().entity_uuid);
            } else {
                debug!("Respawn attempt failed – missing crystal near {:?}.", check);
                return;
            }
        }

        debug!("Found all four ritual crystals – beginning respawn.");
        self.begin_respawn(world, ritual_uuids).await;
    }

    async fn begin_respawn(&mut self, world: &Arc<World>, crystal_uuids: Vec<Uuid>) {
        // Tear down the active portal (replace END_PORTAL/BEDROCK with END_STONE)
        // so the podium resets, matching vanilla.
        if let Some(loc) = self.portal_location {
            self.clear_portal_blocks(world, loc).await;
        }

        self.respawn_stage = Some(DragonRespawnStage::Start);
        self.respawn_time = 0;
        self.respawn_crystal_uuids = crystal_uuids;
        self.spawn_exit_portal(world, false).await;
    }

    /// Replace the bedrock/portal blocks of the current podium with end-stone,
    /// matching the vanilla portal-reset done during respawn.
    async fn clear_portal_blocks(&self, world: &Arc<World>, loc: BlockPos) {
        // The podium is 7×6×7 centred on loc; just scan a generous volume.
        for dy in -1i32..=5 {
            for dx in -4i32..=4 {
                for dz in -4i32..=4 {
                    let pos = BlockPos::new(loc.0.x + dx, loc.0.y + dy, loc.0.z + dz);
                    let block = world.get_block(&pos).await;
                    if block == &Block::BEDROCK || block == &Block::END_PORTAL {
                        world
                            .set_block_state(
                                &pos,
                                Block::END_STONE.default_state.id,
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await;
                    }
                }
            }
        }
    }

    async fn abort_respawn(&mut self, world: &Arc<World>) {
        debug!("Aborting dragon respawn sequence.");
        self.respawn_stage = None;
        self.respawn_time = 0;
        self.respawn_crystal_uuids.clear();
        // Re-activate the portal so the world remains in a valid state.
        self.spawn_exit_portal(world, true).await;
    }

    /// Drive the respawn animation forward by one tick.
    async fn tick_respawn(&mut self, world: &Arc<World>) {
        let Some(stage) = self.respawn_stage else {
            return;
        };

        // Verify all ritual crystals still exist.
        {
            let entities = world.entities.load();
            let all_alive = self
                .respawn_crystal_uuids
                .iter()
                .all(|uid| entities.iter().any(|e| e.get_entity().entity_uuid == *uid));
            if !all_alive {
                self.abort_respawn(world).await;
                return;
            }
        }

        self.respawn_time += 1;

        if stage.tick(self.respawn_time) {
            match stage {
                DragonRespawnStage::Start => {
                    self.respawn_stage = Some(DragonRespawnStage::Preparing);
                    self.respawn_time = 0;
                }
                DragonRespawnStage::Preparing => {
                    self.respawn_stage = Some(DragonRespawnStage::End);
                    self.respawn_time = 0;
                }
                DragonRespawnStage::End => {
                    // Spawn the new dragon.
                    self.respawn_stage = None;
                    self.respawn_time = 0;
                    self.respawn_crystal_uuids.clear();
                    self.dragon_killed = false;
                    self.create_new_dragon(world).await;
                }
            }
        }
    }

    // ── Gateway spawning ──────────────────────────────────────────────────────

    /// Spawn the next end gateway, consuming one index from `pending_gateways`.
    /// Matches vanilla `spawnNewGateway`.
    async fn spawn_new_gateway(&mut self, world: &Arc<World>) {
        let Some(idx) = self.pending_gateways.pop() else {
            return;
        };

        let angle = 2.0 * (-std::f64::consts::PI + std::f64::consts::PI * 0.1 * idx as f64);
        let x = (GATEWAY_DISTANCE * angle.cos()).floor() as i32;
        let z = (GATEWAY_DISTANCE * angle.sin()).floor() as i32;
        let pos = BlockPos::new(x, GATEWAY_Y, z);

        world
            .sync_world_event(WorldEvent::AnimationEndGatewaySpawn, pos, 0)
            .await;
        info!("Spawned end gateway #{} at {:?}.", idx, pos);
    }

    // ── Crystal spawning ──────────────────────────────────────────────────────

    /// Spawn end crystals on the obsidian spike tops.  Skips if any crystal
    /// already exists (resumed world).  Matches vanilla `respawnCrystals`.
    pub async fn spawn_crystals(&mut self, world: &Arc<World>) {
        if world
            .entities
            .load()
            .iter()
            .any(|e| e.get_entity().entity_type == &EntityType::END_CRYSTAL)
        {
            return;
        }

        for i in 0..10usize {
            let angle = 2.0 * (-std::f64::consts::PI + std::f64::consts::PI * 0.1 * i as f64);
            let cx = (42.0f64 * angle.cos()).floor() as i32;
            let cz = (42.0f64 * angle.sin()).floor() as i32;

            // Find the top of the spike by scanning down from y=115 for bedrock.
            let mut crystal_y = 78i32;
            for y in (70..=115i32).rev() {
                if world.get_block(&BlockPos::new(cx, y, cz)).await == &Block::BEDROCK {
                    crystal_y = y + 1;
                    break;
                }
            }

            let entity = Entity::new(
                world.clone(),
                Vector3::new(cx as f64 + 0.5, crystal_y as f64, cz as f64 + 0.5),
                &EntityType::END_CRYSTAL,
            );
            let crystal = Arc::new(EndCrystalEntity::new(entity));
            crystal.set_show_bottom(true).await;
            world.spawn_entity(crystal).await;
        }
        info!("Spawned end crystals on spike tops.");
    }

    // ── Exit portal ───────────────────────────────────────────────────────────

    /// Place (or activate) the exit podium.  `active = true` fills the portal
    /// disc with `END_PORTAL` blocks after the dragon dies.
    pub async fn spawn_exit_portal(&mut self, world: &Arc<World>, active: bool) {
        // Determine location once and cache it.
        if self.portal_location.is_none() {
            let top_y = world.get_top_block(Vector2::new(0, 0)).await;
            let mut portal_y = top_y;
            while portal_y > 63 {
                if world.get_block(&BlockPos::new(0, portal_y, 0)).await != &Block::BEDROCK {
                    break;
                }
                portal_y -= 1;
            }
            portal_y = portal_y.max(world.min_y + 1);
            self.portal_location = Some(BlockPos::new(0, portal_y, 0));
        }

        if let Some(loc) = self.portal_location {
            super::end_podium::place(world, loc, active).await;
        }
    }

    // ── Boss bar ──────────────────────────────────────────────────────────────

    fn make_bossbar(&self) -> Bossbar {
        Bossbar {
            uuid: self.bossbar_uuid,
            title: TextComponent::translate("entity.minecraft.ender_dragon", []),
            health: 1.0,
            color: BossbarColor::Pink,
            division: BossbarDivisions::NoDivision,
            flags: BossbarFlags::DRAGON_BAR,
        }
    }

    async fn update_bossbar_health(&self, world: &Arc<World>, health: f32) {
        for player in world.players.load().iter() {
            if self.bossbar_players.contains(&player.gameprofile.id) {
                player
                    .update_bossbar_health(&self.bossbar_uuid, health)
                    .await;
            }
        }
    }

    async fn remove_all_bossbar(&mut self, world: &Arc<World>) {
        for player in world.players.load().iter() {
            if self.bossbar_players.contains(&player.gameprofile.id) {
                player.remove_bossbar(self.bossbar_uuid).await;
            }
        }
        self.bossbar_players.clear();
    }

    /// Sync the boss-bar recipient list with nearby players.
    /// Matches vanilla `updatePlayers`.
    async fn update_players(&mut self, world: &Arc<World>) {
        let players = world.players.load();

        let current: Vec<Uuid> = players
            .iter()
            .filter(|p| {
                let pos = p.living_entity.entity.pos.load();
                let dx = pos.x;
                let dy = pos.y - DRAGON_SPAWN_Y;
                let dz = pos.z;
                dx * dx + dy * dy + dz * dz < ARENA_RADIUS * ARENA_RADIUS
            })
            .map(|p| p.gameprofile.id)
            .collect();

        // Add newly-in-range players.
        for &uid in &current {
            if !self.bossbar_players.contains(&uid) {
                if !self.dragon_killed
                    && let Some(p) = players.iter().find(|p| p.gameprofile.id == uid)
                {
                    p.send_bossbar(&self.make_bossbar()).await;
                }
                self.bossbar_players.push(uid);
            }
        }

        // Remove out-of-range players.
        let to_remove: Vec<Uuid> = self
            .bossbar_players
            .iter()
            .filter(|uid| !current.contains(uid))
            .copied()
            .collect();

        for uid in &to_remove {
            if let Some(p) = players.iter().find(|p| &p.gameprofile.id == uid) {
                p.remove_bossbar(self.bossbar_uuid).await;
            }
            self.bossbar_players.retain(|u| u != uid);
        }
    }

    // ── Public queries ────────────────────────────────────────────────────────

    #[must_use]
    pub const fn alive_crystals(&self) -> u32 {
        self.crystals_alive
    }

    #[must_use]
    pub const fn has_previously_killed_dragon(&self) -> bool {
        self.previously_killed
    }

    #[must_use]
    pub const fn is_respawning(&self) -> bool {
        self.respawn_stage.is_some()
    }
}
