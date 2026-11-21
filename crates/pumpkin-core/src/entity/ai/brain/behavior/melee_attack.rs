use super::super::memory::{MemoryStatus, types};
use super::{OneShot, utils};
use crate::entity::mob::Mob;

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
            tick.mob.get_mob_entity().living_entity.swing_hand();
            if tick.mob.do_hurt_target(target.as_ref()) {
                tick.mob.on_attack(target.as_ref());
            }
            tick.state.set_with_expiry(
                types::ATTACK_COOLING_DOWN,
                true,
                i64::from(cooldown_between_attacks),
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
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use pumpkin_data::entity::EntityType;
    use pumpkin_data::environment_attribute::Activity;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::ai::brain::Brain;
    use crate::entity::ai::brain::activity::ActivityData;
    use crate::entity::ai::brain::memory::NearestVisibleLivingEntities;
    use crate::entity::ai::test_support::world;
    use crate::entity::mob::{Mob, MobEntity};
    use crate::entity::passive::cow::CowEntity;
    use crate::entity::{Entity, EntityBase};
    use crate::world::World;

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
}
