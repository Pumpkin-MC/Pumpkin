use std::sync::{Arc, Weak};

use pumpkin_data::{entity::EntityType, item::Item};

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    ai::pathfinder::node::PathType,
    mob::{Mob, MobEntity},
};

const TEMPT_ITEMS: &[&Item] = &[&Item::SEAGRASS];

pub struct TurtleEntity {
    pub mob_entity: MobEntity,
}

impl TurtleEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        {
            let mut nav = mob_entity.navigator.lock().unwrap();
            nav.set_pathfinding_malus(PathType::Water, 0.0);
            nav.set_pathfinding_malus(PathType::WaterBorder, 0.0);
        }
        let turtle = Self { mob_entity };
        let mob_arc = Arc::new(turtle);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            // Vanilla 26.2 Turtle.registerGoals — no predator AvoidEntity
            // (home/lay-egg/go-to-water are custom goals TODO).
            goal_selector.add_goal(0, EscapeDangerGoal::new(1.2));
            goal_selector.add_goal(1, BreedGoal::new(1.0));
            goal_selector.add_goal(2, Box::new(TemptGoal::new(1.1, TEMPT_ITEMS)));
            goal_selector.add_goal(3, Box::new(SwimGoal::default()));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(0.8)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(9, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for TurtleEntity {
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

impl Mob for TurtleEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
