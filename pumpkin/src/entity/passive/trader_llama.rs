use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity},
    passive::llama::LlamaEntity,
};

/// Trader Llama — a llama that accompanies wandering traders.
///
/// Delegates to `LlamaEntity` for base AI behavior.
pub struct TraderLlamaEntity {
    llama: Arc<LlamaEntity>,
}

impl TraderLlamaEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let llama = LlamaEntity::new(entity).await;
        Arc::new(Self { llama })
    }
}

impl NBTStorage for TraderLlamaEntity {}

impl Mob for TraderLlamaEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.llama.mob_entity
    }
}
