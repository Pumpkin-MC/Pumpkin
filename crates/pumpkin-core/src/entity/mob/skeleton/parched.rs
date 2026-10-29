use pumpkin_data::BlockState;
use pumpkin_data::sound::Sound;
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::entity::{
    Entity, EntityBase,
    mob::{Mob, MobEntity, skeleton::SkeletonEntityBase},
};

pub struct ParchedSkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl ParchedSkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::new(entity);
        let parched = Self { entity };
        Arc::new(parched)
    }
}

// TODO: arrows inflict Weakness for 600 ticks, immune to Weakness.
impl Mob for ParchedSkeletonEntity {
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityParchedStep, 0.15, 1.0);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }
}
