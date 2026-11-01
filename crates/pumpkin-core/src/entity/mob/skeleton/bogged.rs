use crate::entity::{
    Entity, EntityBase,
    mob::{Mob, MobEntity, skeleton::SkeletonEntityBase},
};
use pumpkin_data::BlockState;
use pumpkin_data::sound::Sound;
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

pub struct BoggedSkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl BoggedSkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity);
        let bogged = Self { entity };
        Arc::new(bogged)
    }
}

impl Mob for BoggedSkeletonEntity {
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityBoggedStep, 0.15, 1.0);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }
}
