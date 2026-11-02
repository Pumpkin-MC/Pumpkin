use std::sync::Arc;

use pumpkin_data::BlockState;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::Sound;
use pumpkin_util::math::position::BlockPos;

use crate::entity::{
    Entity, EntityBase,
    ai::behavior::hunting::Hunt,
    ai::target_match::TargetMatch,
    mob::{Mob, MobEntity, skeleton::SkeletonEntityBase},
    projectile::arrow::ArrowEntity,
};

const WITHER_SKELETON_HUNTS: &[Hunt] = &[Hunt::new(3, TargetMatch::PIGLIN_LIKE, true)];

pub struct WitherSkeletonEntity {
    entity: Arc<SkeletonEntityBase>,
}

impl WitherSkeletonEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = SkeletonEntityBase::with_hunts(entity, WITHER_SKELETON_HUNTS);
        let skeleton = Self { entity };
        Arc::new(skeleton)
    }
}

impl Mob for WitherSkeletonEntity {
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityWitherSkeletonStep, 0.15, 1.0);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn on_attack(&self, target: &dyn EntityBase) {
        if let Some(living) = target.get_living_entity() {
            living.add_effect(Effect {
                effect_type: &StatusEffect::WITHER,
                duration: 200,
                amplifier: 0,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: false,
            });
        }
    }

    fn get_arrow(&self, arrow: ArrowEntity) -> ArrowEntity {
        // vanilla lights it for 100 seconds
        arrow.set_flame(true);
        arrow
    }
}
