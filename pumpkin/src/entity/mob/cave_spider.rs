use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity, spider::SpiderEntity},
};

/// Cave spider — a smaller, poisonous variant of the Spider.
///
/// Inherits spider AI (melee attack, player targeting, swim, wander).
pub struct CaveSpiderEntity {
    spider: Arc<SpiderEntity>,
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
        &self.spider.mob_entity
    }
}
