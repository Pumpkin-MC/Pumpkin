use pumpkin_util::math::subtract_angles;

pub mod look_control;

pub trait Control: Send + Sync {
    fn change_angle(&self, start: f32, end: f32, max_change: f32) -> f32 {
        let i = subtract_angles(start, end);
        let j = i.clamp(-max_change, max_change);
        start + j
    }
}
