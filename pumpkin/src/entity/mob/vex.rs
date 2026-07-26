use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, random_float::RandomFloatGoal, revenge::RevengeGoal,
    },
    mob::{Mob, MobEntity},
};

/// Vex — vanilla 26.2: Float, ChargeAttack, RandomMove, Look; player + owner target.
///
/// Decompile: no villager/golem NearestAttackableTarget; CopyOwnerTarget TODO.
pub struct VexEntity {
    pub mob_entity: MobEntity,
}

impl VexEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let vex = Self { mob_entity };
        let mob_arc = Arc::new(vex);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            // FloatGoal — flying uses zero gravity; RandomFloat ≈ VexRandomMove
            goal_selector.add_goal(0, RandomFloatGoal::new());
            // VexChargeAttackGoal stand-in
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.2, true)));
            goal_selector.add_goal(8, RandomFloatGoal::new());
            goal_selector.add_goal(
                9,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 3.0),
            );

            // HurtByTarget(Raider ignore).setAlertOthers — JoinAnger raiders TODO
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            // VexCopyOwnerTargetGoal TODO
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for VexEntity {
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

impl Mob for VexEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0
    }
}
