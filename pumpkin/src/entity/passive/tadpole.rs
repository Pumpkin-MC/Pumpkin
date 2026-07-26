use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        avoid_entity::AvoidEntityGoal, escape_danger::EscapeDangerGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

const TEMPT_ITEMS: &[&Item] = &[&Item::SLIME_BALL];

/// Tadpole — panic + tempt; grow-into-frog TODO.
pub struct TadpoleEntity {
    pub mob_entity: MobEntity,
}

impl TadpoleEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let tadpole = Self { mob_entity };
        let mob_arc = Arc::new(tadpole);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.5));
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::PLAYER, 8.0, 1.5, 1.5)),
            );
            goal_selector.add_goal(
                1,
                Box::new(AvoidEntityGoal::new(&EntityType::AXOLOTL, 8.0, 1.5, 1.5)),
            );
            goal_selector.add_goal(2, Box::new(TemptGoal::new(1.0, TEMPT_ITEMS)));
            goal_selector.add_goal(3, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                4,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(5, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for TadpoleEntity {
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

impl Mob for TadpoleEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
