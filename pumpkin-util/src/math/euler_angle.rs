use pumpkin_nbt::tag::NbtTag;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EulerAngle {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

impl EulerAngle {
    #[must_use]
    pub fn new(pitch: f32, yaw: f32, roll: f32) -> Self {
        let pitch = pitch % 360.0;
        let yaw = yaw % 360.0;
        let roll = roll % 360.0;

        Self { pitch, yaw, roll }
    }

    pub const ZERO: Self = Self {
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

impl From<EulerAngle> for NbtTag {
    fn from(val: EulerAngle) -> Self {
        Self::List(vec![
            Self::Float(val.pitch),
            Self::Float(val.yaw),
            Self::Float(val.roll),
        ])
    }
}

impl From<NbtTag> for EulerAngle {
    fn from(tag: NbtTag) -> Self {
        if let NbtTag::List(list) = tag
            && list.len() == 3
        {
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

        Self::ZERO
    }
}
