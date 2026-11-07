use std::sync::Arc;

use pumpkin_util::math::{vector3::Vector3, wrap_degrees};
use rand::RngExt;

use super::super::BrainTick;
use super::super::memory::position_tracker::{BlockPosTracker, PositionTracker};
use super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::timed::Behavior;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 2] = [
    (types::LOOK_TARGET.id(), MemoryStatus::ValueAbsent),
    (types::GAZE_COOLDOWN_TICKS.id(), MemoryStatus::ValueAbsent),
];

pub struct RandomLookAround {
    min_interval: i32,
    max_interval: i32,
    max_yaw: f32,
    min_pitch: f32,
    pitch_range: f32,
}

impl RandomLookAround {
    #[must_use]
    pub const fn new(
        min_interval: i32,
        max_interval: i32,
        max_yaw: f32,
        min_pitch: f32,
        max_pitch: f32,
    ) -> Self {
        Self {
            min_interval,
            max_interval,
            max_yaw,
            min_pitch,
            pitch_range: max_pitch - min_pitch,
        }
    }
}

impl Behavior for RandomLookAround {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        let mut rng = tick.mob.get_random();
        let entity = tick.mob.get_entity();
        let pitch = rng
            .random::<f32>()
            .mul_add(self.pitch_range, self.min_pitch)
            .clamp(-90.0, 90.0);
        let yaw = wrap_degrees(
            entity.yaw.load() + 2.0 * rng.random::<f32>() * self.max_yaw - self.max_yaw,
        );
        let direction = Vector3::rotation_vector(f64::from(pitch), f64::from(yaw));
        tick.state.set(
            types::LOOK_TARGET,
            Arc::new(BlockPosTracker::from_vec(entity.get_eye_pos() + direction))
                as Arc<dyn PositionTracker>,
        );
        tick.state.set(
            types::GAZE_COOLDOWN_TICKS,
            rng.random_range(self.min_interval..=self.max_interval),
        );
    }

    fn debug_name(&self) -> &'static str {
        "RandomLookAround"
    }
}
