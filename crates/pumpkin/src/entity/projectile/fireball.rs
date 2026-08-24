use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::RwLock;

use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::atomic_f32::AtomicF32;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NbtFuture,
        projectile::{ProjectileHit, ThrownItemEntity},
        projectile_deflection::ProjectileDeflectionType,
    },
    server::Server,
};

pub const MIN_CAMERA_DISTANCE_SQUARED: f32 = 12.25;
pub const INITIAL_ACCELERATION_POWER: f64 = 0.1;
pub const DEFLECTION_SCALE: f64 = 0.5;
pub const DEFAULT_EXPLOSION_POWER: f32 = 1.0;
pub const AIR_INERTIA: f64 = 0.95;
pub const WATER_INERTIA: f64 = 0.8;

pub struct FireballEntity {
    pub thrown: ThrownItemEntity,
    pub item_stack: RwLock<ItemStack>,
    pub acceleration_power: AtomicU64,
    pub explosion_power: AtomicF32,
}

impl FireballEntity {
    #[must_use]
    pub fn new(entity: Entity) -> Self {
        let thrown = ThrownItemEntity {
            entity,
            owner_id: None,
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
            gravity: 0.0,
        };

        Self {
            thrown,
            item_stack: RwLock::new(Self::get_default_item()),
            acceleration_power: AtomicU64::new(INITIAL_ACCELERATION_POWER.to_bits()),
            explosion_power: AtomicF32::new(DEFAULT_EXPLOSION_POWER),
        }
    }

    #[must_use]
    pub fn new_shot(entity: Entity, shooter: &Entity, direction: Vector3<f64>) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter, 0.0);
        let accel = INITIAL_ACCELERATION_POWER;
        let vel = direction.normalize().multiply(accel, accel, accel);
        thrown.entity.velocity.store(vel);

        Self {
            thrown,
            item_stack: RwLock::new(Self::get_default_item()),
            acceleration_power: AtomicU64::new(accel.to_bits()),
            explosion_power: AtomicF32::new(DEFAULT_EXPLOSION_POWER),
        }
    }

    #[must_use]
    pub fn new_directional(
        entity: Entity,
        direction: Vector3<f64>,
        acceleration_power: f64,
    ) -> Self {
        let thrown = ThrownItemEntity {
            entity,
            owner_id: None,
            collides_with_projectiles: false,
            has_hit: AtomicBool::new(false),
            gravity: 0.0,
        };
        let vel = direction.normalize().multiply(
            acceleration_power,
            acceleration_power,
            acceleration_power,
        );
        thrown.entity.velocity.store(vel);

        Self {
            thrown,
            item_stack: RwLock::new(Self::get_default_item()),
            acceleration_power: AtomicU64::new(acceleration_power.to_bits()),
            explosion_power: AtomicF32::new(DEFAULT_EXPLOSION_POWER),
        }
    }

    #[must_use]
    pub fn get_default_item() -> ItemStack {
        ItemStack::new(1, &Item::FIRE_CHARGE)
    }

    pub async fn get_item(&self) -> ItemStack {
        self.item_stack.read().await.clone()
    }

    pub async fn set_item(&self, source: ItemStack) {
        let new_item = if source.item_count == 0 {
            Self::get_default_item()
        } else {
            let mut item = source;
            item.item_count = 1;
            item
        };
        *self.item_stack.write().await = new_item.clone();

        self.get_entity().send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::fireball::ITEM_STACK,
                &ItemStackSerializer::from(new_item),
            )],
            None,
        );
    }

    pub fn get_acceleration_power(&self) -> f64 {
        f64::from_bits(self.acceleration_power.load(Ordering::Relaxed))
    }

    pub fn set_acceleration_power(&self, power: f64) {
        self.acceleration_power
            .store(power.to_bits(), Ordering::Relaxed);
    }

    pub fn get_explosion_power(&self) -> f32 {
        self.explosion_power.load(Ordering::Relaxed)
    }

    pub fn set_explosion_power(&self, power: f32) {
        self.explosion_power.store(power, Ordering::Relaxed);
    }

    pub fn on_deflection(&self, _deflection: &ProjectileDeflectionType, by_attack: bool) {
        if by_attack {
            self.set_acceleration_power(INITIAL_ACCELERATION_POWER);
        } else {
            let current = self.get_acceleration_power();
            self.set_acceleration_power(current * DEFLECTION_SCALE);
        }
    }

    pub fn should_render_at_sqr_distance(&self, distance_sqr: f64) -> bool {
        if self.get_entity().age.load(Ordering::Relaxed) < 2
            && distance_sqr < f64::from(MIN_CAMERA_DISTANCE_SQUARED)
        {
            false
        } else {
            let bb_size = self
                .get_entity()
                .bounding_box
                .load()
                .get_average_side_length()
                * 4.0;
            let size = if bb_size.is_nan() { 4.0 } else { bb_size } * 64.0;
            distance_sqr < size * size
        }
    }
}

impl EntityBase for FireballEntity {
    fn write_custom_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            nbt.put_double("acceleration_power", self.get_acceleration_power());
            nbt.put_float("ExplosionPower", self.get_explosion_power());
        })
    }

    fn read_custom_nbt<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            if let Some(accel) = nbt
                .get_double("acceleration_power")
                .or_else(|| nbt.get_double("power"))
            {
                self.set_acceleration_power(accel);
            }
            if let Some(exp) = nbt.get_float("ExplosionPower") {
                self.set_explosion_power(exp);
            }
        })
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let stack = self.item_stack.read().await;

            entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::fireball::ITEM_STACK,
                    &ItemStackSerializer::from(stack.clone()),
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
            let entity = self.get_entity();
            let mut velocity = entity.velocity.load();

            let inertia = if entity.touching_water.load(Ordering::Relaxed) {
                WATER_INERTIA
            } else {
                AIR_INERTIA
            };

            let accel = self.get_acceleration_power();
            let speed = velocity.length();
            if speed > 1e-6 {
                let norm = velocity.normalize();
                velocity = norm
                    .multiply(accel, accel, accel)
                    .add(&velocity)
                    .multiply(inertia, inertia, inertia);
                entity.velocity.store(velocity);
            }

            self.thrown.process_tick(caller, server).await;
        })
    }

    fn get_entity(&self) -> &Entity {
        self.thrown.get_entity()
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, hit: ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let world = self.get_entity().world.load();

            if let ProjectileHit::Entity { ref entity, .. } = hit {
                let entity_clone = entity.clone();

                tokio::spawn(async move {
                    entity_clone.get_entity().set_on_fire_for(5.0);
                    let _ = entity_clone
                        .damage(
                            entity_clone.as_ref(),
                            6.0,
                            pumpkin_data::damage::DamageType::FIREBALL,
                        )
                        .await;
                });
            }

            let hit_pos = hit.hit_pos();
            world
                .explode(
                    hit_pos,
                    self.get_explosion_power(),
                    crate::world::ExplosionInteraction::Mob,
                )
                .await;
        })
    }
}
