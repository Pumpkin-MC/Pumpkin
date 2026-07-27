use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::water_animal::WaterAnimalAir;
use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        avoid_entity::AvoidEntityGoal, escape_danger::EscapeDangerGoal,
        follow_school_leader::FollowSchoolLeaderGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    ai::pathfinder::{node::PathType, node_evaluator::EvaluatorKind},
    mob::{Mob, MobEntity},
};

/// Tropical fish — school; flee players.
pub struct TropicalFishEntity {
    pub mob_entity: MobEntity,
    /// `AbstractFish` inherits `WaterAnimal`'s land-drowning air supply
    /// (WaterAnimal.java:43-53).
    air: WaterAnimalAir,
}

impl TropicalFishEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        {
            let mut nav = mob_entity.navigator.lock().unwrap();
            // Vanilla AbstractFish.createNavigation (AbstractFish.java:106-108):
            // `new WaterBoundPathNavigation(this, level)`; breaching is dolphin-only
            // (WaterBoundPathNavigation.java:25).
            nav.set_evaluator_kind(EvaluatorKind::Swim {
                allow_breaching: false,
            });
            nav.set_pathfinding_malus(PathType::Water, 0.0);
            nav.set_pathfinding_malus(PathType::WaterBorder, 0.0);
        }
        let fish = Self {
            mob_entity,
            air: WaterAnimalAir::new(),
        };
        let mob_arc = Arc::new(fish);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.25));
            goal_selector.add_goal(
                2,
                Box::new(AvoidEntityGoal::new(&EntityType::PLAYER, 8.0, 1.6, 1.4)),
            );
            goal_selector.add_goal(4, Box::new(WanderAroundGoal::new(1.0)));
            // Vanilla AbstractSchoolingFish priority 5: FollowFlockLeaderGoal.
            goal_selector.add_goal(5, Box::new(FollowSchoolLeaderGoal::new(1.0)));
            goal_selector.add_goal(
                5,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for TropicalFishEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.get_mob_entity().living_entity.write_nbt(nbt).await;
            self.air.write_nbt(nbt);
        })
    }

    fn read_nbt_non_mut<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            self.get_mob_entity()
                .living_entity
                .read_nbt_non_mut(nbt)
                .await;
            self.air.read_nbt(nbt);
        })
    }
}

impl Mob for TropicalFishEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// `WaterAnimal.baseTick` drains the air supply and drowns the fish
    /// on land (WaterAnimal.java:56-64).
    fn mob_tick<'a>(&'a self, caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.air.tick(&self.mob_entity, caller).await;
        })
    }
}
