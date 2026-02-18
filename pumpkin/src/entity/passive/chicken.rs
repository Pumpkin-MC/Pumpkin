use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, Ordering},
};

use pumpkin_data::{entity::EntityType, item::Item};
use pumpkin_world::item::ItemStack;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        escape_danger::EscapeDangerGoal, look_around::LookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

const TEMPT_ITEMS: &[&Item] = &[
    &Item::WHEAT_SEEDS,
    &Item::MELON_SEEDS,
    &Item::PUMPKIN_SEEDS,
    &Item::BEETROOT_SEEDS,
    &Item::TORCHFLOWER_SEEDS,
    &Item::PITCHER_POD,
];

pub struct ChickenEntity {
    pub mob_entity: MobEntity,
    egg_lay_time: AtomicI32,
}

impl ChickenEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let egg_lay_time = rand::rng().random_range(6000..12000);
        let chicken = Self {
            mob_entity,
            egg_lay_time: AtomicI32::new(egg_lay_time),
        };
        let mob_arc = Arc::new(chicken);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        mob_arc.mob_entity.living_entity.movement_speed.store(0.25);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, Box::new(EscapeDangerGoal::default()));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.0, TEMPT_ITEMS)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(7, Box::new(LookAroundGoal::default()));
        };

        mob_arc
    }
}

impl NBTStorage for ChickenEntity {}

impl Mob for ChickenEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async {
            if self.egg_lay_time.fetch_sub(1, Ordering::Relaxed) <= 1 {
                let next_time = rand::rng().random_range(6000..12000);
                let entity = &self.mob_entity.living_entity.entity;
                let world = entity.world.load_full();
                let pos = entity.block_pos.load();
                world.drop_stack(&pos, ItemStack::new(1, &Item::EGG)).await;
                self.egg_lay_time.store(next_time, Ordering::Relaxed);
            }
        })
    }
}
