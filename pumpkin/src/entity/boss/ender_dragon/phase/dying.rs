use super::EnderDragonPhase;
use crate::entity::boss::ender_dragon::{DEATH_TIMER_MAX, EnderDragonEntity, Vector3Ext};
use crate::entity::experience_orb::ExperienceOrbEntity;
use futures::future::BoxFuture;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;
use std::sync::atomic::Ordering;

pub struct DyingPhase;

impl super::Phase for DyingPhase {
    fn get_type(&self) -> EnderDragonPhase {
        EnderDragonPhase::Dying
    }

    fn begin<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            *dragon.target_location.lock().await = None;
            *dragon.dragon_death_time.lock().await = 0;
        })
    }

    #[expect(clippy::too_many_lines)]
    fn tick<'a>(&'a self, dragon: &'a EnderDragonEntity) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            // ────────────────────────────────────────────────────────────
            // Java: DragonDeathPhase.doServerTick() runs ONLY while
            // !isDeadOrDying() (health > 0).  Once health drops to 0,
            // this method is never called again and tickDeath() takes over.
            //
            // In Rust, DyingPhase.tick() is always called (because
            // !is_dead || phase == Dying), so we gate health management
            // on dragon_death_time == 0.  Once the death timer starts
            // (which only happens when health <= 0), health must stay
            // at 0.0 permanently — otherwise the dragon oscillates
            // between flying to the portal and rising up.
            // ────────────────────────────────────────────────────────────
            let death_started = *dragon.dragon_death_time.lock().await > 0;

            if !death_started {
                let origin = {
                    let guard = dragon.fight_origin.lock().await;
                    guard.0
                };

                let mut target = dragon.target_location.lock().await;
                if target.is_none() {
                    let world = dragon.mob_entity.living_entity.entity.world.load();
                    let height = world.get_heightmap_height(
                        pumpkin_world::chunk::ChunkHeightmapType::MotionBlocking,
                        origin.x,
                        origin.z,
                    );
                    let top_y = height as f64 + 1.0;
                    *target = Some(Vector3::new(
                        origin.x as f64 + 0.5,
                        top_y,
                        origin.z as f64 + 0.5,
                    ));
                }

                let entity = &dragon.mob_entity.living_entity.entity;
                let pos = entity.pos.load();
                let horizontal_collision = entity.horizontal_collision.load(Ordering::Relaxed);
                let vertical_collision = entity.vertical_collision.load(Ordering::Relaxed);
                if let Some(t) = *target {
                    let d = pos.distance_squared(t);
                    let in_range = (100.0..=22500.0).contains(&d)
                        && !horizontal_collision
                        && !vertical_collision;
                    if in_range {
                        dragon.mob_entity.living_entity.set_health(1.0);
                    } else {
                        dragon.mob_entity.living_entity.set_health(0.0);
                    }
                }
                drop(target);

                // If health is still > 0 (dragon is flying to the portal),
                // return early — steering is handled by ai_step().
                if dragon.mob_entity.living_entity.health.load() > 0.0 {
                    return;
                }
            }

            // ────────────────────────────────────────────────────────────
            // Java: EnderDragon.tickDeath() — only runs when isDeadOrDying().
            // Everything below (sound, particles, XP, rise, removal) is
            // gated on the dragon actually being dead.
            // ────────────────────────────────────────────────────────────
            let entity = &dragon.mob_entity.living_entity.entity;
            let world = entity.world.load();

            let mut t = dragon.dragon_death_time.lock().await;
            *t += 1;

            // Java: tickDeath line 533-535 — globalLevelEvent(1028, ...)
            if *t == 1 {
                world.play_sound(
                    Sound::EntityEnderDragonDeath,
                    SoundCategory::Hostile,
                    &entity.pos.load(),
                );
            }

            // Java: tickDeath lines 516-521 — explosion emitter particles at ticks 180-200.
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
                && !fight_mutex.lock().await.has_previously_killed_dragon()
            {
                12000
            } else {
                500
            };

            // Java: tickDeath lines 529-531 — XP orbs every 5 ticks after tick 150.
            if *t > 150 && *t % 5 == 0 {
                ExperienceOrbEntity::spawn(
                    &world,
                    entity.pos.load(),
                    (xp_count as f32 * 0.08) as u32,
                )
                .await;
            }

            // Java: tickDeath lines 538-544 — rise upward by 0.1 blocks/tick.
            let death_move = Vector3::new(0.0, 0.1, 0.0);
            let pos = entity.pos.load();
            entity.set_pos(Vector3::new(pos.x, pos.y + death_move.y, pos.z));

            for part in &dragon.parts {
                let part_pos = part.entity.pos.load();
                part.entity.set_pos(Vector3::new(
                    part_pos.x,
                    part_pos.y + death_move.y,
                    part_pos.z,
                ));
            }

            entity.send_pos_rot();

            // Java: tickDeath lines 546-557 — final XP drop, dragon killed, remove.
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

    fn get_max_y_acceleration(&self) -> f32 {
        3.0
    }

    fn get_fly_speed(&self) -> f32 {
        3.0
    }
}
