use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

/// Represents a Mooshroom, a fungal variant of cows that can be milked for mushroom stew.
///
/// Wiki: <https://minecraft.wiki/w/Mooshroom>
pub struct MooshroomEntity {
    pub mob_entity: MobEntity,
}

impl MooshroomEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mooshroom = Self { mob_entity };
        let mob_arc = Arc::new(mooshroom);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                2,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(3, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for MooshroomEntity {}

impl Mob for MooshroomEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
