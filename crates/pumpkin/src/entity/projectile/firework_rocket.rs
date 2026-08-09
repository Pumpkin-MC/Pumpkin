use crate::{
    entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, projectile::ThrownItemEntity},
    server::Server,
    world::World,
};
use pumpkin_data::{
    data_component_impl::FireworksImpl, entity::EntityStatus, item_stack::ItemStack,
    meta_data_type::MetaDataType, tracked_data::TrackedData,
};
use pumpkin_protocol::{
    codec::{item_stack_seralizer::ItemStackSerializer, optional_int::OptionalInt},
    java::client::play::Metadata,
};
use pumpkin_util::{
    math::vector3::Vector3,
    random::{RandomGenerator, RandomImpl, get_seed, xoroshiro128::Xoroshiro},
};
use std::sync::atomic::AtomicBool;
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

const GRAVITY: f64 = 0.0;

pub struct FireworkRocketEntity {
    entity: ThrownItemEntity,
    item_stack: ItemStack,
    life: AtomicU32,
    life_time: AtomicU32,
}

impl FireworkRocketEntity {
    fn lifetime(item_stack: &ItemStack, random: &mut RandomGenerator) -> u32 {
        let flight_count = item_stack
            .get_data_component::<FireworksImpl>()
            .map_or(1, |fireworks| 1 + fireworks.flight_duration)
            .max(1) as u32;

        10 * flight_count + random.next_bounded_i32(6) as u32 + random.next_bounded_i32(7) as u32
    }

    pub fn new(entity: Entity, item_stack: ItemStack) -> Self {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));

        entity.set_velocity(Vector3::new(
            random.next_triangular(0.0, 0.002_297),
            0.05,
            random.next_triangular(0.0, 0.002_297),
        ));
        let life_time = Self::lifetime(&item_stack, &mut random);
        Self {
            entity: ThrownItemEntity {
                entity,
                owner_id: None,
                collides_with_projectiles: false,
                has_hit: AtomicBool::new(false),
                gravity: GRAVITY,
            },
            item_stack,
            life: 0.into(),
            life_time: life_time.into(),
        }
    }

    pub fn new_shot(entity: Entity, shooter: &Entity, item_stack: ItemStack) -> Self {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));

        // Set random initial velocity
        // Set on the inner entity after constructing ThrownItemEntity
        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        thrown.entity.set_velocity(Vector3::new(
            random.next_triangular(0.0, 0.002_297),
            0.05,
            random.next_triangular(0.0, 0.002_297),
        ));
        let life_time = Self::lifetime(&item_stack, &mut random);

        Self {
            entity: thrown,
            item_stack,
            life: 0.into(),
            life_time: life_time.into(),
        }
    }

    pub async fn explode_and_remove(&self, world: &World) {
        let entity = self.get_entity();
        world.send_entity_status(entity, EntityStatus::FireworksExplode);

        // TODO: Explode/colors

        entity.remove().await;
    }

    fn free_flight_velocity(velocity: Vector3<f64>, horizontal_collision: bool) -> Vector3<f64> {
        let horizontal_acceleration = if horizontal_collision { 1.0 } else { 1.15 };
        velocity
            .multiply(horizontal_acceleration, 1.0, horizontal_acceleration)
            .add_raw(0.0, 0.04, 0.0)
    }

    fn has_explosion(&self) -> bool {
        self.item_stack
            .get_data_component::<FireworksImpl>()
            .is_some_and(|fireworks| !fireworks.explosions.is_empty())
    }
}

impl NBTStorage for FireworkRocketEntity {}

impl EntityBase for FireworkRocketEntity {
    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::ID_FIREWORKS_ITEM,
                    MetaDataType::ITEM_STACK,
                    &ItemStackSerializer::from(self.item_stack.clone()),
                )],
                None,
            );

            if let Some(shooter_id) = self.entity.owner_id {
                entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::ATTACHED_TO_TARGET,
                        MetaDataType::OPTIONAL_LIVING_ENTITY_REFERENCE,
                        OptionalInt(Some(shooter_id)),
                    )],
                    None,
                );
            }
        })
    }

    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        _server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let world = entity.world.load();
            entity.update_last_pos();

            if let Some(shooter_id) = self.entity.owner_id {
                if let Some(shooter) = world.get_entity_by_id(shooter_id) {
                    let shooter = shooter.get_entity();

                    if shooter.is_fall_flying() {
                        let rotation = shooter.rotation().to_f64();
                        let shooter_vel = shooter.velocity.load();

                        let new_shooter_vel =
                            shooter_vel + (rotation * 0.1 + (rotation * 1.5 - shooter_vel) * 0.5);

                        shooter.set_velocity(new_shooter_vel);
                    }

                    entity.set_pos(shooter.pos.load());
                    entity.set_velocity(shooter.velocity.load());
                }
            } else {
                let velocity = Self::free_flight_velocity(
                    entity.velocity.load(),
                    entity.horizontal_collision.load(Ordering::Relaxed),
                );
                let start_position = entity.pos.load();
                entity.move_entity(caller, velocity).await;
                entity.set_velocity(velocity);

                let actual_movement = entity.pos.load() - start_position;
                let collided = (actual_movement - velocity).length_squared() > 1.0e-12;
                if collided && self.has_explosion() {
                    self.explode_and_remove(&world).await;
                    return;
                }
            }

            // Increment life and check for explosion
            let current_life = self.life.fetch_add(1, Ordering::Relaxed) + 1;
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

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_rocket_accelerates_before_moving() {
        let velocity =
            FireworkRocketEntity::free_flight_velocity(Vector3::new(0.1, 0.05, -0.2), false);

        assert!((velocity.x - 0.115).abs() < f64::EPSILON);
        assert!((velocity.y - 0.09).abs() < f64::EPSILON);
        assert!((velocity.z + 0.23).abs() < f64::EPSILON);
    }

    #[test]
    fn horizontal_collision_suppresses_horizontal_acceleration() {
        let velocity =
            FireworkRocketEntity::free_flight_velocity(Vector3::new(0.1, 0.05, -0.2), true);

        assert_eq!(velocity, Vector3::new(0.1, 0.09, -0.2));
    }
}
