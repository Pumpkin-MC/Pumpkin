use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_nbt::compound::NbtCompound;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, revenge::RevengeGoal, spear_use::SpearUseGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    persistent_anger::PersistentAnger,
};

pub struct ZombifiedPiglinEntity {
    pub mob_entity: MobEntity,
    pub persistent_anger: PersistentAnger,
}

impl ZombifiedPiglinEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let piglin = Self {
            mob_entity,
            persistent_anger: PersistentAnger::default(),
        };
        let mob_arc = Arc::new(piglin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, SpearUseGoal::new(1.0, 1.0, 10.0, 2.0));
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            // Zombified piglins are neutral: vanilla `ZombifiedPiglin.registerGoals`
            // registers no unconditional player target, only
            // `HurtByTargetGoal(this).setAlertOthers()` plus the `NeutralMob` anger
            // goals, which need per player anger state Pumpkin does not track yet.
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
        };

        mob_arc
    }
}

impl NBTStorage for ZombifiedPiglinEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async { self.persistent_anger.write_nbt(nbt).await })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async { self.persistent_anger.read_nbt(nbt).await })
    }
}

impl Mob for ZombifiedPiglinEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move { self.persistent_anger.tick().await })
    }
}
