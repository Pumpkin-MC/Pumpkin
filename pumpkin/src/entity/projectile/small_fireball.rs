use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NBTStorage,
        projectile::{ProjectileHit, ThrownItemEntity},
    },
    server::Server,
};

pub struct SmallFireballEntity {
    pub thrown: ThrownItemEntity,
}

impl SmallFireballEntity {
    #[must_use]
    pub const fn new(entity: Entity) -> Self {
        let thrown = ThrownItemEntity {
            entity,
            owner_id: None,
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
        };

        Self { thrown }
    }

    #[must_use]
    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter);
        Self { thrown }
    }
}

impl NBTStorage for SmallFireballEntity {}

impl EntityBase for SmallFireballEntity {
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
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

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            match hit {
                ProjectileHit::Entity { ref entity, .. } => {
                    let entity_clone = entity.clone();

                    tokio::spawn(async move {
                        entity_clone.get_entity().set_on_fire_for(5.0);
                        let _ = entity_clone
                            .damage(
                                entity_clone.as_ref(),
                                5.0,
                                pumpkin_data::damage::DamageType::FIREBALL,
                            )
                            .await;
                    });
                }
                ProjectileHit::Block { pos, face, .. } => {
                    // Try to place fire
                    let block_to_place = match face {
                        pumpkin_data::BlockDirection::Up => pos.up(),
                        pumpkin_data::BlockDirection::Down => pos.down(),
                        pumpkin_data::BlockDirection::North => pos.north(),
                        pumpkin_data::BlockDirection::South => pos.south(),
                        pumpkin_data::BlockDirection::West => pos.west(),
                        pumpkin_data::BlockDirection::East => pos.east(),
                    };
                    let world = self.get_entity().world.load();
                    let fire_state = pumpkin_data::Block::FIRE.default_state.id;
                    world
                        .set_block_state(
                            &block_to_place,
                            fire_state,
                            pumpkin_world::world::BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                }
            }
        })
    }
}
