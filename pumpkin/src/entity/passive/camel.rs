use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        camel_sit::CamelSitGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// `Camel.java:314` (`dash()`).
const DASH_COOLDOWN_TICKS: i32 = 55;
/// `Camel.java:187` (`isDashing() && dashCooldown < 50 && (onGround || isInLiquid || isPassenger)`).
const DASH_CLEAR_THRESHOLD: i32 = 50;

/// Represents a Camel, a passive mount that can carry two players and dash.
///
/// Wiki: <https://minecraft.wiki/w/Camel>
///
/// Vanilla's dash/rider-jump mechanic (`getRiddenInput`/`canJump`/`handleStartJump`,
/// `Camel.java:260-349`) is entirely rider-input driven -- a saddled, ridden camel dashes when its
/// controlling passenger double-taps jump. Pumpkin has no `PlayerRideableJumping`/mounted-input
/// routing system at all yet (confirmed absent from `entity/mod.rs` and `mob/mod.rs`, same gap
/// the equine framework's design doc flags as "Phase 2"), so nothing can ever call `dash()`/
/// `set_dashing(true)` here. What *is* implemented is the non-rider-dependent half of the state
/// machine: the `isDashing`/`DASH` synced flag and the cooldown countdown plus its
/// auto-clear-on-landing rule (`Camel.java:187-193`), so the plumbing is ready and correct for
/// whenever a future mounted-input system starts calling `set_dashing`/`dash`. Sitting/pose
/// (`CamelSitGoal`) is unaffected and already implemented separately.
pub struct CamelEntity {
    pub mob_entity: MobEntity,
    dashing: AtomicBool,
    dash_cooldown: AtomicI32,
}

impl CamelEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let camel = Self {
            mob_entity,
            dashing: AtomicBool::new(false),
            dash_cooldown: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(camel);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(CamelSitGoal::new()));
            goal_selector.add_goal(2, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                3,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(4, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn is_dashing(&self) -> bool {
        self.dashing.load(Relaxed)
    }

    /// `Camel.setDashing` (`Camel.java:323-325`).
    pub fn set_dashing(&self, dashing: bool) {
        self.dashing.store(dashing, Relaxed);
        self.get_entity().send_meta_data(
            &[Metadata::new(
                TrackedData::DASH,
                MetaDataType::BOOLEAN,
                dashing,
            )],
            None,
        );
    }

    /// `Camel.dash` (`Camel.java:312-316`): sets the cooldown and the dashing flag. Nothing
    /// currently calls this (see the struct doc), but it's kept ready for the future rider-input
    /// path.
    pub fn dash(&self) {
        self.dash_cooldown.store(DASH_COOLDOWN_TICKS, Relaxed);
        self.set_dashing(true);
    }
}

impl NBTStorage for CamelEntity {}

impl Mob for CamelEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            // Re-sends `BABY_ID` (dropped by overriding `mob_init_data_tracker`), matching the
            // blanket `Mob` `EntityBase` impl's default behavior (`mob/mod.rs`) -- same reason
            // `CatEntity` re-sends it manually.
            if entity.age.load(std::sync::atomic::Ordering::Relaxed) < 0 {
                entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    None,
                );
            }
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::DASH,
                    MetaDataType::BOOLEAN,
                    false,
                )],
                None,
            );
        })
    }

    /// `Camel.aiStep`'s dash-cooldown handling (`Camel.java:187-193`): once dashing, the flag
    /// clears as soon as the cooldown drops under 50 ticks and the camel is grounded, in liquid,
    /// or carrying a passenger; the cooldown itself always ticks down to zero regardless.
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let cooldown = self.dash_cooldown.load(Relaxed);

            if self.is_dashing() && cooldown < DASH_CLEAR_THRESHOLD {
                let grounded =
                    entity.on_ground.load(Relaxed) || entity.touching_water.load(Relaxed);
                let carrying_passenger = !entity.passengers.lock().await.is_empty();
                if grounded || carrying_passenger {
                    self.set_dashing(false);
                }
            }

            if cooldown > 0 {
                self.dash_cooldown.store(cooldown - 1, Relaxed);
            }
        })
    }
}
