use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{DEATH_TIMER_MAX, EnderDragonEntity, Vector3Ext};
use crate::entity::experience_orb::ExperienceOrbEntity;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

pub struct DyingPhase;

impl super::Phase for DyingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::Dying
    }

    fn begin(&self, dragon: &EnderDragonEntity) {
        *dragon
            .target_location
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        *dragon
            .dragon_death_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = 0;
    }

    fn get_fly_speed(&self) -> f32 {
        3.0
    }

    fn get_fly_target_location(&self) -> Option<Vector3<f64>> {
        None
    }

    fn tick(&self, dragon: &EnderDragonEntity) {
        let death_time = {
            let mut t = dragon
                .dragon_death_time
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *t += 1;
            *t
        };

        let entity = &dragon.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();

        // 1. Resolve target location (center of the exit podium) if unset
        let target = {
            let mut target_loc_lock = dragon
                .target_location
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            if target_loc_lock.is_none() {
                let fight_origin = *dragon
                    .fight_origin
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let top_y = world.get_top_block(Vector2::new(fight_origin.0.x, fight_origin.0.z));
                let egg_y = if top_y > world.min_y {
                    top_y as f64
                } else {
                    65.0
                };
                *target_loc_lock = Some(Vector3::new(
                    fight_origin.0.x as f64 + 0.5,
                    egg_y,
                    fight_origin.0.z as f64 + 0.5,
                ));
            }
            *target_loc_lock
        };

        // 2. Steer towards target podium and update health matching vanilla doServerTick
        if let Some(target_pos) = target {
            let dist_sq = pos.distance_squared(target_pos);
            if (100.0..=22500.0).contains(&dist_sq) {
                dragon.steer_toward(pos, target_pos, 3.0, 0.1);
                dragon.mob_entity.living_entity.health.store(1.0);
            } else {
                dragon.mob_entity.living_entity.health.store(0.0);
            }
        }

        // 3. Play death sound at tick 1
        if death_time == 1 {
            world.play_sound(Sound::EntityEnderDragonDeath, SoundCategory::Hostile, &pos);
        }

        // 4. Explosion particles matching doClientTick (every 10 ticks and during final ticks)
        if death_time % 10 == 0 || (180..=200).contains(&death_time) {
            let xo = (rand::random::<f32>() - 0.5) * 8.0;
            let yo = (rand::random::<f32>() - 0.5) * 4.0;
            let zo = (rand::random::<f32>() - 0.5) * 8.0;
            world.spawn_particle(
                Vector3::new(
                    pos.x + xo as f64,
                    pos.y + 2.0 + yo as f64,
                    pos.z + zo as f64,
                ),
                Vector3::new(0.0, 0.0, 0.0),
                0.0,
                1,
                Particle::ExplosionEmitter,
            );
        }

        // 5. Spawn experience orbs
        let xp_count = if let Some(ref fight_mutex) = world.dragon_fight
            && !fight_mutex
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .has_previously_killed_dragon()
        {
            12000
        } else {
            500
        };

        if death_time > 150 && death_time % 5 == 0 {
            ExperienceOrbEntity::spawn(&world, pos, (xp_count as f32 * 0.08) as u32);
        }

        entity.velocity.store(Vector3::new(0.0, 0.1, 0.0));

        // 6. Complete death sequence at DEATH_TIMER_MAX
        if death_time >= DEATH_TIMER_MAX {
            ExperienceOrbEntity::spawn(&world, pos, (xp_count as f32 * 0.2) as u32);

            if let Some(ref fight_mutex) = world.dragon_fight {
                fight_mutex
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .set_dragon_killed(&world, entity.entity_uuid);
            }
            for part in &dragon.parts {
                part.entity.remove();
            }
            entity.remove();
        }
    }
}
