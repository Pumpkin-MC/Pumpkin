use std::sync::Arc;

use pumpkin_data::{effect::StatusEffect, potion::Effect};
use pumpkin_util::Difficulty;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    mob::{Mob, MobEntity, spider::SpiderEntity},
};

pub struct CaveSpiderEntity {
    pub spider: Arc<SpiderEntity>,
}

const fn poison_duration(difficulty: Difficulty) -> Option<i32> {
    match difficulty {
        Difficulty::Normal => Some(7 * 20),
        Difficulty::Hard => Some(15 * 20),
        Difficulty::Peaceful | Difficulty::Easy => None,
    }
}

impl CaveSpiderEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let spider = SpiderEntity::new(entity);
        Arc::new(Self { spider })
    }
}

impl NBTStorage for CaveSpiderEntity {}

impl Mob for CaveSpiderEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        self.spider.get_mob_entity()
    }

    fn on_successful_attack<'a>(&'a self, target: &'a dyn EntityBase) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let duration =
                poison_duration(self.get_entity().world.load().level_info.load().difficulty);

            if let (Some(duration), Some(living)) = (duration, target.get_living_entity()) {
                living
                    .add_effect(Effect {
                        effect_type: &StatusEffect::POISON,
                        duration,
                        amplifier: 0,
                        ambient: false,
                        show_particles: true,
                        show_icon: true,
                        blend: false,
                    })
                    .await;
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::poison_duration;
    use pumpkin_util::Difficulty;

    #[test]
    fn cave_spider_poison_matches_vanilla_difficulty_durations() {
        assert_eq!(poison_duration(Difficulty::Peaceful), None);
        assert_eq!(poison_duration(Difficulty::Easy), None);
        assert_eq!(poison_duration(Difficulty::Normal), Some(140));
        assert_eq!(poison_duration(Difficulty::Hard), Some(300));
    }
}
