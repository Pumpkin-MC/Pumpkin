use async_trait::async_trait;
use pumpkin_data::damage::DamageType;
use pumpkin_protocol::{
    client::play::{MetaDataType, Metadata},
    codec::slot::Slot,
};
use pumpkin_world::item::ItemStack;

use super::{living::LivingEntity, Entity, EntityBase};

pub struct ItemEntity {
    entity: Entity,
    item: Slot,
}

impl ItemEntity {
    pub fn new(entity: Entity, stack: &ItemStack) -> Self {
        let slot = Slot::from(stack);
        Self { entity, item: slot }
    }
    pub async fn send_meta_packet(&self) {
        self.entity
            .send_meta_data(Metadata::new(8, MetaDataType::ItemStack, self.item.clone()))
            .await;
    }
}

#[async_trait]
impl EntityBase for ItemEntity {
    async fn tick(&self) {}

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn is_invulnerable_to(&self, damage_type: DamageType) -> bool {
        self.get_entity().is_invulnerable_to(damage_type)
    }
}
