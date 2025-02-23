use std::{
    f32::{self},
    sync::Arc,
};

use async_trait::async_trait;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::item::Item;
use pumpkin_util::math::vector3::Vector3;

use crate::server::Server;

use super::{Entity, EntityBase, living::LivingEntity, player::Player};

pub struct ThrownItemEntity {
    entity: Entity,
    // TODO: remove this for onCollisionWithBlock function
    time_to_destruct: u32,
    last_tick_time: AtomicCell<u32>,
    item: Item,
}

impl ThrownItemEntity {
    pub fn new(entity: Entity, owner: &Entity, time_to_destruct: u32, item: Item) -> Self {
        let mut owner_pos = owner.pos.load();
        owner_pos.y = (owner_pos.y + f64::from(owner.standing_eye_height)) - 0.1;
        entity.pos.store(owner_pos);
        Self {
            entity,
            time_to_destruct,
            last_tick_time: AtomicCell::new(0),
            item,
        }
    }

    pub fn set_velocity_from(
        &self,
        shooter: &Entity,
        pitch: f32,
        yaw: f32,
        roll: f32,
        speed: f32,
        divergence: f32,
    ) {
        // Convert 3 degree of freedom to radians
        let yaw_rad = yaw.to_radians();
        let pitch_rad = pitch.to_radians();
        let roll_rad = (pitch + roll).to_radians();

        // Slight optimization, store cos of pitch, reduces nondeterministic trig
        let pitch_rad_cos = pitch_rad.cos();

        // The player is oriented -90 degrees on the yaw and pitch is -90 to 90
        let x = -yaw_rad.sin() * pitch_rad_cos;
        let y = -roll_rad.sin();
        let z = yaw_rad.cos() * pitch_rad_cos;
        self.set_velocity(
            f64::from(x),
            f64::from(y),
            f64::from(z),
            f64::from(speed),
            f64::from(divergence),
        );

        // Add player velocity to velocity
        let shooter_vel = shooter.velocity.load();
        self.entity
            .velocity
            .store(self.entity.velocity.load().add_raw(
                shooter_vel.x,
                if shooter.on_ground.load(std::sync::atomic::Ordering::Relaxed) {
                    0.0
                } else {
                    shooter_vel.y
                },
                shooter_vel.z,
            ));
    }

    /// The velocity and rotation will be set to the same direction.
    pub fn set_velocity(&self, x: f64, y: f64, z: f64, power: f64, uncertainty: f64) {
        fn next_triangular(mode: f64, deviation: f64) -> f64 {
            mode + deviation * (rand::random::<f64>() - rand::random::<f64>())
        }
        let velocity = Vector3::new(x, y, z)
            .normalize()
            .add_raw(
                next_triangular(0.0, 0.017_227_5 * uncertainty),
                next_triangular(0.0, 0.017_227_5 * uncertainty),
                next_triangular(0.0, 0.017_227_5 * uncertainty),
            )
            .multiply(power, power, power);
        self.entity.velocity.store(velocity);
        let len = velocity.horizontal_length();
        self.entity.set_rotation(
            velocity.x.atan2(velocity.z) as f32 * 57.295_776,
            velocity.y.atan2(len) as f32 * 57.295_776,
        );
    }

    pub fn set_last_time(&self, new_time: u32) {
        self.last_tick_time.store(new_time);
    }
}

#[async_trait]
impl EntityBase for ThrownItemEntity {
    async fn tick(&self, server: &Server) {
        // Gravity for thrown potion, projectile is 0.03;
        let gravity = Vector3::new(0.0, -0.05, 0.0);

        // Moves throwable to next location
        let previous_velocity = self.entity.velocity.load();
        let previous_position = self.entity.pos.load();

        let new_velocity = previous_velocity.add(&gravity).multiply(0.99, 0.99, 0.99);
        let new_position = previous_position.add(&new_velocity);

        self.entity.velocity.store(new_velocity);
        self.entity.set_pos(new_position);

        // Calculate time
        self.set_last_time(self.last_tick_time.load() + 1);
        if self.last_tick_time.load() > self.time_to_destruct {
            let world = self.entity.world.read().await;
            server
                .item_registry
                .on_entity_destroy(&self.item, &self.entity, &world)
                .await;
            self.entity.remove().await;
            // self.entity.world.get_mut().;
        }

        // TODO: this should be replaced with function to determine if entity touched ground
        if new_velocity.x.abs() < 0.1 && new_velocity.z.abs() < 0.1 {
            self.entity.remove().await;
        }

        // TODO: need to check if projectile hit a target in between movements
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    async fn on_player_collision(&self, player: Arc<Player>) {
        // Only damage player if thrower is not the collision
        // TODO: does this mean snowballs thrown upwards should do nothing?
        // TODO: Add on_destroy function here
        if let Some(id) = self.entity.owner_id {
            if player.entity_id() != id {
                player.damage(1).await;
                self.entity.remove().await;
            }
        }
    }
}
