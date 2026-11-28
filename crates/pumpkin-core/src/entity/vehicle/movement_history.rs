use std::collections::VecDeque;

use pumpkin_util::math::vector3::Vector3;

const MAX_MOVEMENTS_PER_TICK: usize = 100;
pub const BLOCK_INTERSECTION_EPSILON: f32 = 1.0e-5;

#[derive(Clone, Copy)]
pub struct VehicleMovement {
    pub from: Vector3<f64>,
    pub to: Vector3<f64>,
    pub requested: Option<Vector3<f64>>,
}

#[derive(Default)]
pub struct MovementHistory {
    pub pending: VecDeque<VehicleMovement>,
    pub teleport_revision: u64,
}

impl MovementHistory {
    /// Returns whether the resolved movement is large enough to apply.
    pub fn record(
        &mut self,
        from: Vector3<f64>,
        resolved: Vector3<f64>,
        requested: Vector3<f64>,
    ) -> bool {
        // A blocked attempt may write no entry. Removing the latest movement can then
        // consume an earlier accepted movement, as in Entity.removeLatestMovementRecording.
        let resolved_length = resolved.length_squared();
        if resolved_length <= 1.0e-7 && requested.length_squared() - resolved_length >= 1.0e-7 {
            return false;
        }

        let movements = &mut self.pending;
        if movements.len() >= MAX_MOVEMENTS_PER_TICK
            && let Some(first) = movements.pop_front()
            && let Some(second) = movements.front_mut()
        {
            second.from = first.from;
            second.requested = None;
        }
        movements.push_back(VehicleMovement {
            from,
            to: from + resolved,
            requested: Some(requested),
        });
        true
    }
}
