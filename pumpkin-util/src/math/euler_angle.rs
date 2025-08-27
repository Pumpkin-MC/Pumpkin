use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EulerAngle {
    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
}

impl EulerAngle {
    pub fn new(pitch: f32, yaw: f32, roll: f32) -> Self {
        let pitch = if pitch.is_finite() { pitch % 360.0 } else { 0.0 };
        let yaw = if yaw.is_finite() { yaw % 360.0 } else { 0.0 };
        let roll = if roll.is_finite() { roll % 360.0 } else { 0.0 };

        Self { pitch, yaw, roll }
    }

    pub const ZERO: EulerAngle = EulerAngle {
        pitch: 0.0,
        yaw: 0.0,
        roll: 0.0,
    };

    //? Correct me if i am wrong with that or have better solutions
    /// To NBT compatible ig
    pub fn to_list(&self) -> Vec<f32> {
        vec![self.pitch, self.yaw, self.roll]
    }

    //? Correct me if i am wrong with that or have better solutions
    /// From NBT compatible list
    pub fn from_list(list: &[f32]) -> Result<Self, String> {
        if list.len() != 3 {
            return Err(format!("Expected 3 floats, got {}", list.len()));
        }
        Ok(Self::new(list[0], list[1], list[2]))
    }
}

/// AtomicCell default
impl Default for EulerAngle {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<EulerAngle> for Vec<f32> {
    fn from(angle: EulerAngle) -> Self {
        angle.to_list()
    }
}

impl TryFrom<Vec<f32>> for EulerAngle {
    type Error = String;

    fn try_from(value: Vec<f32>) -> Result<Self, Self::Error> {
        Self::from_list(&value)
    }
}

impl From<(f32, f32, f32)> for EulerAngle {
    fn from((pitch, yaw, roll): (f32, f32, f32)) -> Self {
        Self::new(pitch, yaw, roll)
    }
}
