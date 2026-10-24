use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

use pumpkin_data::effect::StatusEffect;
use pumpkin_data::potion::Effect;

use crate::entity::{
    Entity, EntityBase,
    mob::{
        Mob, MobEntity, MobMetadata,
        spider::{
            initialize_spider_navigation, register_spider_goals, set_climbing, spider_metadata,
        },
    },
};

pub struct CaveSpiderEntity {
    pub mob_entity: MobEntity,
}

impl CaveSpiderEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        initialize_spider_navigation(&mut mob_entity);
        let cave_spider = Self { mob_entity };
        let mob_arc = Arc::new(cave_spider);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        register_spider_goals(&mob_arc.mob_entity, mob_weak);

        mob_arc
    }
}

impl Mob for CaveSpiderEntity {
    fn finalize_spawn(
        &self,
        world: &Arc<crate::world::World>,
        group_data: Option<super::spawn::SpawnGroupData>,
    ) -> Option<super::spawn::SpawnGroupData> {
        super::spider::finalize_spider_spawn(&self.mob_entity, world, group_data)
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_metadata(&self) -> Option<MobMetadata> {
        Some(spider_metadata(&self.mob_entity))
    }

    fn post_tick(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        set_climbing(
            &self.mob_entity,
            entity.horizontal_collision.load(Ordering::SeqCst),
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
                living.add_effect(Effect {
                    effect_type: &StatusEffect::POISON,
                    duration,
                    amplifier: 0,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                });
            }
        }
    }
}
