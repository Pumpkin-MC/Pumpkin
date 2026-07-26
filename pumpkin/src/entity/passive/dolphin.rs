use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        avoid_entity::AvoidEntityGoal, dolphin_swim_with_player::DolphinSwimWithPlayerGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    ai::pathfinder::node::PathType,
    mob::{Mob, MobEntity},
};

/// Dolphin — water movement, player swimming support, and guardian avoidance.
pub struct DolphinEntity {
    pub mob_entity: MobEntity,
}

impl DolphinEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        {
            let mut nav = mob_entity.navigator.lock().unwrap();
            nav.set_pathfinding_malus(PathType::Water, 0.0);
            nav.set_pathfinding_malus(PathType::WaterBorder, 0.0);
        }
        let dolphin = Self { mob_entity };
        let mob_arc = Arc::new(dolphin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            // Vanilla 26.2 Dolphin.registerGoals (treasure/air/water TODO).
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(2, DolphinSwimWithPlayerGoal::new(4.0));
            goal_selector.add_goal(4, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(4, Box::new(RandomLookAroundGoal::default()));
            goal_selector.add_goal(
                5,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(6, Box::new(MeleeAttackGoal::new(1.2, true)));
            // Avoid guardians — do NOT hunt them.
            goal_selector.add_goal(
                9,
                Box::new(AvoidEntityGoal::new(&EntityType::GUARDIAN, 8.0, 1.0, 1.0)),
            );
            goal_selector.add_goal(
                9,
                Box::new(AvoidEntityGoal::new(
                    &EntityType::ELDER_GUARDIAN,
                    8.0,
                    1.0,
                    1.0,
                )),
            );

            // HurtByTarget + setAlertOthers (pack); ignore guardian as damage source TODO.
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                1,
                crate::entity::ai::goal::join_anger::JoinAngerGoal::new(&EntityType::DOLPHIN),
            );
        };

        mob_arc
    }
}

impl NBTStorage for DolphinEntity {
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

impl Mob for DolphinEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
