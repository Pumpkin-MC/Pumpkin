use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct BlazeEntity {
    pub entity: Arc<MobEntity>,
}

impl BlazeEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let entity = Arc::new(MobEntity::new(entity));
        let zombie = Self { entity };
        let mob_arc = Arc::new(zombie);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        {
            let mut goal_selector = mob_arc.entity.goals_selector.lock().await;
            let mut target_selector = mob_arc.entity.target_selector.lock().await;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));

            goal_selector.add_goal(
                4,
                Box::new(
                    crate::entity::ai::goal::blaze_attack::BlazeShootFireballGoal::new(
                        Arc::downgrade(&mob_arc),
                    ),
                ),
            );

            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    pub const fn set_charged(&self, _charged: bool) {
        // TODO:
        // let flags = &self.entity.living_entity.entity.flags;

        // let new_je_flags = if charged {
        //     flags.fetch_or(1, Ordering::Relaxed) | 1
        // } else {
        //     flags.fetch_and(!1, Ordering::Relaxed) & !1
        // };
        // self.entity
        //     .living_entity
        //     .entity
        //     .send_meta_data(&[Metadata::new(
        //         TrackedData::FLAGS_ID,
        //         MetaDataType::BYTE,
        //         new_je_flags,
        //     )])
        //     .await;
    }
}

impl NBTStorage for BlazeEntity {}

impl Mob for BlazeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity
    }
}
