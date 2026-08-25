use std::sync::Arc;

use crate::entity::{
    Entity, EntityBaseFuture,
    mob::{Mob, MobEntity, equipment::RegionalDifficulty, skeleton::SkeletonEntityBase},
};
use crate::world::World;

pub struct SkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl SkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity);
        let skeleton = Self { entity };
        Arc::new(skeleton)
    }
}

impl Mob for SkeletonEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn populate_default_equipment_slots<'a>(
        &'a self,
        world: &'a Arc<World>,
        difficulty: &'a RegionalDifficulty,
    ) -> EntityBaseFuture<'a, ()> {
        self.entity
            .populate_default_equipment_slots(world, difficulty)
    }

    fn populate_default_equipment_enchantments<'a>(
        &'a self,
        difficulty: &'a RegionalDifficulty,
    ) -> EntityBaseFuture<'a, ()> {
        self.entity
            .populate_default_equipment_enchantments(difficulty)
    }
}
