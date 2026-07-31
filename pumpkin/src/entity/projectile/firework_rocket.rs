use crate::{
    entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage, projectile::ThrownItemEntity},
    server::Server,
    world::World,
};
use pumpkin_data::{
    data_component_impl::FireworksImpl, entity::EntityStatus, item::Item, item_stack::ItemStack,
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
    atomic::{AtomicI32, Ordering},
};

const GRAVITY: f64 = 0.0;

pub struct FireworkRocketEntity {
    entity: ThrownItemEntity,
    item_stack: ItemStack,
    life: AtomicI32,
    life_time: AtomicI32,
}

impl FireworkRocketEntity {
    pub fn new(entity: Entity) -> Self {
        Self::new_with_item(entity, &ItemStack::new(1, &Item::FIREWORK_ROCKET))
    }

    pub fn new_with_item(entity: Entity, item_stack: &ItemStack) -> Self {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));

        entity.set_velocity(Vector3::new(
            random.next_triangular(0.0, 0.002_297),
            0.05,
            random.next_triangular(0.0, 0.002_297),
        ));
        Self {
            entity: ThrownItemEntity {
                entity,
                owner_id: None,
                collides_with_projectiles: false,
                has_hit: AtomicBool::new(false),
                gravity: GRAVITY,
            },
            item_stack: item_stack.clone(),
            life: 0.into(),
            life_time: firework_lifetime(
                flight_duration(item_stack),
                random.next_bounded_i32(6),
                random.next_bounded_i32(7),
            )
            .into(),
        }
    }

    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        Self::new_shot_with_item(entity, shooter, &ItemStack::new(1, &Item::FIREWORK_ROCKET))
    }

    pub fn new_shot_with_item(entity: Entity, shooter: &Entity, item_stack: &ItemStack) -> Self {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));

        // Set random initial velocity
        // Set on the inner entity after constructing ThrownItemEntity
        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        thrown.entity.set_velocity(Vector3::new(
            random.next_triangular(0.0, 0.002_297),
            0.05,
            random.next_triangular(0.0, 0.002_297),
        ));

        // Set random life
        let rocket = Self {
            entity: thrown,
            item_stack: item_stack.clone(),
            life: 0.into(),
            life_time: firework_lifetime(
                flight_duration(item_stack),
                random.next_bounded_i32(6),
                random.next_bounded_i32(7),
            )
            .into(),
        };

        // Set shooter metadata
        rocket.entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::ATTACHED_TO_TARGET,
                MetaDataType::OPTIONAL_INT,
                OptionalInt(Some(shooter.entity_id)),
            )],
            None,
        );

        rocket
    }

    pub async fn explode_and_remove(&self, world: &World) {
        let entity = self.get_entity();
        world.send_entity_status(entity, EntityStatus::FireworksExplode);

        // TODO: Explode/colors

        entity.remove().await;
    }
}

/// Matches `FireworkRocketEntity(Level, ..., ItemStack)`: vanilla adds one to the
/// data-component duration, then adds its two independent random lifetime offsets.
const fn firework_lifetime(flight_duration: i32, first_random: i32, second_random: i32) -> i32 {
    10i32
        .wrapping_mul(1i32.wrapping_add(flight_duration))
        .wrapping_add(first_random)
        .wrapping_add(second_random)
}

fn flight_duration(item_stack: &ItemStack) -> i32 {
    item_stack
        .get_data_component::<FireworksImpl>()
        .map_or(0, |fireworks| fireworks.flight_duration)
}

impl NBTStorage for FireworkRocketEntity {}

impl EntityBase for FireworkRocketEntity {
    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.get_entity().send_meta_data(
                &[Metadata::new(
                    TrackedData::ID_FIREWORKS_ITEM,
                    MetaDataType::ITEM_STACK,
                    &ItemStackSerializer::from(self.item_stack.clone()),
                )],
                None,
            );
        })
    }

    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.entity.process_tick(caller, server).await;

            let entity = self.get_entity();
            let world = entity.world.load();
            let mut velocity = entity.velocity.load();

            if let Some(shooter_id) = self.entity.owner_id {
                // Check if the player who fired this rocket still exists in the world
                if let Some(shooter) = world.get_entity_by_id(shooter_id) {
                    let shooter = shooter.get_entity();

                    // Logic for boosting Elytra flight
                    if shooter.is_fall_flying() {
                        let rotation = shooter.rotation().to_f64();
                        let shooter_vel = shooter.velocity.load();

                        let new_shooter_vel =
                            shooter_vel + (rotation * 0.1 + (rotation * 1.5 - shooter_vel) * 0.5);

                        shooter.set_velocity(new_shooter_vel);

                        entity.set_pos(shooter.pos.load());
                        entity.set_velocity(new_shooter_vel);
                    }
                }
            } else {
                // Standard firework rocket flight logic
                velocity.x *= 1.15;
                velocity.z *= 1.15;
                velocity.y += 0.04;
                entity.set_velocity(velocity);
            }

            // Increment life and check for explosion
            let current_life = self.life.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
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
    use super::firework_lifetime;

    #[test]
    fn lifetime_includes_default_flight_duration() {
        // The default Fireworks component has flight_duration 1, so vanilla starts
        // at 20 ticks before adding its two random offsets.
        assert_eq!(firework_lifetime(1, 0, 0), 20);
        assert_eq!(firework_lifetime(1, 5, 6), 31);
    }

    #[test]
    fn lifetime_scales_with_component_flight_duration() {
        assert_eq!(firework_lifetime(3, 0, 0), 40);
        assert_eq!(firework_lifetime(3, 5, 6), 51);
    }
}
