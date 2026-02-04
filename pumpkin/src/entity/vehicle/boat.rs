use std::sync::Arc;

use crate::entity::item::ItemEntity;
use crate::entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, living::LivingEntity};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;

pub struct BoatEntity {
    entity: Entity,
}

impl BoatEntity {
    pub fn new(entity: Entity) -> Self {
        Self { entity }
    }

    /// Maps boat entity type to corresponding item for drops
    fn entity_to_item(entity_type: &EntityType) -> &'static Item {
        match entity_type.id {
            val if val == EntityType::OAK_BOAT.id => &Item::OAK_BOAT,
            val if val == EntityType::OAK_CHEST_BOAT.id => &Item::OAK_CHEST_BOAT,
            val if val == EntityType::SPRUCE_BOAT.id => &Item::SPRUCE_BOAT,
            val if val == EntityType::SPRUCE_CHEST_BOAT.id => &Item::SPRUCE_CHEST_BOAT,
            val if val == EntityType::BIRCH_BOAT.id => &Item::BIRCH_BOAT,
            val if val == EntityType::BIRCH_CHEST_BOAT.id => &Item::BIRCH_CHEST_BOAT,
            val if val == EntityType::JUNGLE_BOAT.id => &Item::JUNGLE_BOAT,
            val if val == EntityType::JUNGLE_CHEST_BOAT.id => &Item::JUNGLE_CHEST_BOAT,
            val if val == EntityType::ACACIA_BOAT.id => &Item::ACACIA_BOAT,
            val if val == EntityType::ACACIA_CHEST_BOAT.id => &Item::ACACIA_CHEST_BOAT,
            val if val == EntityType::DARK_OAK_BOAT.id => &Item::DARK_OAK_BOAT,
            val if val == EntityType::DARK_OAK_CHEST_BOAT.id => &Item::DARK_OAK_CHEST_BOAT,
            val if val == EntityType::MANGROVE_BOAT.id => &Item::MANGROVE_BOAT,
            val if val == EntityType::MANGROVE_CHEST_BOAT.id => &Item::MANGROVE_CHEST_BOAT,
            val if val == EntityType::CHERRY_BOAT.id => &Item::CHERRY_BOAT,
            val if val == EntityType::CHERRY_CHEST_BOAT.id => &Item::CHERRY_CHEST_BOAT,
            val if val == EntityType::PALE_OAK_BOAT.id => &Item::PALE_OAK_BOAT,
            val if val == EntityType::PALE_OAK_CHEST_BOAT.id => &Item::PALE_OAK_CHEST_BOAT,
            val if val == EntityType::BAMBOO_RAFT.id => &Item::BAMBOO_RAFT,
            val if val == EntityType::BAMBOO_CHEST_RAFT.id => &Item::BAMBOO_CHEST_RAFT,
            _ => &Item::OAK_BOAT, // fallback
        }
    }
}

impl NBTStorage for BoatEntity {}

impl EntityBase for BoatEntity {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    /// Vanilla: `AbstractBoatEntity.canHit()` returns `!this.isRemoved()`
    fn can_hit(&self) -> bool {
        self.entity.is_alive()
    }

    /// Vanilla: boats are collidable
    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    /// Vanilla: `VehicleEntity.damage()` - accumulates damage and destroys when > 40
    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            // Simple implementation: destroy on any damage and drop the boat item
            // TODO: Implement proper damage wobble accumulation like vanilla

            let world = self.entity.world.load();
            let pos = self.entity.pos.load();

            // Drop the boat as an item
            let item = Self::entity_to_item(self.entity.entity_type);
            let item_stack = ItemStack::new(1, item);

            let item_entity = Entity::new(world.clone(), pos, &EntityType::ITEM);
            let item_entity = Arc::new(ItemEntity::new(item_entity, item_stack).await);
            world.spawn_entity(item_entity).await;

            // Remove the boat entity
            self.entity.remove().await;

            true
        })
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }
}
