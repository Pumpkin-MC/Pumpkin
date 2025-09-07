use pumpkin_nbt::tag::NbtTag;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EulerAngle {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

impl EulerAngle {
    pub fn new(pitch: f32, yaw: f32, roll: f32) -> Self {
        let pitch = if pitch.is_finite() {
            pitch % 360.0
        } else {
            0.0
        };
        let yaw = if yaw.is_finite() { yaw % 360.0 } else { 0.0 };
        let roll = if roll.is_finite() { roll % 360.0 } else { 0.0 };

        Self { pitch, yaw, roll }
    }

    pub const ZERO: EulerAngle = EulerAngle {
        pitch: 0.0,
        yaw: 0.0,
        roll: 0.0,
    };
}

impl Default for EulerAngle {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Into<NbtTag> for EulerAngle {
    fn into(self) -> NbtTag {
        NbtTag::List(vec![
            NbtTag::Float(self.pitch),
            NbtTag::Float(self.yaw),
            NbtTag::Float(self.roll),
        ])
    }
}

impl From<NbtTag> for EulerAngle {
    fn from(tag: NbtTag) -> Self {
        if let NbtTag::List(list) = tag {
            if list.len() == 3 {
                let pitch = if let NbtTag::Float(f) = list[0] {
                    f
                } else {
                    0.0
                };
                let yaw = if let NbtTag::Float(f) = list[1] {
                    f
                } else {
                    0.0
                };
                let roll = if let NbtTag::Float(f) = list[2] {
                    f
                } else {
                    0.0
                };

                return Self::new(pitch, yaw, roll);
            }
        }

        Self::ZERO
    }
}
