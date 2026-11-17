use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering::Relaxed};

use pumpkin_data::entity::EntityType;
use pumpkin_data::statistic::CustomStatistic;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::mob::equipment::RegionalDifficulty;
use crate::entity::mob::spawn::{EntitySpawnReason, finalize_spawn};
use crate::entity::r#type::from_type;
use crate::world::World;
use crate::world::natural_spawner::is_valid_empty_spawn_block;

const MIN_SKY_DARKEN: i32 = 5;
const MIN_TIME_SINCE_REST: i32 = 72_000;

/// Sends phantoms after players who have not slept for three days.
#[derive(Default)]
pub struct PhantomSpawner {
    next_tick: AtomicI32,
}

impl PhantomSpawner {
    pub fn tick(&self, world: &Arc<World>, spawn_enemies: bool) {
        if !spawn_enemies || !world.level_info.load().game_rules.spawn_phantoms {
            return;
        }
        let mut rng = rand::rng();
        if self.next_tick.fetch_sub(1, Relaxed) - 1 > 0 {
            return;
        }
        self.next_tick
            .fetch_add((60 + rng.random_range(0..60)) * 20, Relaxed);
        let has_sky_light = world.dimension.has_skylight;
        if world.get_sky_darken() < MIN_SKY_DARKEN && has_sky_light {
            return;
        }
        for player in world.players.load().iter() {
            if player.is_spectator() {
                continue;
            }
            let player_pos = player.get_entity().block_pos.load();
            if has_sky_light
                && (player_pos.0.y < world.sea_level || !world.can_see_sky(&player_pos))
            {
                continue;
            }
            let difficulty = RegionalDifficulty::at(world, player_pos.to_f64());
            if difficulty.effective_difficulty <= rng.random::<f32>() * 3.0 {
                continue;
            }
            let time_since_rest = player
                .get_custom_stat(CustomStatistic::TimeSinceRest)
                .max(1);
            if rng.random_range(0..time_since_rest) < MIN_TIME_SINCE_REST {
                continue;
            }
            let up = 20 + rng.random_range(0..15);
            let east = -10 + rng.random_range(0..21);
            let south = -10 + rng.random_range(0..21);
            let spawn_pos = player_pos.offset(Vector3::new(east, up, south));
            if !is_valid_empty_spawn_block(world.get_block_state(&spawn_pos), &EntityType::PHANTOM)
            {
                continue;
            }
            let group_size = 1 + rng.random_range(0..=difficulty.base_difficulty as i32);
            let pos = Vector3::new(
                f64::from(spawn_pos.0.x) + 0.5,
                f64::from(spawn_pos.0.y),
                f64::from(spawn_pos.0.z) + 0.5,
            );
            let mut group_data = None;
            for _ in 0..group_size {
                let phantom = from_type(&EntityType::PHANTOM, pos, world, Uuid::new_v4());
                phantom.get_entity().set_rotation(0.0, 0.0);
                group_data =
                    finalize_spawn(&phantom, world, EntitySpawnReason::Natural, group_data);
                world.spawn_entity(phantom);
            }
        }
    }
}
