use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, slime::SlimeEntity},
};

pub struct MagmaCubeEntity {
    pub slime: Arc<SlimeEntity>,
}

impl MagmaCubeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let slime = SlimeEntity::new(entity);
        Arc::new(Self { slime })
    }
}

impl NBTStorage for MagmaCubeEntity {}

impl Mob for MagmaCubeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        self.slime.get_mob_entity()
    }

    fn mob_tick<'a>(
        &'a self,
        caller: &'a Arc<dyn crate::entity::EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        self.slime.mob_tick(caller)
    }

    fn post_tick(&self) -> crate::entity::EntityBaseFuture<'_, ()> {
        self.slime.post_tick()
    }

    fn mob_player_collision<'a>(
        &'a self,
        player: &'a Arc<crate::entity::player::Player>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        self.slime.mob_player_collision(player)
    }
}
