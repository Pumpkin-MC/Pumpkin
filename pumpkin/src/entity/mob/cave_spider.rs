use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, spider::SpiderEntity},
};

pub struct CaveSpiderEntity {
    pub spider: Arc<SpiderEntity>,
}

impl CaveSpiderEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let spider = SpiderEntity::new(entity).await;
        Arc::new(Self { spider })
    }
}

impl NBTStorage for CaveSpiderEntity {}

impl Mob for CaveSpiderEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        self.spider.get_mob_entity()
    }
    // TODO: Poison effect on attack
}
