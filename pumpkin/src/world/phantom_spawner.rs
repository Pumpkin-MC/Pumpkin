//! Vanilla `PhantomSpawner` (26.2 CFR).
//!
//! **Not** "no damage taken then refresh monsters". Vanilla has no such rule for
//! hostiles. Monster population "refreshes" via `NaturalSpawner` + `Mob.checkDespawn`
//! (far from players → despawn, free cap → new spawns).
//!
//! Phantoms use **insomnia**: custom stat `TIME_SINCE_REST`. After ~3 in-game days
//! without sleeping (`random.nextInt(timeSinceRest) >= 72000`), phantoms may spawn
//! near the player at night.

use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

use pumpkin_data::entity::EntityType;
use pumpkin_data::statistic::CustomStatistic;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::{RngExt, rng};
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::entity::player::statistics::StatisticCategory;
use crate::entity::r#type::from_type;
use crate::world::World;

/// Vanilla: nextTick += (60 + random(60)) * 20 between attempts.
pub struct PhantomSpawner {
    next_tick: AtomicI32,
}

impl Default for PhantomSpawner {
    fn default() -> Self {
        Self {
            next_tick: AtomicI32::new(0),
        }
    }
}

impl PhantomSpawner {
    /// Vanilla `PhantomSpawner.tick(level, spawnEnemies)`.
    pub async fn tick(&self, world: &Arc<World>, spawn_enemies: bool) {
        if !spawn_enemies {
            return;
        }
        if !world.level_info.load().game_rules.spawn_phantoms {
            return;
        }

        // fetch_sub returns previous value; proceed when previous was <= 0 after --.
        let remaining = self.next_tick.fetch_sub(1, Ordering::Relaxed) - 1;
        if remaining > 0 {
            return;
        }

        let delay = (60 + rng().random_range(0..60)) * 20;
        self.next_tick.store(delay, Ordering::Relaxed);

        // Vanilla: if skyDarken < 5 && hasSkyLight → return (too bright).
        let sky_darken = world.sky_darken.load(Ordering::Relaxed);
        if sky_darken < 5 {
            return;
        }

        let sea_level = world.sea_level;
        let players: Vec<Arc<Player>> = world.players.load().iter().cloned().collect();

        for player in players {
            if player.is_spectator() {
                continue;
            }

            let player_pos = player.living_entity.entity.block_pos.load();
            // Must be above sea level and "see sky" (approx: full sky light).
            if player_pos.0.y < sea_level {
                continue;
            }
            let sky = world.get_sky_light_level(&player_pos);
            if sky < 15 {
                continue;
            }

            // Vanilla DifficultyInstance.isHarderThan(random * 3) — approximate
            // with global difficulty id (local difficulty not fully ported).
            let difficulty = world.level_info.load().difficulty;
            let diff_id = match difficulty {
                pumpkin_util::Difficulty::Peaceful => continue,
                pumpkin_util::Difficulty::Easy => 1.0f32,
                pumpkin_util::Difficulty::Normal => 2.0f32,
                pumpkin_util::Difficulty::Hard => 3.0f32,
            };
            if diff_id < rng().random::<f32>() * 3.0 {
                continue;
            }

            // Vanilla: Mth.clamp(TIME_SINCE_REST, 1, MAX)
            let time_since_rest = {
                let stats = player.stats.lock().await;
                stats
                    .get(
                        StatisticCategory::Custom,
                        CustomStatistic::TimeSinceRest as i32,
                    )
                    .max(1)
            };

            // Vanilla: `if (random.nextInt(value) < 72000 || !validBlock) continue;`
            // Proceed only when random >= 72000 (needs >3 days without sleep) AND air.
            if rng().random_range(0..time_since_rest) < 72_000 {
                continue;
            }

            // Spawn above player: y+20..35, x/z ±10
            let spawn_pos = BlockPos::new(
                player_pos.0.x + rng().random_range(-10..=10),
                player_pos.0.y + 20 + rng().random_range(0..15),
                player_pos.0.z + rng().random_range(-10..=10),
            );
            let state = world.get_block_state(&spawn_pos);
            if !state.is_air() {
                continue;
            }

            let group_size = 1 + rng().random_range(0..=diff_id as i32);
            for _ in 0..group_size {
                let pos_f64 = Vector3::new(
                    f64::from(spawn_pos.0.x) + 0.5,
                    f64::from(spawn_pos.0.y),
                    f64::from(spawn_pos.0.z) + 0.5,
                );
                let entity = from_type(&EntityType::PHANTOM, pos_f64, world, Uuid::new_v4());
                entity.get_entity().set_rotation(0.0, 0.0);
                world.spawn_entity(entity).await;
            }
        }
    }
}
