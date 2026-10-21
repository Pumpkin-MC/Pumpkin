mod block_effects;
mod dismount;
mod passengers;
mod physics;

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crossbeam::atomic::AtomicCell;

use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase, living::LivingEntity};
use crate::server::Server;

use pumpkin_data::damage::DamageType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag;
use pumpkin_protocol::java::client::play::Metadata;

use pumpkin_util::math::vector3::Vector3;

use crate::entity::vehicle::vehicle::VehicleEntity;
use block_effects::BoatMovement;
use physics::BoatStatus;

const TIME_TO_EJECT: f32 = 60.0;

pub struct BoatEntity {
    pub vehicle: VehicleEntity,
    ticks_underwater: AtomicCell<f32>,
    status: AtomicCell<Option<BoatStatus>>,
    water_level: AtomicCell<f64>,
    land_friction: AtomicCell<f32>,
    last_yd: AtomicCell<f64>,
    movements: Mutex<VecDeque<BoatMovement>>,
    left_paddle_moving: AtomicBool,
    right_paddle_moving: AtomicBool,
}

impl BoatEntity {
    pub const fn new(entity: Entity) -> Self {
        Self {
            vehicle: VehicleEntity::new(entity),
            ticks_underwater: AtomicCell::new(0.0),
            status: AtomicCell::new(None),
            water_level: AtomicCell::new(0.0),
            land_friction: AtomicCell::new(0.0),
            last_yd: AtomicCell::new(0.0),
            movements: Mutex::new(VecDeque::new()),
            left_paddle_moving: AtomicBool::new(false),
            right_paddle_moving: AtomicBool::new(false),
        }
    }

    pub fn set_paddles(&self, left: bool, right: bool) {
        self.left_paddle_moving.store(left, Ordering::Relaxed);
        self.right_paddle_moving.store(right, Ordering::Relaxed);

        self.vehicle.entity.send_meta_data(
            &[
                Metadata::new(pumpkin_data::tracked_data::boat::ID_PADDLE_LEFT, left),
                Metadata::new(pumpkin_data::tracked_data::boat::ID_PADDLE_RIGHT, right),
            ],
            None,
        );
    }

    fn send_wobble_metadata(&self) {
        self.vehicle.send_wobble_metadata();
    }

    fn tick_underwater(&self, status: BoatStatus) {
        let ticks = if status.is_underwater() {
            self.ticks_underwater.load() + 1.0
        } else {
            0.0
        };
        self.ticks_underwater.store(ticks);
        if ticks < TIME_TO_EJECT {
            return;
        }

        let passengers = self
            .vehicle
            .entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for passenger in passengers.iter().rev() {
            self.vehicle
                .entity
                .remove_passenger(passenger.get_entity().entity_id);
        }
    }

    fn is_client_controlled(&self) -> bool {
        self.vehicle
            .entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .first()
            .is_some_and(|passenger| passenger.get_player().is_some())
    }

    fn get_max_passengers(&self) -> usize {
        if self.is_chest_boat() { 1 } else { 2 }
    }

    fn is_chest_boat(&self) -> bool {
        tag::Item::MINECRAFT_CHEST_BOATS
            .0
            .contains(&self.vehicle.entity.entity_type.resource_name)
    }

    fn tick_passengers(&self) {
        let entity = &self.vehicle.entity;
        let passengers = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        for passenger in passengers {
            if let Some(player) = passenger.get_player() {
                player.ride_tick();
            }
            if entity.has_passenger(passenger.get_entity().entity_id) {
                self.position_rider(passenger.as_ref());
            }
        }
    }

    pub(in crate::entity) fn can_push(&self, other: &Entity, is_boat: bool) -> bool {
        let bounds = self.vehicle.entity.bounding_box.load();
        let other_bottom = other.bounding_box.load().min.y;
        if is_boat {
            other_bottom < bounds.max.y
        } else {
            other_bottom <= bounds.min.y
        }
    }
}

impl EntityBase for BoatEntity {
    fn get_entity(&self) -> &Entity {
        &self.vehicle.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        let status = self.get_status();
        let old_status = self.status.swap(Some(status));
        self.tick_underwater(status);
        self.vehicle.tick();
        let entity = &self.vehicle.entity;
        entity.tick(caller, server);
        if entity.is_removed() {
            return;
        }

        // Vanilla leaves ridden boat movement to its controlling player's client.
        if self.is_client_controlled() {
            entity.velocity.store(Vector3::default());
        } else {
            if self.left_paddle_moving.load(Ordering::Relaxed)
                || self.right_paddle_moving.load(Ordering::Relaxed)
            {
                self.set_paddles(false, false);
            }
            let surface_offset = self.float_boat(old_status, status);
            self.move_boat(caller, surface_offset);
        }
        self.apply_effects_from_blocks();
        entity.tick_block_collisions(caller);
        self.tick_passengers();
    }

    fn init_data_tracker(&self) {
        self.send_wobble_metadata();
    }

    fn can_hit(&self) -> bool {
        self.vehicle.entity.is_alive()
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        true
    }

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        self.vehicle.damage_with_context(amount, source)
    }

    fn interact(&self, player: &Arc<Player>, _item_stack: &mut ItemStack) -> bool {
        if player.get_entity().is_sneaking() {
            return false;
        }

        if self.ticks_underwater.load() >= TIME_TO_EJECT || self.is_eye_in_water() {
            return false;
        }

        if self
            .vehicle
            .entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
            >= self.get_max_passengers()
        {
            return false;
        }

        if player.get_entity().has_vehicle() {
            return false;
        }

        let world = self.vehicle.entity.world.load();
        let Some(vehicle) = world.get_entity_by_id(self.vehicle.entity.entity_id) else {
            return false;
        };

        let Some(passenger) = world.get_player_by_id(player.entity_id()) else {
            return false;
        };

        self.vehicle
            .entity
            .add_passenger(vehicle, passenger as Arc<dyn EntityBase>);

        self.vehicle.entity.has_passenger(player.entity_id())
    }

    fn set_paddle_state(&self, left: bool, right: bool) {
        self.set_paddles(left, right);
    }
    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn is_pushable(&self) -> bool {
        true
    }

    fn get_gravity(&self) -> f64 {
        if self.vehicle.entity.has_no_gravity() {
            0.0
        } else {
            physics::DEFAULT_GRAVITY
        }
    }
}
