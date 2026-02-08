use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, Ordering},
};

use pumpkin_data::{entity::EntityType, meta_data_type::MetaDataType, tracked_data::TrackedData};
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, creeper_ignite::CreeperIgniteGoal,
        flee_entity::FleeEntityGoal, look_around::LookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct CreeperEntity {
    pub mob_entity: MobEntity,
    pub fuse_speed: AtomicI32,
}

impl CreeperEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let entity = Self {
            mob_entity,
            fuse_speed: AtomicI32::new(-1),
        };
        let mob_arc = Arc::new(entity);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().await;

            goal_selector.add_goal(0, SwimGoal::new());
            goal_selector.add_goal(
                1,
                FleeEntityGoal::new(&EntityType::CAT, 6.0, 1.0, 1.2),
            );
            goal_selector.add_goal(2, Box::new(CreeperIgniteGoal::new(mob_arc.clone())));
            goal_selector.add_goal(5, WanderAroundGoal::new(0.8));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(LookAroundGoal::default()));

            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    pub async fn set_fuse_speed(&self, speed: i32) {
        self.fuse_speed.store(speed, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_FUSE_SPEED,
                MetaDataType::Integer,
                speed,
            )])
            .await;
    }
}

impl NBTStorage for CreeperEntity {}

impl Mob for CreeperEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
