use std::sync::Arc;

pub use pumpkin_data::enchantment::{PositionSource, PositionSourceType, VelocitySource};
use pumpkin_data::particle::Particle;
use pumpkin_util::math::float_provider::FloatProvider;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomGenerator, get_seed, xoroshiro128::Xoroshiro};

use super::EnchantmentEntityEffectExt;
use crate::entity::Entity;
use crate::entity::player::Player;
use crate::world::World;

/// Enchantment entity effect that spawns particles.
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnParticlesEffect {
    pub particle: Particle,
    pub horizontal_position: PositionSource,
    pub vertical_position: PositionSource,
    pub horizontal_velocity: VelocitySource,
    pub vertical_velocity: VelocitySource,
    pub speed: FloatProvider,
}

impl SpawnParticlesEffect {
    #[must_use]
    pub const fn new(
        particle: Particle,
        horizontal_position: PositionSource,
        vertical_position: PositionSource,
        horizontal_velocity: VelocitySource,
        vertical_velocity: VelocitySource,
        speed: FloatProvider,
    ) -> Self {
        Self {
            particle,
            horizontal_position,
            vertical_position,
            horizontal_velocity,
            vertical_velocity,
            speed,
        }
    }

    pub fn apply(&self, world: &Arc<World>, position: Vector3<f64>, entity: Option<&Entity>) {
        let (bb_width, bb_height, movement) = entity.map_or_else(
            || (0.6f32, 1.8f32, Vector3::new(0.0, 0.0, 0.0)),
            |entity| {
                let bb = entity.bounding_box.load();
                let width = (bb.max.x - bb.min.x) as f32;
                let height = (bb.max.y - bb.min.y) as f32;
                let vel = entity.velocity.load();
                (width, height, vel)
            },
        );

        let mut random_gen = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));

        let x = self.horizontal_position.get_coordinate(
            position.x,
            position.x,
            bb_width,
            &mut random_gen,
        );
        let y = self.vertical_position.get_coordinate(
            position.y,
            position.y + f64::from(bb_height) / 2.0,
            bb_height,
            &mut random_gen,
        );
        let z = self.horizontal_position.get_coordinate(
            position.z,
            position.z,
            bb_width,
            &mut random_gen,
        );

        let vx = self
            .horizontal_velocity
            .get_velocity(movement.x, &mut random_gen);
        let vy = self
            .vertical_velocity
            .get_velocity(movement.y, &mut random_gen);
        let vz = self
            .horizontal_velocity
            .get_velocity(movement.z, &mut random_gen);
        let speed = self.speed.get(&mut random_gen);

        world.spawn_particle(
            Vector3::new(x, y, z),
            Vector3::new(vx as f32, vy as f32, vz as f32),
            speed,
            0,
            self.particle,
        );
    }
}

impl EnchantmentEntityEffectExt for SpawnParticlesEffect {
    fn apply(
        &self,
        world: &Arc<World>,
        _enchantment_level: i32,
        _owner: Option<&Arc<Player>>,
        entity: Option<&Entity>,
        position: Vector3<f64>,
    ) {
        self.apply(world, position, entity);
    }
}
