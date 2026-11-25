use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::BlockState;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::Sound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase,
    mob::{Mob, MobEntity, MobMetadata},
};

const VEHICLE_SEAT_HEIGHT: f64 = 0.218_75;

pub struct CaveSpiderEntity {
    pub mob_entity: MobEntity,
    pub is_climbing: AtomicBool,
}

impl CaveSpiderEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let cave_spider = Self {
            mob_entity,
            is_climbing: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(cave_spider);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        super::spider::init_spider_ai(&mob_arc.mob_entity, mob_weak);

        mob_arc
    }
}

impl Mob for CaveSpiderEntity {
    fn mob_vehicle_attachment_point(&self, vehicle: &dyn EntityBase) -> Option<Vector3<f64>> {
        super::spider::small_vehicle_attachment_point(
            &self.mob_entity,
            vehicle,
            VEHICLE_SEAT_HEIGHT,
        )
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntitySpiderStep, 0.15, 1.0);
    }

    fn finalize_spawn(
        &self,
        _world: &Arc<crate::world::World>,
        _reason: crate::entity::mob::spawn::EntitySpawnReason,
        group_data: Option<super::spawn::SpawnGroupData>,
    ) -> Option<super::spawn::SpawnGroupData> {
        // No base spawn setup: no jockey, group effect, follow range bonus or handedness.
        group_data
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn on_climbable(&self) -> bool {
        self.is_climbing.load(Ordering::Relaxed)
    }

    fn mob_metadata(&self) -> Option<MobMetadata> {
        Some(super::spider::spider_metadata(
            &self.mob_entity,
            self.is_climbing.load(Ordering::Relaxed),
        ))
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        super::spider::tick_wall_climbing(
            self,
            &self.is_climbing,
            pumpkin_data::tracked_data::cave_spider::DATA_FLAGS_ID,
        );
    }

    fn on_attack(&self, target: &dyn EntityBase) {
        if let Some(living) = target.get_living_entity() {
            let world = self.mob_entity.living_entity.entity.world.load();
            let duration = match world.level_info.load().difficulty {
                pumpkin_util::Difficulty::Normal => 7 * 20,
                pumpkin_util::Difficulty::Hard => 15 * 20,
                _ => 0,
            };
            if duration > 0 {
                living.add_effect_with_source(
                    Effect {
                        effect_type: &StatusEffect::POISON,
                        duration,
                        amplifier: 0,
                        ambient: false,
                        show_particles: true,
                        show_icon: true,
                        blend: false,
                    },
                    Some(self),
                );
            }
        }
    }
}
