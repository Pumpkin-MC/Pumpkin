use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, slime::SlimeEntity},
};

pub struct MagmaCubeEntity {
    pub slime: Arc<SlimeEntity>,
}

impl MagmaCubeEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let slime = SlimeEntity::new(entity).await;
        Arc::new(Self { slime })
    }
}

impl NBTStorage for MagmaCubeEntity {}

impl Mob for MagmaCubeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        self.slime.get_mob_entity()
    }
}
