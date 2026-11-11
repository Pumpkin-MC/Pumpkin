use super::super::memory::{MemoryStatus, types};
use super::{OneShot, utils};

/// Vanilla `MeleeAttack.create(cooldownBetweenAttacks)`: once the attack target is in reach and in
/// sight and the cooldown is over, looks at it, swings, hits it and starts the cooldown.
///
/// `on_attack` runs after a hit that did damage, where vanilla plays the attack sound inside
/// `doHurtTarget`. A mob holding a ranged weapon (`isHoldingUsableNonMeleeWeapon`) is not
/// special-cased: nothing here has that concept yet.
#[must_use]
pub fn create(cooldown_between_attacks: i32) -> OneShot {
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
            if !utils::is_within_attack_range(tick.mob, target.as_ref())
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

    /// A mob that counts how often it hit and whether it played its attack sound; the test world
    /// does not let a hit do damage, so `damages` says what `do_hurt_target` reports.
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

    /// In reach and in sight it hits once, looks at the target and starts the 20 tick cooldown;
    /// the sound plays only for a hit that did damage; during the cooldown it does not hit, and
    /// it hits again the tick after the cooldown memory has run out.
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

    /// A target out of reach or out of sight is not hit and starts no cooldown.
    #[tokio::test]
    async fn does_not_hit_a_target_out_of_reach_or_out_of_sight() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let body = biter(&world, true);
        let far = cow(&world, 4.0);
        let near = cow(&world, 1.5);
        let mut out_of_reach = brain_for(&far, true);
        let mut out_of_sight = brain_for(&near, false);

        out_of_reach.tick(&world, &body, 1);
        out_of_sight.tick(&world, &body, 1);
        world.level.shutdown().await;

        assert_eq!(body.hits.load(Ordering::Relaxed), 0);
        assert!(!out_of_reach.has_memory_value(types::ATTACK_COOLING_DOWN.id()));
        assert!(!out_of_sight.has_memory_value(types::ATTACK_COOLING_DOWN.id()));
        Ok(())
    }
}
