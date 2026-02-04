use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NBTStorage, projectile::ThrownItemEntity,
        r#type::from_type,
    },
    server::Server,
};
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct EggEntity {
    pub thrown: ThrownItemEntity,
    pub item_stack: RwLock<ItemStack>,
}

impl EggEntity {
    pub async fn new(entity: Entity) -> Self {
        // Default velocity slightly upward for thrown egg
        entity.set_velocity(Vector3::new(0.0, 0.1, 0.0)).await;
        let thrown = ThrownItemEntity {
            entity,
            owner_id: None,
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
        };

        Self {
            thrown,
            item_stack: RwLock::new(ItemStack::new(1, &Item::EGG)),
        }
    }

    pub async fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter);
        // Default slight upward velocity
        thrown
            .entity
            .set_velocity(Vector3::new(0.0, 0.1, 0.0))
            .await;

        Self {
            thrown,
            item_stack: RwLock::new(ItemStack::new(1, &Item::EGG)),
        }
    }

    /// Set the item stack shown by this thrown egg
    pub async fn set_item_stack(&self, item_stack: ItemStack) {
        let mut write = self.item_stack.write().await;
        *write = item_stack;
    }
}

impl NBTStorage for EggEntity {}

impl EntityBase for EggEntity {
    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let stack = self.item_stack.read().await;

            // Sync the item stack so the client renders the correct color/variant
            entity
                .send_meta_data(&[Metadata::new(
                    TrackedData::DATA_ITEM,
                    MetaDataType::ItemStack,
                    &ItemStackSerializer::from(stack.clone()),
                )])
                .await;
        })
    }

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

    fn on_hit(&self, hit: crate::entity::projectile::ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let world = self.get_entity().world.load();
            let hit_pos = hit.hit_pos();
            let normal = hit.normal();

            // Chicken spawn position offset slightly from hit position
            let spawn_pos = hit_pos.add(&normal.multiply(0.5, 0.5, 0.5));

            // Play egg break particles
            world
                .send_entity_status(
                    self.get_entity(),
                    EntityStatus::PlayDeathSoundOrAddProjectileHitParticles,
                )
                .await;

            // Decide spawn count per probabilities:
            // r == 0 -> spawn 4 (1/256)
            // r in 1..31 -> spawn 1 (31/256)
            // else -> 0
            let r: u8 = rand::random(); // 0..=255
            let to_spawn = if r == 0 { 4usize } else { usize::from(r < 32) };

            // Spawn chickens in a separate task to prevent stack overflow
            if to_spawn > 0 {
                let world_clone = world.clone();
                let spawn_pos_clone = spawn_pos;

                // Read the stack stored in set_item_stack
                //let stack = self.item_stack.lock().await;

                // TODO: Map the item ID to the chicken variant
                // let variant = match stack.item.id {
                //     id if id == Item::BLUE_EGG.id => EntityVariant::Cold,
                //     id if id == Item::BROWN_EGG.id => EntityVariant::Warm,
                //     _ => EntityVariant::Default,
                // };

                tokio::spawn(async move {
                    for _ in 0..to_spawn {
                        let mob = from_type(
                            &EntityType::CHICKEN,
                            spawn_pos_clone,
                            &world_clone,
                            Uuid::new_v4(),
                        )
                        .await;

                        let yaw = rand::random::<f32>() * 360.0;
                        let new_entity = mob.get_entity();
                        new_entity.set_rotation(yaw, 0.0);
                        new_entity.set_age(-24000);
                        //new_entity.set_variant(variant);

                        world_clone.spawn_entity(mob).await;
                    }
                });
            }
        })
    }
}
