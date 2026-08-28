use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{DEATH_TIMER_MAX, EnderDragonEntity};
use crate::entity::experience_orb::ExperienceOrbEntity;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

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
    }

    fn tick(&self, dragon: &EnderDragonEntity) {
        let mut t = dragon
            .dragon_death_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *t += 1;

        let entity = &dragon.mob_entity.living_entity.entity;
        let world = entity.world.load();

        if *t == 1 {
            world.play_sound(
                Sound::EntityEnderDragonDeath,
                SoundCategory::Hostile,
                &entity.pos.load(),
            );
        }

        if *t >= 180 && *t <= 200 {
            let xo = (rand::random::<f32>() - 0.5) * 8.0;
            let yo = (rand::random::<f32>() - 0.5) * 4.0;
            let zo = (rand::random::<f32>() - 0.5) * 8.0;
            let pos = entity.pos.load();
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

        if *t > 150 && *t % 5 == 0 {
            ExperienceOrbEntity::spawn(&world, entity.pos.load(), (xp_count as f32 * 0.08) as u32);
        }

        entity.velocity.store(Vector3::new(0.0, 0.1, 0.0));

        if *t >= DEATH_TIMER_MAX {
            ExperienceOrbEntity::spawn(&world, entity.pos.load(), (xp_count as f32 * 0.2) as u32);

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
