use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NBTStorage,
        projectile::{ProjectileHit, ThrownItemEntity},
    },
    server::Server,
    world::World,
};
use pumpkin_data::{
    damage::DamageType, data_component_impl::FireworksImpl, entity::EntityStatus, item::Item,
    item_stack::ItemStack, meta_data_type::MetaDataType, sound::Sound, sound::SoundCategory,
    tracked_data::TrackedData,
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
    pub entity: ThrownItemEntity,
    item_stack: ItemStack,
    life: AtomicI32,
    life_time: AtomicI32,
    /// Vanilla `DATA_SHOT_AT_ANGLE`: true for a rocket fired from a crossbow, or a
    /// dispenser-fired-at-a-non-default-angle rocket (dispenser wiring for fireworks is
    /// out of scope here). Either way it skips the normal self-propelled acceleration
    /// branch entirely and flies a plain ballistic arc.
    shot_at_angle: AtomicBool,
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
            shot_at_angle: AtomicBool::new(false),
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
            shot_at_angle: AtomicBool::new(false),
        };

        // Set shooter metadata
        rocket.entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::ATTACHED_TO_TARGET,
                MetaDataType::OPTIONAL_UNSIGNED_INT,
                OptionalInt(Some(shooter.entity_id)),
            )],
            None,
        );

        rocket
    }

    /// Sets this rocket's velocity from a shot direction, matching the same shot-vector
    /// math `ArrowEntity::set_velocity_from_rotation` uses.
    pub fn set_shot_velocity(
        &self,
        shooter: &Entity,
        pitch: f32,
        yaw: f32,
        roll: f32,
        speed: f32,
        divergence: f32,
    ) {
        self.entity
            .set_velocity_from(shooter, pitch, yaw, roll, speed, divergence);
    }

    /// Vanilla `CrossbowItem#createProjectile`'s firework branch: `new FireworkRocketEntity(
    /// level, projectile, shooter, x, y, z, true)`. Unlike `new_shot_with_item` (the
    /// elytra-boost throw), this does not send `ATTACHED_TO_TARGET` - a crossbow-fired
    /// rocket is a one-shot projectile, not a rendering-follow boost effect - and it flags
    /// `shot_at_angle` so `tick` gives it a normal ballistic arc.
    pub fn new_crossbow_shot(entity: Entity, shooter: &Entity, item_stack: &ItemStack) -> Self {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        Self {
            entity: thrown,
            item_stack: item_stack.clone(),
            life: 0.into(),
            life_time: firework_lifetime(
                flight_duration(item_stack),
                random.next_bounded_i32(6),
                random.next_bounded_i32(7),
            )
            .into(),
            shot_at_angle: AtomicBool::new(true),
        }
    }

    pub async fn explode_and_remove(&self, world: &Arc<World>) {
        let entity = self.get_entity();
        world.send_entity_status(entity, EntityStatus::FireworksExplode);

        let explosion_count = self
            .item_stack
            .get_data_component::<FireworksImpl>()
            .map_or(0, |fireworks| fireworks.explosions.len());
        if explosion_count > 0 {
            let damage = 5.0 + (explosion_count as f32 * 2.0);
            let rocket_pos = entity.pos.load();
            // Vanilla's flat, un-falloff damage and the loop-exclusion below apply only to
            // `attachedToEntity` - the player holding the rocket during an elytra boost. A
            // crossbow- or dispenser-fired rocket (`shot_at_angle`) never sets that field, so
            // its `owner` is a normal target: it takes falloff damage like anyone else, not
            // this exemption.
            let attached_to_entity = (!self.shot_at_angle.load(Ordering::Relaxed))
                .then_some(self.entity.owner_id)
                .flatten();
            if let Some(owner_id) = attached_to_entity
                && let Some(owner) = world.get_entity_by_id(owner_id)
            {
                owner.damage(self, damage, DamageType::FIREWORKS).await;
            }
            let targets = world.get_all_at_box(&entity.bounding_box.load().expand(5.0, 5.0, 5.0));

            for target in targets {
                let target_entity = target.get_entity();
                if !target_entity.is_alive()
                    || attached_to_entity
                        .is_some_and(|owner_id| owner_id == target_entity.entity_id)
                {
                    continue;
                }

                let distance = rocket_pos
                    .squared_distance_to_vec(&target_entity.pos.load())
                    .sqrt();
                let Some(amount) = firework_damage(damage, distance) else {
                    continue;
                };

                // Vanilla `dealExplosionDamage`: tests line-of-sight to the target twice, at
                // `getY(0)` (feet) and `getY(0.5)` (bounding-box midpoint), taking the first
                // clear one - not a single eye-height raycast.
                let target_pos = target_entity.pos.load();
                let target_bb = target_entity.bounding_box.load();
                let mid_y = target_bb.min.y + (target_bb.max.y - target_bb.min.y) * 0.5;
                let mut can_see = false;
                for test_y in [target_bb.min.y, mid_y] {
                    let to = Vector3::new(target_pos.x, test_y, target_pos.z);
                    if world
                        .raycast(rocket_pos, to, async |block_pos, world| {
                            world.get_block_state(block_pos).is_solid()
                        })
                        .await
                        .is_none()
                    {
                        can_see = true;
                        break;
                    }
                }
                if !can_see {
                    continue;
                }
                target.damage(self, amount, DamageType::FIREWORKS).await;
            }
        }

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

fn has_explosion(item_stack: &ItemStack) -> bool {
    item_stack
        .get_data_component::<FireworksImpl>()
        .is_some_and(|fireworks| !fireworks.explosions.is_empty())
}

fn firework_damage(base_damage: f32, distance: f64) -> Option<f32> {
    if !(0.0..=5.0).contains(&distance) {
        return None;
    }
    Some(base_damage * (((5.0 - distance) / 5.0).sqrt() as f32))
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

            let boosting_elytra_owner = self
                .entity
                .owner_id
                .and_then(|shooter_id| world.get_entity_by_id(shooter_id))
                .filter(|shooter| shooter.get_entity().is_fall_flying());

            if let Some(shooter) = boosting_elytra_owner {
                // Logic for boosting Elytra flight
                let shooter = shooter.get_entity();
                let rotation = shooter.rotation().to_f64();
                let shooter_vel = shooter.velocity.load();

                let new_shooter_vel =
                    shooter_vel + (rotation * 0.1 + (rotation * 1.5 - shooter_vel) * 0.5);

                shooter.set_velocity(new_shooter_vel);

                entity.set_pos(shooter.pos.load());
                entity.set_velocity(new_shooter_vel);
            } else if !self.shot_at_angle.load(Ordering::Relaxed) {
                // Standard firework rocket flight logic: not applied to a crossbow- or
                // dispenser-fired-at-angle rocket (`shot_at_angle`), which instead flies a
                // normal ballistic arc. Vanilla: `horizontalAcceleration = horizontalCollision
                // ? 1.0 : 1.15`.
                let horizontal_acceleration = if entity.horizontal_collision.load(Ordering::Relaxed)
                {
                    1.0
                } else {
                    1.15
                };
                velocity.x *= horizontal_acceleration;
                velocity.z *= horizontal_acceleration;
                velocity.y += 0.04;
                entity.set_velocity(velocity);
            }

            // Vanilla: `if (this.life == 0 && !this.isSilent()) { playSound(FIREWORK_ROCKET_LAUNCH...) }`,
            // called before `this.life++`. Pumpkin entities have no `isSilent` flag yet, so the
            // silence check is not modelled. The client-side `FIREWORK` particle trail
            // (`this.level().isClientSide()`-guarded in vanilla) is intentionally not ported: it
            // is generated by the client itself, and spawning it server-side would double it.
            if self.life.load(Ordering::Relaxed) == 0 {
                let pos = entity.pos.load();
                world.play_sound_raw(
                    Sound::EntityFireworkRocketLaunch as u16,
                    SoundCategory::Ambient,
                    &pos,
                    3.0,
                    1.0,
                );
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

    fn on_hit(&self, hit: ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let should_explode = match hit {
                ProjectileHit::Entity { .. } => true,
                ProjectileHit::Block { .. } => has_explosion(&self.item_stack),
            };
            if should_explode {
                let world = self.get_entity().world.load_full();
                self.explode_and_remove(&world).await;
            }
        })
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
    use super::{firework_damage, firework_lifetime};

    #[test]
    fn explosion_damage_falls_off_at_radius() {
        assert_eq!(firework_damage(7.0, 0.0), Some(7.0));
        assert_eq!(firework_damage(7.0, 5.0), Some(0.0));
        assert_eq!(firework_damage(7.0, 5.1), None);
    }

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
