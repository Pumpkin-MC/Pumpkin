use super::{Mob, MobEntity};
use crate::entity::ai::goal::destroy_egg::DestroyEggGoal;
use crate::entity::ai::goal::look_around::LookAroundGoal;
use crate::entity::ai::goal::zombie_attack::ZombieAttackGoal;
use crate::entity::attributes::AttributeBuilder;
use crate::entity::{
    Entity, NBTStorage,
    ai::goal::{active_target::ActiveTargetGoal, look_at_entity::LookAtEntityGoal},
};
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use std::sync::{Arc, Weak};

pub mod drowned;
pub mod husk;
#[allow(clippy::module_inception)]
pub mod zombie;
pub mod zombie_villager;

pub struct ZombieEntityBase {
    pub mob_entity: MobEntity,
}

impl ZombieEntityBase {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let zombie = Self { mob_entity };
        let mob_arc = Arc::new(zombie);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().await;
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().await;

            goal_selector.add_goal(4, DestroyEggGoal::new(1.0, 3));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(LookAroundGoal::default()));
            goal_selector.add_goal(3, ZombieAttackGoal::new(1.0, false));

            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    #[must_use]
    pub fn create_attributes() -> AttributeBuilder {
        AttributeBuilder::new()
            .add(Attributes::MAX_HEALTH, 20.0)
            .add(Attributes::MOVEMENT_SPEED, 0.23)
            .add(Attributes::ATTACK_DAMAGE, 3.0)
            .add(Attributes::FOLLOW_RANGE, 35.0)
            .add(Attributes::ARMOR, 2.0)
    }
}

impl NBTStorage for ZombieEntityBase {}

impl Mob for ZombieEntityBase {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
