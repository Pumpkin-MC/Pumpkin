//! End Dragon fight manager. Handles the dragon lifecycle, end crystal
//! spawning, boss bar, and exit portal.

use std::sync::Arc;

use tracing::info;
use uuid::Uuid;

use pumpkin_data::{Block, entity::EntityType};
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

const MAX_TICKS_BEFORE_DRAGON_RESPAWN: i32 = 1200;
const CRYSTAL_SCAN_INTERVAL: i32 = 100;
const PLAYER_SCAN_INTERVAL: i32 = 20;
const DRAGON_SPAWN_Y: f64 = 128.0;
const ARENA_RADIUS: f64 = 192.0;

pub struct DragonFight {
    pub dragon_killed: bool,
    pub previously_killed: bool,
    pub needs_state_scanning: bool,
    pub dragon_uuid: Option<Uuid>,
    pub portal_location: Option<BlockPos>,
    pub crystals_alive: u32,

    ticks_since_dragon_seen: i32,
    ticks_since_crystals_scanned: i32,
    ticks_since_last_player_scan: i32,
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
        Self {
            dragon_killed: false,
            previously_killed: false,
            needs_state_scanning: true,
            dragon_uuid: None,
            portal_location: None,
            crystals_alive: 0,
            ticks_since_dragon_seen: 0,
            ticks_since_crystals_scanned: 0,
            // Start over the threshold so the scan runs on the first tick with players
            ticks_since_last_player_scan: PLAYER_SCAN_INTERVAL + 1,
            bossbar_uuid: Uuid::new_v4(),
            bossbar_players: Vec::new(),
        }
    }

    pub async fn tick(&mut self, world: &Arc<World>) {
        self.ticks_since_last_player_scan += 1;
        if self.ticks_since_last_player_scan >= PLAYER_SCAN_INTERVAL {
            self.update_players(world).await;
            self.ticks_since_last_player_scan = 0;
        }

        // Nothing to do without players nearby
        if self.bossbar_players.is_empty() {
            return;
        }

        if self.needs_state_scanning {
            self.scan_state(world).await;
            self.needs_state_scanning = false;
        }

        if !self.dragon_killed {
            self.ticks_since_dragon_seen += 1;
            if self.dragon_uuid.is_none()
                || self.ticks_since_dragon_seen >= MAX_TICKS_BEFORE_DRAGON_RESPAWN
            {
                self.find_or_create_dragon(world).await;
                self.ticks_since_dragon_seen = 0;
            }

            self.ticks_since_crystals_scanned += 1;
            if self.ticks_since_crystals_scanned >= CRYSTAL_SCAN_INTERVAL {
                self.update_crystal_count(world);
                self.ticks_since_crystals_scanned = 0;
            }
        }
    }
}

impl DragonFight {
    /// Runs once on the first tick that has players. Figures out whether this
    /// is a fresh world or a resumed one and brings the podium/dragon into a
    /// consistent state.
    async fn scan_state(&mut self, world: &Arc<World>) {
        info!("Scanning End fight state...");

        let has_active_portal = self.has_active_exit_portal(world).await;

        if has_active_portal {
            info!("Exit portal found – dragon has been killed previously.");
            self.previously_killed = true;
        } else {
            info!("No exit portal – fight is fresh or in progress.");
            self.previously_killed = false;
            // Place the inactive podium if it hasn't been set yet
            if self.portal_location.is_none() {
                self.spawn_exit_portal(world, false).await;
            }
            // Spawn crystals on the spike caps
            self.spawn_crystals(world).await;
        }

        // Check for a live dragon entity.
        let existing_dragon = {
            let entities = world.entities.load();
            entities
                .iter()
                .find(|e| e.get_entity().entity_type == &EntityType::ENDER_DRAGON)
                .map(|e| e.get_entity().entity_uuid)
        };

        if let Some(uuid) = existing_dragon {
            if has_active_portal {
                // The portal is active but a dragon entity still exists... so clean it up
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
            } else {
                info!("Found existing dragon entity.");
                self.dragon_uuid = Some(uuid);
                self.dragon_killed = false;
            }
        } else {
            self.dragon_killed = true;
        }

        // If the world looks fresh but the dragon appears dead, reset so one gets spawned
        if !self.previously_killed && self.dragon_killed {
            self.dragon_killed = false;
        }
    }

    /// Returns true if an active `END_PORTAL` block exists within an 8-chunk
    /// radius of the origin. Samples one column per chunk for performance
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
}

impl DragonFight {
    async fn find_or_create_dragon(&mut self, world: &Arc<World>) {
        let uuid = {
            let entities = world.entities.load();
            entities
                .iter()
                .find(|e| e.get_entity().entity_type == &EntityType::ENDER_DRAGON)
                .map(|e| e.get_entity().entity_uuid)
        };

        if let Some(u) = uuid {
            info!("Re-acquired existing dragon.");
            self.dragon_uuid = Some(u);
        } else {
            info!("No dragon found \u{2013} spawning one.");
            self.create_new_dragon(world).await;
        }
    }

    async fn create_new_dragon(&mut self, world: &Arc<World>) {
        let entity = Entity::new(
            world.clone(),
            Vector3::new(0.5, DRAGON_SPAWN_Y, 0.5),
            &EntityType::ENDER_DRAGON,
        );
        let uuid = entity.entity_uuid;
        world.spawn_entity(Arc::new(entity)).await;
        self.dragon_uuid = Some(uuid);
        info!("Spawned ender dragon.");
    }

    fn update_crystal_count(&mut self, world: &Arc<World>) {
        self.crystals_alive = world
            .entities
            .load()
            .iter()
            .filter(|e| e.get_entity().entity_type == &EntityType::END_CRYSTAL)
            .count() as u32;
    }

    /// Called when the dragon dies. Activates the exit portal, places the
    /// dragon egg on a first kill, and hides the boss bar.
    pub async fn set_dragon_killed(&mut self, world: &Arc<World>, killed_uuid: Uuid) {
        if Some(killed_uuid) != self.dragon_uuid {
            return;
        }

        self.update_bossbar_health(world, 0.0).await;
        self.remove_all_bossbar(world).await;

        self.spawn_exit_portal(world, true).await;

        if !self.previously_killed
            && let Some(loc) = self.portal_location
        {
            // The bedrock pillar is 4 blocks tall, so the egg sits at podium_y + 4
            let egg_pos = BlockPos::new(loc.0.x, loc.0.y + 4, loc.0.z);
            world
                .set_block_state(
                    &egg_pos,
                    Block::DRAGON_EGG.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
        }

        self.previously_killed = true;
        self.dragon_killed = true;
        self.dragon_uuid = None;
    }
}

impl DragonFight {
    /// Spawns end crystals on each obsidian spike. Scans downward from y=115
    /// for the bedrock cap placed by the spike generator and places the crystal
    /// one block above it. Falls back to y=78 if the chunk hasn't loaded yet.
    pub async fn spawn_crystals(&mut self, world: &Arc<World>) {
        // Don't re-spawn if crystals are already present (resumed world).
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

            let mut crystal_y: Option<i32> = None;
            for y in (70..=115i32).rev() {
                if world.get_block(&BlockPos::new(cx, y, cz)).await == &Block::BEDROCK {
                    crystal_y = Some(y + 1);
                    break;
                }
            }
            let y = crystal_y.unwrap_or(78);

            let entity = Entity::new(
                world.clone(),
                Vector3::new(cx as f64 + 0.5, y as f64, cz as f64 + 0.5),
                &EntityType::END_CRYSTAL,
            );
            let crystal = Arc::new(EndCrystalEntity::new(entity));
            crystal.set_show_bottom(true).await;
            world.spawn_entity(crystal).await;
        }
        info!("Spawned end crystals on spike tops.");
    }
}

impl DragonFight {
    /// Places the exit podium centred at column (0, 0). Pass `active = true`
    /// to fill the portal disc with `END_PORTAL` blocks after the dragon dies.
    pub async fn spawn_exit_portal(&mut self, world: &Arc<World>, active: bool) {
        if self.portal_location.is_none() {
            let top_y = world.get_top_block(Vector2::new(0, 0)).await;
            let mut portal_y = top_y;

            // Walk down past any bedrock already at the centre (e.g. spike cap).
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
}

impl DragonFight {
    fn make_bossbar(&self) -> Bossbar {
        Bossbar {
            uuid: self.bossbar_uuid,
            title: TextComponent::translate("entity.minecraft.ender_dragon", []),
            health: 1.0,
            color: BossbarColor::Pink,
            division: BossbarDivisions::NoDivision,
            flags: BossbarFlags::DragonBar,
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

    /// Syncs the boss bar recipient list
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
}
