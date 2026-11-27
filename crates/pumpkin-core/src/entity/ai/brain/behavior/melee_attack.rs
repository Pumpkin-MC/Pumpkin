use std::sync::Arc;

use super::super::memory::{MemoryStatus, types};
use super::{OneShot, utils};
use crate::entity::{EntityBase, mob::Mob};
use crate::world::World;

fn perform(world: &Arc<World>, mob: &dyn Mob, target: &dyn EntityBase) {
    let attacker = mob.get_entity();
    let victim = target.get_entity();
    let still_present = || {
        !attacker.is_removed()
            && !victim.is_removed()
            && Arc::ptr_eq(&attacker.world.load(), world)
            && Arc::ptr_eq(&victim.world.load(), world)
    };
    // Validation at effect entry is handled by the dispatcher. The swing packet
    // can synchronously call plugins, so validate both participants after it.
    mob.get_mob_entity().living_entity.swing_hand();
    if still_present()
        && mob.do_hurt_target(target)
        && !attacker.is_removed()
        && Arc::ptr_eq(&attacker.world.load(), world)
    {
        mob.on_attack(target);
    }
}

// The attack sound plays from `Mob::on_attack` after a damaging hit, not inside `do_hurt_target`.
#[must_use]
pub fn create(cooldown_between_attacks: i32) -> OneShot {
    create_with(|_| true, cooldown_between_attacks)
}

#[must_use]
pub fn create_with(
    can_attack: impl Fn(&dyn Mob) -> bool + Send + Sync + 'static,
    cooldown_between_attacks: i32,
) -> OneShot {
    OneShot::new(
        "MeleeAttack",
        vec![
            (types::LOOK_TARGET.id(), MemoryStatus::Registered),
            (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
            (types::ATTACK_COOLING_DOWN.id(), MemoryStatus::ValueAbsent),
            (
                types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
                MemoryStatus::ValuePresent,
            ),
        ],
        move |tick| {
            let Some(target) = tick.state.get(types::ATTACK_TARGET).cloned() else {
                return false;
            };
            if !can_attack(tick.mob)
                || is_holding_usable_non_melee_weapon(tick.mob)
                || !utils::is_within_attack_range(tick.mob, target.as_ref())
                || !utils::entity_is_visible(&tick.visibility(), target.as_ref())
            {
                return false;
            }
            utils::look_at_entity(tick.state, target.clone());
            let world = tick.world.clone();
            tick.defer_with_update(
                move |mob| perform(&world, mob, target.as_ref()),
                move |tick, ()| {
                    tick.state.set_with_expiry(
                        types::ATTACK_COOLING_DOWN,
                        true,
                        i64::from(cooldown_between_attacks),
                    );
                },
            );
            true
        },
    )
}

fn is_holding_usable_non_melee_weapon(mob: &dyn Mob) -> bool {
    let living = &mob.get_mob_entity().living_entity;
    mob.can_use_non_melee_weapon(&living.held_item(mob))
        || mob.can_use_non_melee_weapon(&living.off_hand_item(mob))
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::mpsc;
    use std::time::Duration;

    use pumpkin_data::dimension::Dimension;
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::environment_attribute::Activity;
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::brain::Brain;
    use crate::entity::ai::brain::activity::ActivityData;
    use crate::entity::ai::brain::memory::NearestVisibleLivingEntities;
    use crate::entity::ai::test_support::{server, world};
    use crate::entity::mob::{Mob, MobEntity};
    use crate::entity::passive::axolotl::AxolotlEntity;
    use crate::entity::passive::cow::CowEntity;
    use crate::plugin::{
        BoxFuture, EventRwHandler,
        api::events::{EventPriority, entity::entity_damage_by_entity::EntityDamageByEntityEvent},
    };
    use crate::server::Server;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    struct Biter {
        mob_entity: MobEntity,
        hits: AtomicUsize,
        sounds: AtomicUsize,
        damages: bool,
    }

    impl Mob for Biter {
        fn get_mob_entity(&self) -> &MobEntity {
            &self.mob_entity
        }

        fn do_hurt_target(&self, _target: &dyn EntityBase) -> bool {
            self.hits.fetch_add(1, Ordering::Relaxed);
            self.damages
        }

        fn on_attack(&self, _target: &dyn EntityBase) {
            self.sounds.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn biter(world: &Arc<World>, damages: bool) -> Biter {
        Biter {
            mob_entity: MobEntity::new(Entity::new(
                world.clone(),
                Vector3::new(0.0, 64.0, 0.0),
                &EntityType::COW,
            )),
            hits: AtomicUsize::new(0),
            sounds: AtomicUsize::new(0),
            damages,
        }
    }

    fn cow(world: &Arc<World>, x: f64) -> Arc<CowEntity> {
        CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(x, 64.0, 0.0),
            &EntityType::COW,
        ))
    }

    fn brain_for(target: &Arc<CowEntity>, visible: bool) -> Brain {
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Core,
            0,
            vec![Box::new(create(20))],
        ));
        let target: Arc<dyn EntityBase> = target.clone();
        brain.set(
            types::NEAREST_VISIBLE_LIVING_ENTITIES,
            NearestVisibleLivingEntities::new(if visible {
                vec![target.clone()]
            } else {
                Vec::new()
            }),
        );
        brain.set(types::ATTACK_TARGET, target);
        brain
    }

    #[tokio::test]
    async fn hits_once_per_cooldown_and_plays_the_sound_for_damaging_hits() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let body = biter(&world, true);
        let prey = cow(&world, 1.5);
        let mut brain = brain_for(&prey, true);

        brain.tick(&world, &body, 100);
        let cooldown = brain.time_until_expiry(types::ATTACK_COOLING_DOWN.id());
        let looks_at_it = brain
            .get(types::LOOK_TARGET)
            .and_then(|look| look.tracked_entity())
            .map(|entity| entity.get_entity().entity_id);
        let after_first = (
            body.hits.load(Ordering::Relaxed),
            body.sounds.load(Ordering::Relaxed),
        );

        for time in 101..=120 {
            brain.tick(&world, &body, time);
        }
        let during_cooldown = body.hits.load(Ordering::Relaxed);
        brain.tick(&world, &body, 121);
        let after_cooldown = body.hits.load(Ordering::Relaxed);
        world.level.shutdown().await;

        assert_eq!(cooldown, Some(20));
        assert_eq!(looks_at_it, Some(prey.get_entity().entity_id));
        assert_eq!(after_first, (1, 1));
        assert_eq!(during_cooldown, 1);
        assert_eq!(after_cooldown, 2);
        Ok(())
    }

    #[derive(Default)]
    struct ReadAttackerNbt(AtomicBool);

    impl EventRwHandler<EntityDamageByEntityEvent> for ReadAttackerNbt {
        fn handle<'a>(
            &'a self,
            server: &'a Arc<Server>,
            event: &'a mut EntityDamageByEntityEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
                let Some(attacker) = world.get_entity_by_id(event.damager_id) else {
                    return;
                };
                let (sent, read) = mpsc::channel();
                // If the tick is still holding the brain lock, let the handler time out
                // and return so the test can fail without leaving the tick blocked.
                let _reader = std::thread::spawn(move || {
                    let mut nbt = NbtCompound::new();
                    attacker.write_nbt(&mut nbt);
                    let before_cooldown = attacker.get_mob().is_some_and(|mob| {
                        mob.get_mob_entity().with_brain(mob, |brain| {
                            !brain.has_memory_value(types::ATTACK_COOLING_DOWN.id())
                        })
                    });
                    let _ = sent.send(nbt.get_compound("Brain").is_some() && before_cooldown);
                });
                self.0.store(
                    read.recv_timeout(Duration::from_secs(2))
                        .is_ok_and(|result| result),
                    Ordering::Relaxed,
                );
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn axolotl_damage_callback_reads_brain_before_cooldown() -> TestResult {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        let axolotl = AxolotlEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::AXOLOTL,
        ));
        let prey = cow(&world, 1.5);
        world.add_entity_silent(axolotl.clone());
        world.add_entity_silent(prey.clone());
        let health = prey.get_mob_entity().living_entity.health.load();
        let handler = Arc::new(ReadAttackerNbt::default());
        server
            .plugin_manager
            .register_blocking::<EntityDamageByEntityEvent, _>(
                handler.clone(),
                EventPriority::Normal,
                None,
            );
        let after_attack = Arc::new(AtomicBool::new(false));
        let mut brain = brain_for(&prey, true);
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Core,
            1,
            vec![Box::new(OneShot::new("AfterMelee", Vec::new(), {
                let after_attack = after_attack.clone();
                let prey = prey.clone();
                move |tick| {
                    after_attack.store(
                        prey.get_mob_entity().living_entity.health.load() < health
                            && tick.state.has_memory_value(types::ATTACK_COOLING_DOWN.id()),
                        Ordering::Relaxed,
                    );
                    true
                }
            }))],
        ));
        let mob = axolotl.get_mob_entity();
        mob.set_brain(brain);
        mob.tick_brain(axolotl.as_ref());
        server.shutdown().await;

        assert!(handler.0.load(Ordering::Relaxed));
        assert!(after_attack.load(Ordering::Relaxed));
        Ok(())
    }
}
