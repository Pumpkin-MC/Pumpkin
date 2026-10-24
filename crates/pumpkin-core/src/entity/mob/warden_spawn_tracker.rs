use std::sync::Arc;

use pumpkin_data::entity::EntityType;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};

use crate::entity::{EntityBase, player::Player};
use crate::world::World;

pub const MAX_WARNING_LEVEL: i32 = 4;
const PLAYER_SEARCH_RADIUS: f64 = 16.0;
const WARNING_CHECK_DIAMETER: f64 = 48.0;
const DECREASE_WARNING_LEVEL_EVERY_INTERVAL: i32 = 12000;
const WARNING_LEVEL_INCREASE_COOLDOWN: i32 = 200;

#[derive(Clone, Copy, Default)]
pub struct WardenSpawnTracker {
    ticks_since_last_warning: i32,
    warning_level: i32,
    cooldown_ticks: i32,
}

impl WardenSpawnTracker {
    pub const fn tick(&mut self) {
        if self.ticks_since_last_warning >= DECREASE_WARNING_LEVEL_EVERY_INTERVAL {
            self.set_warning_level(self.warning_level - 1);
            self.ticks_since_last_warning = 0;
        } else {
            self.ticks_since_last_warning += 1;
        }

        if self.cooldown_ticks > 0 {
            self.cooldown_ticks -= 1;
        }
    }

    pub fn try_warn(world: &World, pos: &BlockPos, trigger_player: &Arc<Player>) -> Option<i32> {
        if Self::has_nearby_warden(world, pos) {
            return None;
        }

        let mut players = Self::get_nearby_players(world, pos);
        if !players
            .iter()
            .any(|player| player.entity_id() == trigger_player.entity_id())
        {
            players.push(trigger_player.clone());
        }

        let trackers: Vec<Self> = players.iter().map(|player| *Self::lock(player)).collect();
        if trackers.iter().any(Self::on_cooldown) {
            return None;
        }

        let mut highest = trackers.into_iter().reduce(|best, tracker| {
            if tracker.warning_level >= best.warning_level {
                tracker
            } else {
                best
            }
        })?;
        highest.increase_warning_level();
        for player in &players {
            *Self::lock(player) = highest;
        }
        Some(highest.warning_level)
    }

    fn lock(player: &Player) -> std::sync::MutexGuard<'_, Self> {
        player
            .warden_spawn_tracker
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    const fn on_cooldown(&self) -> bool {
        self.cooldown_ticks > 0
    }

    fn has_nearby_warden(world: &World, pos: &BlockPos) -> bool {
        let center = pos.to_centered_f64();
        let half = WARNING_CHECK_DIAMETER / 2.0;
        let area = BoundingBox::new(
            Vector3::new(center.x - half, center.y - half, center.z - half),
            Vector3::new(center.x + half, center.y + half, center.z + half),
        );
        world
            .get_entities_at_box(&area)
            .iter()
            .any(|entity| entity.get_entity().entity_type.id == EntityType::WARDEN.id)
    }

    fn get_nearby_players(world: &World, pos: &BlockPos) -> Vec<Arc<Player>> {
        world
            .get_nearby_players(pos.to_centered_f64(), PLAYER_SEARCH_RADIUS)
            .into_iter()
            .filter(|player| !player.is_spectator() && player.get_entity().is_alive())
            .collect()
    }

    const fn increase_warning_level(&mut self) {
        if !self.on_cooldown() {
            self.ticks_since_last_warning = 0;
            self.cooldown_ticks = WARNING_LEVEL_INCREASE_COOLDOWN;
            self.set_warning_level(self.warning_level + 1);
        }
    }

    const fn set_warning_level(&mut self, warning_level: i32) {
        self.warning_level = if warning_level < 0 {
            0
        } else if warning_level > MAX_WARNING_LEVEL {
            MAX_WARNING_LEVEL
        } else {
            warning_level
        };
    }

    #[must_use]
    pub fn from_nbt(nbt: &NbtCompound) -> Self {
        Self {
            ticks_since_last_warning: nbt.get_int("ticks_since_last_warning").unwrap_or(0).max(0),
            warning_level: nbt.get_int("warning_level").unwrap_or(0).max(0),
            cooldown_ticks: nbt.get_int("cooldown_ticks").unwrap_or(0).max(0),
        }
    }

    #[must_use]
    pub fn to_nbt(&self) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        nbt.put_int("ticks_since_last_warning", self.ticks_since_last_warning);
        nbt.put_int("warning_level", self.warning_level);
        nbt.put_int("cooldown_ticks", self.cooldown_ticks);
        nbt
    }
}
