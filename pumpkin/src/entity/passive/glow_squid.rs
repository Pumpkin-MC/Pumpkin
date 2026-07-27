use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        escape_danger::EscapeDangerGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    ai::pathfinder::{node::PathType, node_evaluator::EvaluatorKind},
    mob::{Mob, MobEntity},
};

/// Glow squid — water flee wander; glow particles TODO.
pub struct GlowSquidEntity {
    pub mob_entity: MobEntity,
}

impl GlowSquidEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        {
            let mut nav = mob_entity.navigator.lock().unwrap();
            // Vanilla GlowSquid extends Squid, which has no createNavigation override
            // (default GroundPathNavigation, Mob.java:196-198); it moves via custom
            // travel and SquidRandomMovementGoal (Squid.java:204,232). Pumpkin drives
            // squid wander through the Navigator, so the swim evaluator stands in
            // until custom squid movement lands.
            nav.set_evaluator_kind(EvaluatorKind::Swim {
                allow_breaching: false,
            });
            nav.set_pathfinding_malus(PathType::Water, 0.0);
            nav.set_pathfinding_malus(PathType::WaterBorder, 0.0);
        }
        let glow_squid = Self { mob_entity };
        let mob_arc = Arc::new(glow_squid);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.0));
            goal_selector.add_goal(2, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(
                3,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(4, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for GlowSquidEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        self.get_mob_entity().living_entity.write_nbt(nbt)
    }

    fn read_nbt_non_mut<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        self.get_mob_entity().living_entity.read_nbt_non_mut(nbt)
    }
}

impl Mob for GlowSquidEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
