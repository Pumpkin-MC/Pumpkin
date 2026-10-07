//! Vanilla `EntityAttachments` for riding: where a vehicle seats its passengers and
//! where a rider is held on its seat.

use std::sync::atomic::Ordering::Relaxed;

use pumpkin_util::math::vector3::Vector3;

use super::Entity;

/// The type's size and attachment points for the entity's age: `[width, height]`,
/// passenger seats and the vehicle point.
fn age_attachments(entity: &Entity) -> ([f32; 2], &'static [Vector3<f64>], Vector3<f64>) {
    let entity_type = entity.entity_type;
    match &entity_type.baby {
        Some(baby) if entity.age.load(Relaxed) < 0 => (
            baby.dimension,
            baby.passenger_attachments,
            baby.vehicle_attachment,
        ),
        _ => (
            entity_type.dimension,
            entity_type.passenger_attachments,
            entity_type.vehicle_attachment,
        ),
    }
}

/// Vanilla `EntityDimensions.scale` on attachments: the current size over the age's size.
fn size_scale(entity: &Entity, [width, height]: [f32; 2]) -> (f64, f64) {
    let current = entity.entity_dimension.load();
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
    let (dimension, points, _) = age_attachments(vehicle);
    let Some(last) = points.len().checked_sub(1) else {
        return Vector3::default();
    };
    let (xz, y) = size_scale(vehicle, dimension);
    let point = points[passenger_index(vehicle, passenger).min(last)].multiply(xz, y, xz);
    rotate_by_yaw(point, vehicle.yaw.load())
}

/// Vanilla `getVehicleAttachmentPoint`: where the rider is held on its seat.
#[must_use]
pub fn default_vehicle_attachment(passenger: &Entity) -> Vector3<f64> {
    let (dimension, _, point) = age_attachments(passenger);
    let (xz, y) = size_scale(passenger, dimension);
    point.multiply(xz, y, xz)
}
