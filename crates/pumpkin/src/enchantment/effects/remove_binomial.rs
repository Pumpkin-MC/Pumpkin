use pumpkin_data::enchantment::LevelBasedValue;

#[derive(Clone, Debug, PartialEq)]
pub struct RemoveBinomial {
    pub chance: LevelBasedValue,
}

impl RemoveBinomial {
    #[must_use]
    pub const fn new(chance: LevelBasedValue) -> Self {
        Self { chance }
    }

    #[must_use]
    pub fn process(&self, level: i32, current_value: f32) -> f32 {
        let prob = self.chance.calculate(level);
        if rand::random::<f32>() < prob {
            0.0
        } else {
            current_value
        }
    }
}
