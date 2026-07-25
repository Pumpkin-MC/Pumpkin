use std::sync::{Arc, Weak, atomic::Ordering};

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, escape_danger::EscapeDangerGoal,
        follow_parent::FollowParentGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Polar bear — vanilla 26.2 `PolarBear.registerGoals` (CFR).
///
/// ```text
/// 0 Float; 1 Melee(1.25); 1 Panic(2.0); 4 FollowParent(1.25);
/// 5 Stroll(1.0); 6 LookAt Player 6; 7 RandomLook
/// target: HurtBy; AttackPlayers(cub); angry Player TODO; Fox; ResetAnger TODO
/// ```
pub struct PolarBearEntity {
    pub mob_entity: MobEntity,
}

impl PolarBearEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let polar_bear = Self { mob_entity };
        let mob_arc = Arc::new(polar_bear);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            // Vanilla 26.2 CFR PolarBear.registerGoals
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(MeleeAttackGoal::new(1.25, true)));
            // PanicGoal stand-in (babies panic more broadly in vanilla).
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.0));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.25)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            // PolarBearHurtByTarget + AttackPlayers (cub) + angry player + fox + ResetAnger
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            let polar_bear = Arc::downgrade(&mob_arc);
            target_selector.add_goal(
                2,
                Box::new(
                    ActiveTargetGoal::new(
                        &mob_arc.mob_entity,
                        &EntityType::PLAYER,
                        20,
                        true,
                        false,
                        Some(move |_, world| {
                            let polar_bear = polar_bear.clone();
                            async move {
                                polar_bear
                                    .upgrade()
                                    .is_some_and(|bear| bear.has_nearby_cub(&world))
                            }
                        }),
                    )
                    .with_follow_distance_multiplier(0.5),
                ),
            );
            let polar_bear = Arc::downgrade(&mob_arc);
            target_selector.add_goal(
                4,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.mob_entity,
                    &EntityType::FOX,
                    10,
                    true,
                    false,
                    Some(move |_, _| {
                        let polar_bear = polar_bear.clone();
                        async move { polar_bear.upgrade().is_some_and(|bear| !bear.is_baby()) }
                    }),
                )),
            );
            // NeutralMob anger TODO
        };

        mob_arc
    }

    fn has_nearby_cub(&self, world: &crate::world::World) -> bool {
        let position = self.mob_entity.living_entity.entity.pos.load();
        if self.is_baby() {
            return false;
        }

        world
            .get_nearby_entities(position, 12.0)
            .into_values()
            .any(|candidate| {
                let entity = candidate.get_entity();
                entity.entity_type == &EntityType::POLAR_BEAR
                    && is_nearby_cub(
                        position,
                        entity.pos.load(),
                        entity.age.load(Ordering::Relaxed),
                    )
            })
    }

    fn is_baby(&self) -> bool {
        self.mob_entity
            .living_entity
            .entity
            .age
            .load(Ordering::Relaxed)
            < 0
    }
}

fn is_nearby_cub(bear_position: Vector3<f64>, cub_position: Vector3<f64>, cub_age: i32) -> bool {
    cub_age < 0
        && (cub_position.x - bear_position.x).abs() <= 8.0
        && (cub_position.y - bear_position.y).abs() <= 4.0
        && (cub_position.z - bear_position.z).abs() <= 8.0
}

impl NBTStorage for PolarBearEntity {}

impl Mob for PolarBearEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_baby_polar_bears_inside_the_vanilla_search_box_trigger_player_aggression() {
        let bear = Vector3::new(4.0, 64.0, 8.0);

        assert!(is_nearby_cub(bear, Vector3::new(12.0, 68.0, 0.0), -1,));
        assert!(!is_nearby_cub(bear, Vector3::new(12.1, 64.0, 8.0), -1,));
        assert!(!is_nearby_cub(bear, Vector3::new(4.0, 64.0, 8.0), 0));
    }
}
