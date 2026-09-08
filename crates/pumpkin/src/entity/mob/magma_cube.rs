use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::attributes::Attributes;

use crate::entity::{
    Entity, EntityBase,
    mob::{Mob, MobEntity, slime::SlimeEntity},
};

pub struct MagmaCubeEntity {
    pub slime: Arc<SlimeEntity>,
}

impl MagmaCubeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let slime = SlimeEntity::new(entity);
        let size = slime.get_size();
        {
            let mut attributes = slime
                .get_mob_entity()
                .living_entity
                .attributes
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(speed) = attributes.get_mut(&Attributes::MOVEMENT_SPEED.id) {
                speed.base_value = 0.2;
                speed.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(damage) = attributes.get_mut(&Attributes::ATTACK_DAMAGE.id) {
                damage.base_value = (size + 2) as f64;
                damage.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(armor) = attributes.get_mut(&Attributes::ARMOR.id) {
                armor.base_value = (size * 3) as f64;
                armor.dirty.store(true, Ordering::Relaxed);
            }
        }
        Arc::new(Self { slime })
    }
}

impl Mob for MagmaCubeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        self.slime.get_mob_entity()
    }

    fn mob_tick(&self, caller: &dyn EntityBase) {
        self.slime.mob_tick(caller);
    }

    fn post_tick(&self) {
        self.slime.post_tick();
    }

    fn mob_player_collision(&self, player: &Arc<crate::entity::player::Player>) {
        self.slime
            .get_mob_entity()
            .try_attack(&*self.slime, &**player);
    }
}
