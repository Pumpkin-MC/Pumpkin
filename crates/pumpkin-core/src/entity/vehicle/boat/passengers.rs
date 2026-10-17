use pumpkin_data::entity::{EntityPose, EntityType};
use pumpkin_protocol::PositionFlag;
use pumpkin_util::math::{cos, sin, vector3::Vector3};

use super::BoatEntity;
use crate::entity::{EntityBase, player::Player};

// Avatar.DEFAULT_VEHICLE_ATTACHMENT is shared by standing and crouching players.
const DEFAULT_VEHICLE_ATTACHMENT: Vector3<f64> = Vector3::new(0.0, 0.6, 0.0);

impl BoatEntity {
    pub(in crate::entity) fn position_rider(&self, passenger: &dyn EntityBase) {
        if let Some(position) = self.rider_position(passenger) {
            passenger.get_entity().set_pos(position);
        }
    }

    pub(in crate::entity) fn synchronize_mount(&self, player: &Player) -> bool {
        let Some(position) = self.rider_position(player) else {
            return false;
        };
        if !player.request_teleport_with_flags(
            position,
            0.0,
            0.0,
            vec![PositionFlag::YRot, PositionFlag::XRot],
        ) {
            return false;
        }
        player.get_entity().velocity.store(Vector3::default());
        true
    }

    fn rider_position(&self, passenger: &dyn EntityBase) -> Option<Vector3<f64>> {
        passenger.get_player()?;
        let attachment = self.get_passenger_attachment_point(passenger)?;
        let vehicle_attachment = match passenger.get_entity().pose.load() {
            EntityPose::Standing | EntityPose::Crouching => DEFAULT_VEHICLE_ATTACHMENT,
            _ => Vector3::default(),
        };
        Some(self.vehicle.entity.pos.load() + attachment - vehicle_attachment)
    }

    fn get_passenger_attachment_point(&self, passenger: &dyn EntityBase) -> Option<Vector3<f64>> {
        let entity = &self.vehicle.entity;
        let passengers = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let index = passengers
            .iter()
            .position(|rider| rider.get_entity().entity_id == passenger.get_entity().entity_id)?;
        let offset = if passengers.len() > 1 {
            if index == 0 { 0.2f32 } else { -0.6f32 }
        } else {
            self.get_single_passenger_x_offset()
        };
        drop(passengers);
        let rotation = -entity.yaw.load().to_radians();
        Some(Vector3::new(
            f64::from(offset) * f64::from(sin(rotation)),
            self.ride_height(),
            f64::from(offset) * f64::from(cos(rotation)),
        ))
    }

    fn get_single_passenger_x_offset(&self) -> f32 {
        if self.is_chest_boat() { 0.15 } else { 0.0 }
    }

    fn ride_height(&self) -> f64 {
        let entity = &self.vehicle.entity;
        if entity.entity_type == &EntityType::BAMBOO_RAFT
            || entity.entity_type == &EntityType::BAMBOO_CHEST_RAFT
        {
            f64::from(entity.height() * 0.888_888_9)
        } else {
            f64::from(entity.height() / 3.0)
        }
    }
}
