use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, escape_danger::EscapeDangerGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, revenge::RevengeGoal,
        snowball_attack::SnowballAttackGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

const TEMPT_ITEMS: &[&Item] = &[&Item::HAY_BLOCK];

/// Trader llama — same combat as llama (spit + wolf); caravan/guard TODO.
pub struct TraderLlamaEntity {
    pub mob_entity: MobEntity,
}

impl TraderLlamaEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let llama = Self { mob_entity };
        let mob_arc = Arc::new(llama);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(3, SnowballAttackGoal::new(1.25));
            goal_selector.add_goal(3, EscapeDangerGoal::new(1.2));
            goal_selector.add_goal(5, Box::new(TemptGoal::new(1.25, TEMPT_ITEMS)));
            goal_selector.add_goal(7, Box::new(WanderAroundGoal::new(0.7)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(9, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::WOLF, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for TraderLlamaEntity {
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

impl Mob for TraderLlamaEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
