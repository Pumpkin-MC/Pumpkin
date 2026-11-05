#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CrackinessLevel {
    None,
    Low,
    Medium,
    High,
}

pub struct Crackiness {
    low: f32,
    medium: f32,
    high: f32,
}

impl Crackiness {
    pub const GOLEM: Self = Self::new(0.75, 0.5, 0.25);

    const fn new(low: f32, medium: f32, high: f32) -> Self {
        Self { low, medium, high }
    }

    #[must_use]
    pub fn by_fraction(&self, fraction: f32) -> CrackinessLevel {
        if fraction < self.high {
            CrackinessLevel::High
        } else if fraction < self.medium {
            CrackinessLevel::Medium
        } else if fraction < self.low {
            CrackinessLevel::Low
        } else {
            CrackinessLevel::None
        }
    }
}
