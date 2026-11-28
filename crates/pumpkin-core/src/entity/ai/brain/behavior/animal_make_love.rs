use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::super::{BrainState, BrainTick};
use super::timed::Behavior;
use super::utils;
use crate::entity::EntityBase;
use crate::entity::ai::goal::breed::BreedGoal;
use crate::entity::mob::Mob;

const BREED_RANGE: f64 = 3.0;
const DEFAULT_SPEED_MODIFIER: f32 = 1.0;
const DEFAULT_CLOSE_ENOUGH_DISTANCE: i32 = 2;
const DURATION: i32 = 110;
const MIN_CHILD_DELAY: i32 = 60;
const CHILD_DELAY_SPREAD: i32 = 50;

/// Vanilla `Animal.canMate`: another animal, both in love, and the species' own check.
fn can_mate(mob: &dyn Mob, partner: &dyn EntityBase) -> bool {
    mob.as_animal().is_some_and(|animal| {
        partner.get_entity().entity_id != mob.get_entity().entity_id
            && mob.is_in_love()
            && partner.is_in_love()
            && animal.can_mate(partner)
    })
}

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 5] = [
    (
        types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
        MemoryStatus::ValuePresent,
    ),
    (types::BREED_TARGET.id(), MemoryStatus::ValueAbsent),
    (types::WALK_TARGET.id(), MemoryStatus::Registered),
    (types::LOOK_TARGET.id(), MemoryStatus::Registered),
    (types::IS_PANICKING.id(), MemoryStatus::ValueAbsent),
];

pub struct AnimalMakeLove {
    partner_type: &'static EntityType,
    speed_modifier: f32,
    close_enough_distance: i32,
    spawn_child_at_time: i64,
}

impl AnimalMakeLove {
    #[must_use]
    pub const fn new(partner_type: &'static EntityType) -> Self {
        Self::with_speed_and_distance(
            partner_type,
            DEFAULT_SPEED_MODIFIER,
            DEFAULT_CLOSE_ENOUGH_DISTANCE,
        )
    }

    #[must_use]
    pub const fn with_speed_and_distance(
        partner_type: &'static EntityType,
        speed_modifier: f32,
        close_enough_distance: i32,
    ) -> Self {
        Self {
            partner_type,
            speed_modifier,
            close_enough_distance,
            spawn_child_at_time: 0,
        }
    }

    fn find_valid_breed_partner(&self, tick: &BrainTick<'_>) -> Option<Arc<dyn EntityBase>> {
        tick.state
            .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)?
            .find_closest(&tick.visibility(), |entity| {
                entity.get_entity().entity_type == self.partner_type
                    && entity.get_mob().and_then(|mob| mob.as_animal()).is_some()
                    && can_mate(tick.mob, entity.as_ref())
                    && !entity.is_panicking()
            })
    }

    fn breed_target(&self, state: &BrainState) -> Option<Arc<dyn EntityBase>> {
        state
            .get(types::BREED_TARGET)
            .filter(|target| target.get_entity().entity_type == self.partner_type)
            .cloned()
    }
}

impl Behavior for AnimalMakeLove {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        DURATION
    }

    fn max_duration(&self) -> i32 {
        DURATION
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.mob.get_mob_entity().is_in_love() && self.find_valid_breed_partner(tick).is_some()
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        let Some(partner) = self.find_valid_breed_partner(tick) else {
            return;
        };
        tick.state.set(types::BREED_TARGET, partner.clone());
        if let (Some(partner_mob), Some(body)) = (
            partner.get_mob(),
            tick.world.get_entity_by_id(tick.mob.get_entity().entity_id),
        ) {
            partner_mob
                .get_mob_entity()
                .with_brain(partner_mob, |brain| brain.set(types::BREED_TARGET, body));
        }
        utils::lock_gaze_and_walk_to_each_other(
            tick,
            &partner,
            self.speed_modifier,
            self.close_enough_distance,
        );
        let delay = MIN_CHILD_DELAY
            + rand::RngExt::random_range(&mut tick.mob.get_random(), 0..CHILD_DELAY_SPREAD);
        self.spawn_child_at_time = tick.time + i64::from(delay);
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let Some(partner) = self.breed_target(tick.state) else {
            return false;
        };
        partner.get_entity().is_alive()
            && can_mate(tick.mob, partner.as_ref())
            && utils::entity_is_visible(&tick.visibility(), partner.as_ref())
            && tick.time <= self.spawn_child_at_time
            && !tick.state.has_memory_value(types::IS_PANICKING.id())
            && !partner.is_panicking()
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(partner) = self.breed_target(tick.state) else {
            return;
        };
        utils::lock_gaze_and_walk_to_each_other(
            tick,
            &partner,
            self.speed_modifier,
            self.close_enough_distance,
        );
        let distance = tick
            .mob
            .get_entity()
            .pos
            .load()
            .squared_distance_to_vec(&partner.get_entity().pos.load());
        if distance < BREED_RANGE * BREED_RANGE && tick.time >= self.spawn_child_at_time {
            let parent_id = tick.mob.get_entity().entity_id;
            let partner_id = partner.get_entity().entity_id;
            let partner_brain = partner.get_mob().map(|mate| {
                mate.get_mob_entity()
                    .with_brain(mate, super::super::Brain::identity)
            });
            tick.defer_with_update(
                move |mob| {
                    let mut pregnant = false;
                    BreedGoal::breed(mob, partner.as_ref(), |animal| {
                        // Frog's offspring hook writes Brain memory. Run it in the guarded
                        // update so the event cannot redirect the write to a replacement Brain.
                        if mob.get_entity().entity_type == &EntityType::FROG {
                            pregnant = true;
                        } else {
                            animal.on_bred_without_offspring(None);
                        }
                    });
                    if let (Some(mate), Some(identity)) = (partner.get_mob(), partner_brain) {
                        mate.get_mob_entity().with_brain(mate, |brain| {
                            if brain.has_identity(&identity)
                                && brain.get(types::BREED_TARGET).is_some_and(|target| {
                                    target.get_entity().entity_id == parent_id
                                })
                            {
                                brain.erase(types::BREED_TARGET.id());
                            }
                        });
                    }
                    pregnant
                },
                move |tick, pregnant| {
                    if pregnant && let Some(animal) = tick.mob.as_animal() {
                        animal.on_bred_without_offspring(Some(tick.state));
                    }
                    if tick
                        .state
                        .get(types::BREED_TARGET)
                        .is_some_and(|target| target.get_entity().entity_id == partner_id)
                    {
                        tick.state.erase(types::BREED_TARGET.id());
                    }
                },
            );
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>, _timed_out: bool) {
        tick.state.erase(types::BREED_TARGET.id());
        tick.state.erase(types::WALK_TARGET.id());
        tick.state.erase(types::LOOK_TARGET.id());
        self.spawn_child_at_time = 0;
    }

    fn debug_name(&self) -> &'static str {
        "AnimalMakeLove"
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::damage::DamageType;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::test_support::world;
    use crate::entity::passive::pig::PigEntity;

    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Mutex, mpsc};
    use std::time::Duration;

    use pumpkin_data::{dimension::Dimension, environment_attribute::Activity};

    use crate::entity::ai::brain::{Brain, activity::ActivityData, behavior::OneShot};
    use crate::entity::ai::test_support::server;
    use crate::entity::passive::frog::FrogEntity;
    use crate::plugin::{
        BoxFuture, EventRwHandler,
        api::events::{EventPriority, entity::entity_breed::EntityBreedEvent},
    };
    use crate::server::Server;

    enum BreedingEdit {
        None,
        ReplaceBrain,
        Redirect(Arc<FrogEntity>),
    }

    struct FrogBreedListener {
        frog: Arc<FrogEntity>,
        partner: Arc<FrogEntity>,
        edit: Mutex<BreedingEdit>,
        read_completed: AtomicBool,
        veto: AtomicBool,
    }

    impl EventRwHandler<EntityBreedEvent> for FrogBreedListener {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntityBreedEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                let (send, receive) = mpsc::channel();
                let frog = self.frog.clone();
                std::thread::spawn(move || {
                    let target = frog.mob_entity.get_target();
                    let _ = send.send(target.is_none());
                });
                self.read_completed.store(
                    receive.recv_timeout(Duration::from_secs(2)) == Ok(true),
                    Ordering::Relaxed,
                );
                match &*self
                    .edit
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                {
                    BreedingEdit::None => {}
                    BreedingEdit::ReplaceBrain => {
                        let mut replacement = Brain::default();
                        replacement.register_memory(types::BREED_TARGET.id());
                        // Even an identical target in the replacement belongs to a different Brain.
                        replacement.set(types::BREED_TARGET, self.frog.clone());
                        self.partner.mob_entity.set_brain(replacement);
                    }
                    BreedingEdit::Redirect(target) => {
                        self.partner
                            .mob_entity
                            .with_brain(self.partner.as_ref(), |brain| {
                                brain.set(types::BREED_TARGET, target.clone());
                            });
                        self.frog
                            .mob_entity
                            .with_brain(self.frog.as_ref(), |brain| {
                                brain.set(types::BREED_TARGET, target.clone());
                            });
                    }
                }
                event.cancelled = self.veto.load(Ordering::Relaxed);
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn frog_breeding_event_can_read_target_and_veto_pregnancy()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        let frog = |x| {
            FrogEntity::new(Entity::new(
                world.clone(),
                Vector3::new(x, 64.0, 0.0),
                &EntityType::FROG,
            ))
        };
        let (parent, partner) = (frog(0.0), frog(1.0));
        let listener = Arc::new(FrogBreedListener {
            frog: parent.clone(),
            partner: partner.clone(),
            edit: Mutex::new(BreedingEdit::None),
            read_completed: AtomicBool::new(false),
            veto: AtomicBool::new(false),
        });
        server
            .plugin_manager
            .register_blocking(listener.clone(), EventPriority::Normal, None);

        let other = frog(2.0);
        for (veto, edit, expected_target) in [
            (true, BreedingEdit::None, None),
            (false, BreedingEdit::None, None),
            (
                false,
                BreedingEdit::ReplaceBrain,
                Some(parent.get_entity().entity_id),
            ),
            (
                false,
                BreedingEdit::Redirect(other.clone()),
                Some(other.get_entity().entity_id),
            ),
        ] {
            let expected_parent_target = match &edit {
                BreedingEdit::Redirect(target) => Some(target.get_entity().entity_id),
                _ => None,
            };
            partner.mob_entity.with_brain(partner.as_ref(), |brain| {
                brain.set(types::BREED_TARGET, parent.clone());
            });
            *listener
                .edit
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = edit;
            parent.mob_entity.set_love_ticks(600, None);
            partner.mob_entity.set_love_ticks(600, None);
            listener.read_completed.store(false, Ordering::Relaxed);
            listener.veto.store(veto, Ordering::Relaxed);

            let mut brain = Brain::default();
            brain.register_memory(types::ATTACK_TARGET.id());
            brain.register_memory(types::IS_PREGNANT.id());
            let mut breeding = AnimalMakeLove::new(&EntityType::FROG);
            brain.add_activity(ActivityData::with_priority_start(
                Activity::Core,
                0,
                vec![Box::new(OneShot::new(
                    "CompleteFrogBreeding",
                    vec![(types::BREED_TARGET.id(), MemoryStatus::ValuePresent)],
                    move |tick| {
                        breeding.spawn_child_at_time = tick.time;
                        breeding.tick(tick);
                        true
                    },
                ))],
            ));
            brain.set(types::BREED_TARGET, partner.clone());
            assert!(!brain.has_memory_value(types::IS_PREGNANT.id()));
            parent.mob_entity.set_brain(brain);
            parent.mob_entity.tick_brain(parent.as_ref());

            assert!(listener.read_completed.load(Ordering::Relaxed));
            assert_eq!(
                parent.mob_entity.with_brain(parent.as_ref(), |brain| {
                    brain.has_memory_value(types::IS_PREGNANT.id())
                }),
                !veto,
            );
            assert!(!parent.mob_entity.is_in_love());
            assert!(!partner.mob_entity.is_in_love());
            let parent_target = parent.mob_entity.with_brain(parent.as_ref(), |brain| {
                brain
                    .get(types::BREED_TARGET)
                    .map(|target| target.get_entity().entity_id)
            });
            assert_eq!(parent_target, expected_parent_target);
            let partner_target = partner.mob_entity.with_brain(partner.as_ref(), |brain| {
                brain
                    .get(types::BREED_TARGET)
                    .map(|target| target.get_entity().entity_id)
            });
            assert_eq!(partner_target, expected_target);
        }
        server.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn a_hurt_animal_loses_its_love() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let pig = PigEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.5, 64.0, 0.5),
            &EntityType::PIG,
        ));
        pig.mob_entity.set_love_ticks(600, None);

        pig.actually_hurt(pig.as_ref(), DamageType::GENERIC, None, None, 1.0);
        let in_love = pig.mob_entity.is_in_love();
        world.level.shutdown().await;

        assert!(!in_love);
        Ok(())
    }

    #[tokio::test]
    async fn it_only_mates_with_another_animal_that_is_in_love()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let pig = || {
            PigEntity::new(Entity::new(
                world.clone(),
                Vector3::new(0.5, 64.0, 0.5),
                &EntityType::PIG,
            ))
        };
        let (first, partner) = (pig(), pig());

        first.mob_entity.set_love_ticks(600, None);
        let partner_not_in_love = can_mate(first.as_ref(), partner.as_ref());
        partner.mob_entity.set_love_ticks(600, None);
        let both_in_love = can_mate(first.as_ref(), partner.as_ref());
        let with_itself = can_mate(first.as_ref(), first.as_ref());
        world.level.shutdown().await;

        assert_eq!(
            (partner_not_in_love, both_in_love, with_itself),
            (false, true, false)
        );
        Ok(())
    }
}
