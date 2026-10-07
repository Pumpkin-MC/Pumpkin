//! Vanilla `EntityAttachments` for riding: where a vehicle seats its passengers and
//! where a rider is held on its seat.

use pumpkin_util::math::vector3::Vector3;

use super::Entity;

/// Vanilla `EntityDimensions.scale` on attachments: the current size over the type's size.
// TODO: Babies with their own vanilla attachments (`withAttachments`) use the scaled adult points.
fn size_scale(entity: &Entity) -> (f64, f64) {
    let current = entity.entity_dimension.load();
    let [width, height] = entity.entity_type.dimension;
    let ratio = |current: f32, base: f32| {
        if base > 0.0 {
            f64::from(current / base)
        } else {
            1.0
        }
    };
    (ratio(current.width, width), ratio(current.height, height))
}

/// Vanilla `Vec3.yRot(-yaw)`: a point in the entity's frame turned with its body.
#[must_use]
pub fn rotate_by_yaw(point: Vector3<f64>, yaw: f32) -> Vector3<f64> {
    let (sin, cos) = f64::from(-yaw.to_radians()).sin_cos();
    Vector3::new(
        point.x.mul_add(cos, point.z * sin),
        point.y,
        point.z.mul_add(cos, -point.x * sin),
    )
}

/// The rider's index among the vehicle's passengers, 0 when not seated yet.
#[must_use]
pub fn passenger_index(vehicle: &Entity, passenger: &Entity) -> usize {
    vehicle
        .passengers
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .position(|p| p.get_entity().entity_id == passenger.entity_id)
        .unwrap_or(0)
}

/// Vanilla `getDefaultPassengerAttachmentPoint`: the seat for the rider's index.
#[must_use]
pub fn default_passenger_attachment(vehicle: &Entity, passenger: &Entity) -> Vector3<f64> {
    let points = vehicle.entity_type.passenger_attachments;
    let Some(last) = points.len().checked_sub(1) else {
        return Vector3::default();
    };
    let (xz, y) = size_scale(vehicle);
    let point = points[passenger_index(vehicle, passenger).min(last)].multiply(xz, y, xz);
    rotate_by_yaw(point, vehicle.yaw.load())
}

/// Vanilla `getVehicleAttachmentPoint`: where the rider is held on its seat.
#[must_use]
pub fn default_vehicle_attachment(passenger: &Entity) -> Vector3<f64> {
    let (xz, y) = size_scale(passenger);
    passenger.entity_type.vehicle_attachment.multiply(xz, y, xz)
}
