use std::sync::Arc;

use pumpkin_data::{
    damage::DamageType,
    entity::EntityType,
    sound::{Sound, SoundCategory},
};
use pumpkin_world::item::ItemStack;

use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NBTStorage, item::ItemEntity,
        projectile::ProjectileHit, projectile::ThrownItemEntity,
    },
    server::Server,
};

const TRIDENT_DAMAGE: f32 = 8.0;

pub struct ThrownTridentEntity {
    pub thrown: ThrownItemEntity,
    pickup_item: ItemStack,
    drop_on_hit: bool,
}

impl ThrownTridentEntity {
    pub fn new_shot(
        entity: Entity,
        shooter: &Entity,
        pickup_item: ItemStack,
        drop_on_hit: bool,
    ) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter);
        Self {
            thrown,
            pickup_item,
            drop_on_hit,
        }
    }
}

impl NBTStorage for ThrownTridentEntity {}

impl EntityBase for ThrownTridentEntity {
    fn tick<'a>(
        &'a self,
        caller: Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move { self.thrown.process_tick(caller, server).await })
    }

    fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let world = self.get_entity().world.load_full();

            if let ProjectileHit::Entity { ref entity, .. } = hit {
                let owner = self
                    .thrown
                    .owner_id
                    .and_then(|owner_id| world.get_entity_by_id(owner_id));
                let caller = owner.as_deref().unwrap_or(entity.as_ref());
                entity
                    .damage_with_context(
                        caller,
                        TRIDENT_DAMAGE,
                        DamageType::TRIDENT,
                        Some(hit.hit_pos()),
                        owner.as_deref(),
                        owner.as_deref(),
                    )
                    .await;
                world
                    .play_sound(
                        Sound::ItemTridentHit,
                        SoundCategory::Players,
                        &hit.hit_pos(),
                    )
                    .await;
            } else {
                world
                    .play_sound(
                        Sound::ItemTridentHitGround,
                        SoundCategory::Players,
                        &hit.hit_pos(),
                    )
                    .await;
            }

            if self.drop_on_hit && !self.pickup_item.is_empty() {
                let item_entity = Entity::new(world.clone(), hit.hit_pos(), &EntityType::ITEM);
                let item_entity =
                    Arc::new(ItemEntity::new(item_entity, self.pickup_item.clone()).await);
                world.spawn_entity(item_entity).await;
            }
        })
    }
}
