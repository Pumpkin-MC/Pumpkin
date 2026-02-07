use std::sync::Arc;

use crate::entity::{
    Entity, NBTStorage,
    mob::{Mob, MobEntity},
    passive::cow::CowEntity,
};

/// Mooshroom — a mushroom-covered variant of the cow.
///
/// Delegates to CowEntity for base AI behavior.
/// Shearing for mushrooms is a future addition.
pub struct MooshroomEntity {
    cow: Arc<CowEntity>,
}

impl MooshroomEntity {
    pub async fn new(entity: Entity) -> Arc<Self> {
        let cow = CowEntity::new(entity).await;
        Arc::new(Self { cow })
    }
}

impl NBTStorage for MooshroomEntity {}

impl Mob for MooshroomEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.cow.mob_entity
    }
}
