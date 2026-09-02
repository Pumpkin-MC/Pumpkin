use crate::{
    entity::{Entity, EntityBase, projectile::ThrownItemEntity},
    server::Server,
    world::World,
};
use pumpkin_data::entity::EntityStatus;
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use pumpkin_protocol::{codec::optional_int::OptionalInt, java::client::play::Metadata};
use pumpkin_util::{
    math::vector3::Vector3,
    random::{RandomGenerator, RandomImpl, get_seed, xoroshiro128::Xoroshiro},
};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

const GRAVITY: f64 = 0.0;

pub struct FireworkRocketEntity {
    entity: ThrownItemEntity,
    life: AtomicU32,
    life_time: AtomicU32,
}

impl FireworkRocketEntity {
    pub fn new(entity: Entity) -> Self {
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
            life: 0.into(),
            life_time: (10 + random.next_bounded_i32(6) as u32 + random.next_bounded_i32(7) as u32)
                .into(),
        }
    }

    pub fn new_shot(entity: Entity, shooter: &Entity) -> Self {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));

        let thrown = ThrownItemEntity::new(entity, shooter, GRAVITY);
        thrown.entity.set_velocity(Vector3::new(
            random.next_triangular(0.0, 0.002_297),
            0.05,
            random.next_triangular(0.0, 0.002_297),
        ));

        let rocket = Self {
            entity: thrown,
            life: 0.into(),
            life_time: (10 + random.next_bounded_i32(6) as u32 + random.next_bounded_i32(7) as u32)
                .into(),
        };

        rocket.entity.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::firework_rocket::ATTACHED_TO_TARGET,
                OptionalInt(Some(shooter.entity_id)),
            )],
            None,
        );

        rocket
    }

    pub fn explode_and_remove(&self, world: &World) {
        let entity = self.get_entity();
        world.send_entity_status(
            entity,
            EntityStatus::FireworksExplode,
            Some(ActorEventID::FireworksExplode),
        );

        entity.remove();
    }
}

impl EntityBase for FireworkRocketEntity {
    fn get_owner_id(&self) -> Option<i32> {
        self.entity.owner_id
    }

    fn tick(&self, caller: &dyn EntityBase, _server: &Server) {
        self.entity.process_tick(caller);

        let entity = self.get_entity();
        let world = entity.world.load();
        let mut velocity = entity.velocity.load();

        if let Some(shooter_id) = self.entity.owner_id {
            if let Some(shooter) = world.get_entity_by_id(shooter_id) {
                let shooter = shooter.get_entity();

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
            velocity.x *= 1.15;
            velocity.z *= 1.15;
            velocity.y += 0.04;
            entity.set_velocity(velocity);
        }

        let current_life = self.life.fetch_add(1, Ordering::Relaxed);
        if current_life > self.life_time.load(Ordering::Relaxed) {
            self.explode_and_remove(&world);
        }
    }

    fn get_entity(&self) -> &crate::entity::Entity {
        &self.entity.entity
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn on_hit(&self, _hit: crate::entity::projectile::ProjectileHit) {
        let world = self.get_entity().world.load();
        self.explode_and_remove(&world);
    }
}
