use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{DEATH_TIMER_MAX, EnderDragonEntity};
use crate::entity::experience_orb::ExperienceOrbEntity;
use futures::future::BoxFuture;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

pub struct DyingPhase;

impl super::Phase for DyingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::Dying
    }

    fn begin<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            *dragon.target_location.lock().await = None;
        })
    }

    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let mut t = dragon.dragon_death_time.lock().await;
            *t += 1;

            let entity = &dragon.mob_entity.living_entity.entity;
            let world = entity.world.load();

            if *t == 1 {
                world
                    .play_sound(
                        Sound::EntityEnderDragonDeath,
                        SoundCategory::Hostile,
                        &entity.pos.load(),
                    )
                    .await;
            }

            if *t >= 180 && *t <= 200 {
                let xo = (rand::random::<f32>() - 0.5) * 8.0;
                let yo = (rand::random::<f32>() - 0.5) * 4.0;
                let zo = (rand::random::<f32>() - 0.5) * 8.0;
                let pos = entity.pos.load();
                world
                    .spawn_particle(
                        Vector3::new(
                            pos.x + xo as f64,
                            pos.y + 2.0 + yo as f64,
                            pos.z + zo as f64,
                        ),
                        Vector3::new(0.0, 0.0, 0.0),
                        0.0,
                        1,
                        Particle::ExplosionEmitter,
                    )
                    .await;
            }

            let xp_count = if let Some(ref fight_mutex) = world.dragon_fight
                && !fight_mutex.lock().await.has_previously_killed_dragon()
            {
                12000
            } else {
                500
            };

            if *t > 150 && *t % 5 == 0 {
                ExperienceOrbEntity::spawn(
                    &world,
                    entity.pos.load(),
                    (xp_count as f32 * 0.08) as u32,
                )
                .await;
            }

            entity.velocity.store(Vector3::new(0.0, 0.1, 0.0));

            if *t >= DEATH_TIMER_MAX {
                ExperienceOrbEntity::spawn(
                    &world,
                    entity.pos.load(),
                    (xp_count as f32 * 0.2) as u32,
                )
                .await;

                if let Some(ref fight_mutex) = world.dragon_fight {
                    fight_mutex
                        .lock()
                        .await
                        .set_dragon_killed(&world, entity.entity_uuid)
                        .await;
                }
                for part in &dragon.parts {
                    part.entity.remove().await;
                }
                entity.remove().await;
            }
        })
    }
}
