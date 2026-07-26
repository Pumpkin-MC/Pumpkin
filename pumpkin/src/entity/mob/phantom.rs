use std::sync::Arc;

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, melee_attack::MeleeAttackGoal,
        random_float::RandomFloatGoal,
    },
    mob::{Mob, MobEntity},
};

/// Phantom — vanilla only has AttackStrategy / Sweep / CircleAroundAnchor + player target.
/// Full flight circle not ported; float + melee is the GoalSelector stand-in.
pub struct PhantomEntity {
    pub mob_entity: MobEntity,
}

impl PhantomEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let phantom = Self { mob_entity };
        let mob_arc = Arc::new(phantom);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            // 1 PhantomAttackStrategyGoal ≈ hover / reacquire
            goal_selector.add_goal(1, RandomFloatGoal::new());
            // 2 PhantomSweepAttackGoal ≈ close-range melee dive
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.2, true)));
            // 3 PhantomCircleAroundAnchorGoal TODO

            // 1 PhantomAttackPlayerTargetGoal (insomnia filter TODO)
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for PhantomEntity {
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

impl Mob for PhantomEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0
    }
}
