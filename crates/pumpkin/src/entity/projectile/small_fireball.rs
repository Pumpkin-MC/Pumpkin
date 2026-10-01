use std::sync::atomic::{AtomicBool, Ordering};

use pumpkin_util::math::vector3::Vector3;

use crate::{
    entity::{
        Entity, EntityBase,
        projectile::{ProjectileHit, ThrownItemEntity},
    },
    server::Server,
};

const GRAVITY: f64 = 0.0;
pub const INITIAL_ACCELERATION_POWER: f64 = 0.1;
pub const AIR_INERTIA: f64 = 0.95;
pub const WATER_INERTIA: f64 = 0.8;

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
            has_left_owner: AtomicBool::new(true),
            gravity: GRAVITY,
        };

        Self { thrown }
    }

    #[must_use]
    pub fn new_shot(entity: Entity, shooter: &Entity, direction: Vector3<f64>) -> Self {
        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        let accel = INITIAL_ACCELERATION_POWER;
        thrown
            .entity
            .velocity
            .store(direction.normalize().multiply(accel, accel, accel));
        Self { thrown }
    }
}

impl EntityBase for SmallFireballEntity {
    fn get_owner_id(&self) -> Option<i32> {
        self.thrown.owner_id
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        let entity = self.get_entity();
        let mut velocity = entity.velocity.load();

        let inertia = if entity.touching_water.load(Ordering::Relaxed) {
            WATER_INERTIA
        } else {
            AIR_INERTIA
        };

        let speed = velocity.length();
        if speed > 1e-6 {
            let norm = velocity.normalize();
            velocity = norm
                .multiply(
                    INITIAL_ACCELERATION_POWER,
                    INITIAL_ACCELERATION_POWER,
                    INITIAL_ACCELERATION_POWER,
                )
                .add(&velocity)
                .multiply(inertia, inertia, inertia);
            entity.velocity.store(velocity);
        }

        self.thrown.process_move_and_collision(caller);
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

    fn on_hit(&self, hit: ProjectileHit) {
        match hit {
            ProjectileHit::Entity {
                ref entity,
                hit_pos,
                ..
            } => {
                let world = self.get_entity().world.load();
                let shooter = self
                    .thrown
                    .owner_id
                    .and_then(|id| world.get_entity_by_id(id));

                let target_ent = entity.get_entity();
                let prior_fire_ticks = target_ent.fire_ticks.load(Ordering::Relaxed);
                if !target_ent.is_invulnerable_to(&pumpkin_data::damage::DamageType::FIREBALL) {
                    target_ent.set_on_fire_for(5.0);
                }

                let damaged = entity.damage_with_context(
                    entity.as_ref(),
                    5.0,
                    pumpkin_data::damage::DamageType::FIREBALL,
                    Some(hit_pos),
                    Some(self),
                    shooter.as_deref(),
                );
                if !damaged {
                    target_ent
                        .fire_ticks
                        .store(prior_fire_ticks, Ordering::Relaxed);
                }
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
                world.set_block_state(
                    &block_to_place,
                    fire_state,
                    pumpkin_world::world::BlockFlags::NOTIFY_ALL,
                );
            }
        }
    }
}
