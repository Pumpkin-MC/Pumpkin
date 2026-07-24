use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, avoid_entity::AvoidEntityGoal, breed::BreedGoal,
        escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        leap_at_target::LeapAtTargetGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Fox — **has AI** (vanilla 26.2 CFR stand-in).
///
/// Implemented: Float, Panic, Breed, Avoid(Player/Wolf/PolarBear), Pounce/Leap,
/// Melee, FollowParent, Stroll, LookAt; targets chicken/rabbit/cod/salmon/tropical.
/// TODO: StalkPrey, Sleep, EatBerries, Faceplant, trust/defend.
pub struct FoxEntity {
    pub mob_entity: MobEntity,
}

impl FoxEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let fox = Self { mob_entity };
        let mob_arc = Arc::new(fox);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            // Vanilla: no TemptGoal (berries are FoxEatBerriesGoal).
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(2, EscapeDangerGoal::new(2.2));
            goal_selector.add_goal(3, BreedGoal::new(1.0));
            // Avoid players (trust filter TODO), wolves, polar bears.
            goal_selector.add_goal(
                4,
                Box::new(AvoidEntityGoal::new(&EntityType::PLAYER, 16.0, 1.6, 1.4)),
            );
            goal_selector.add_goal(
                4,
                Box::new(AvoidEntityGoal::new(&EntityType::WOLF, 8.0, 1.6, 1.4)),
            );
            goal_selector.add_goal(
                4,
                Box::new(AvoidEntityGoal::new(&EntityType::POLAR_BEAR, 8.0, 1.6, 1.4)),
            );
            // FoxPounceGoal stand-in
            goal_selector.add_goal(6, Box::new(LeapAtTargetGoal::new(0.4)));
            goal_selector.add_goal(7, Box::new(MeleeAttackGoal::new(1.2, true)));
            goal_selector.add_goal(8, Box::new(FollowParentGoal::new(1.25)));
            goal_selector.add_goal(10, Box::new(LeapAtTargetGoal::new(0.4)));
            goal_selector.add_goal(11, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                12,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 24.0),
            );

            // landTargetGoal: chicken/rabbit; fishTargetGoal: schooling fish
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::CHICKEN, false),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::RABBIT, false),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::COD, false),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::SALMON, false),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(
                    &mob_arc.mob_entity,
                    &EntityType::TROPICAL_FISH,
                    false,
                ),
            );
        };

        mob_arc
    }
}

impl NBTStorage for FoxEntity {}

impl Mob for FoxEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
