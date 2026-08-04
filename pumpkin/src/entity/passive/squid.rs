use std::sync::{Arc, Weak};

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct SquidEntity {
    pub mob_entity: MobEntity,
}

impl SquidEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let squid = Self { mob_entity };
        let mob_arc = Arc::new(squid);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            // Vanilla `Squid.registerGoals` has no float/swim goal at all: squids are always
            // in water and don't need to surface.
            // NOTE: vanilla Squid has no WanderAroundGoal at all - locomotion instead comes
            // entirely from Squid.aiStep's jet-propulsion physics (a `movementVector` applied
            // directly to velocity, with `travel()` neutered to skip the generic AI-movement
            // path). That physics is NOT ported here: this codebase has no per-mob hook to
            // override/neuter the generic travel_in_fluid/travel_in_air path (they are
            // concrete LivingEntity methods, not part of the Mob trait), so a real port would
            // either need that hook added first or would double-apply movement on top of the
            // goal-driven system. Kept as the existing placeholder rather than removed, since
            // removing it with no replacement would leave squids unable to move at all.
            goal_selector.add_goal(1, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(
                2,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub const fn ink_particle(&self) -> Particle {
        Particle::SquidInk
    }

    #[must_use]
    pub const fn squirt_sound(&self) -> Sound {
        Sound::EntitySquidSquirt
    }
}

impl NBTStorage for SquidEntity {}

impl Mob for SquidEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// `Squid.hurtServer`: on a successful hit with a known attacker, spawns an ink cloud and
    /// plays the squirt sound. The exact "behind and below" rotated-cone particle placement
    /// from vanilla's `spawnInk` isn't ported (no body-rotation state is tracked without the
    /// jet-propulsion physics above); particles are emitted at the squid's own position.
    fn on_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if source.is_none() {
                return;
            }
            let entity = &self.mob_entity.living_entity.entity;
            let world = entity.world.load();
            let pos = entity.pos.load();
            world.spawn_particle(
                pos,
                Vector3::new(0.3, 0.3, 0.3),
                0.1,
                30,
                self.ink_particle(),
            );
            world.play_sound(self.squirt_sound(), SoundCategory::Neutral, &pos);
        })
    }
}
