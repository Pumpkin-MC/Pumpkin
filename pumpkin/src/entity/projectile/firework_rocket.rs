use crate::{
    entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, projectile::ThrownItemEntity},
    server::Server,
    world::World,
};
use pumpkin_data::{entity::EntityStatus, meta_data_type::MetaDataType, tracked_data::TrackedData};
use pumpkin_protocol::{codec::optional_int::OptionalInt, java::client::play::Metadata};
use pumpkin_util::{
    math::vector3::Vector3,
    random::{RandomGenerator, RandomImpl, get_seed, xoroshiro128::Xoroshiro},
};
use std::sync::atomic::AtomicBool;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

pub struct FireworkRocketEntity {
    entity: ThrownItemEntity,
    shooter_id: Option<i32>,
    life: AtomicU32,
    life_time: AtomicU32,
}

impl FireworkRocketEntity {
    pub async fn new(entity: Entity) -> Self {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));

        entity
            .set_velocity(Vector3::new(
                random.next_triangular(0.0, 0.002_297),
                0.05,
                random.next_triangular(0.0, 0.002_297),
            ))
            .await;
        Self {
            entity: ThrownItemEntity {
                entity,
                owner_id: None,
                collides_with_projectiles: false,
                has_hit: AtomicBool::new(false),
            },
            shooter_id: None,
            life: 0.into(),
            // TODO
            life_time: (10 + random.next_bounded_i32(6) as u32 + random.next_bounded_i32(7) as u32)
                .into(),
        }
    }

    pub async fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));

        // Set random initial velocity
        // Set on the inner entity after constructing ThrownItemEntity
        let thrown = ThrownItemEntity::new(entity, shooter);
        thrown
            .entity
            .set_velocity(Vector3::new(
                random.next_triangular(0.0, 0.002_297),
                0.05,
                random.next_triangular(0.0, 0.002_297),
            ))
            .await;

        // Set random life
        let rocket = Self {
            entity: thrown,
            shooter_id: Some(shooter.entity_id),
            life: 0.into(),
            life_time: (10 + random.next_bounded_i32(6) as u32 + random.next_bounded_i32(7) as u32)
                .into(),
        };

        // Set shooter metadata
        rocket
            .entity
            .entity
            .send_meta_data(&[Metadata::new(
                TrackedData::DATA_SHOOTER_ENTITY_ID,
                MetaDataType::OptionalInt,
                OptionalInt(Some(shooter.entity_id)),
            )])
            .await;

        rocket
    }

    pub async fn explode_and_remove(&self, world: &World) {
        let entity = self.get_entity();
        world
            .send_entity_status(entity, EntityStatus::ExplodeFireworkClient)
            .await;

        // TODO: Explode/colors

        entity.remove().await;
    }
}

impl NBTStorage for FireworkRocketEntity {}

impl EntityBase for FireworkRocketEntity {
    fn tick<'a>(
        &'a self,
        caller: Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.entity.process_tick(caller, server).await;

            let entity = self.get_entity();
            let world = entity.world.load();
            let mut velocity = entity.velocity.load();

            if let Some(shooter_id) = self.shooter_id {
                // Check if the player who fired this rocket still exists in the world
                if let Some(shooter) = world.get_entity_by_id(shooter_id) {
                    let shooter = shooter.get_entity();

                    // Logic for boosting Elytra flight
                    if shooter.fall_flying.load(Ordering::Relaxed) {
                        let rotation = shooter.rotation().to_f64();
                        let shooter_vel = shooter.velocity.load();

                        let new_shooter_vel =
                            shooter_vel + (rotation * 0.1 + (rotation * 1.5 - shooter_vel) * 0.5);

                        shooter.set_velocity(new_shooter_vel).await;

                        entity.set_pos(shooter.pos.load());
                        entity.set_velocity(new_shooter_vel).await;
                    }
                }
            } else {
                // Standard firework rocket flight logic
                velocity.x *= 1.15;
                velocity.z *= 1.15;
                velocity.y += 0.04;
                entity.set_velocity(velocity).await;
            }

            // Increment life and check for explosion
            let current_life = self.life.fetch_add(1, Ordering::Relaxed);
            if current_life > self.life_time.load(Ordering::Relaxed) {
                self.explode_and_remove(&world).await;
            }
        })
    }

    fn get_entity(&self) -> &crate::entity::Entity {
        &self.entity.entity
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn crate::entity::NBTStorage {
        self
    }
}
