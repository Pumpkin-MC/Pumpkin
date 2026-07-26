use super::MobEntity;
use pumpkin_data::entity::MobCategory;
use pumpkin_util::GameMode;
use rand::RngExt;
use std::sync::atomic::Ordering::Relaxed;

impl MobEntity {
    /// Vanilla `Mob.checkDespawn` — free mob caps so natural spawning can refresh.
    ///
    /// - Immediate remove beyond category `despawn_distance` (usually 128)
    /// - After 600 ticks far from players (>32), 1/800 chance per tick to despawn
    /// - Named / category-persistent creatures skip random far despawn (still
    ///   immediate-despawn at extreme range is skipped for persistent categories)
    pub async fn check_despawn(&self) -> bool {
        let entity = &self.living_entity.entity;
        let entity_type = entity.entity_type;
        let category = entity_type.category;

        if category == &MobCategory::MISC || !entity_type.mob {
            return false;
        }

        // Vanilla `isPersistenceRequired() || requiresCustomPersistence()`.
        let is_leashed = entity.leashed_to.lock().await.is_some();
        let is_riding = entity.has_vehicle().await;
        if entity.custom_name.load().is_some() || is_leashed || is_riding {
            self.despawn_counter.store(0, Relaxed);
            return false;
        }

        let world = entity.world.load();
        let players = world.players.load();
        if players.is_empty() {
            return false;
        }

        let pos = entity.pos.load();
        let mut nearest_sq = f64::MAX;
        let mut has_non_spectator_player = false;
        for player in players.iter() {
            if player.gamemode.load() == GameMode::Spectator {
                continue;
            }
            has_non_spectator_player = true;
            let d = player.position().squared_distance_to_vec(&pos);
            if d < nearest_sq {
                nearest_sq = d;
            }
        }
        if !has_non_spectator_player {
            return false;
        }

        let despawn_range = category.despawn_distance;
        let immediate_sq = f64::from(despawn_range * despawn_range);
        let soft_range = MobCategory::NO_DESPAWN_DISTANCE;
        let soft_sq = f64::from(soft_range * soft_range);

        // Immediate despawn for non-persistent categories past hard range.
        if !category.is_persistent && nearest_sq > immediate_sq {
            entity.remove().await;
            return true;
        }

        if nearest_sq > soft_sq {
            let counter = self.despawn_counter.fetch_add(1, Relaxed) + 1;
            // Vanilla: after 600 ticks far away, 1/800 chance each tick.
            if counter > 600 && rand::rng().random_range(0..800) == 0 && !category.is_persistent {
                entity.remove().await;
                return true;
            }
        } else {
            self.despawn_counter.store(0, Relaxed);
        }

        false
    }
}
