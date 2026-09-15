use pumpkin_data::enchantment::LevelBasedValue;

#[derive(Clone, Debug, PartialEq)]
pub struct MultiplyValue {
    pub factor: LevelBasedValue,
}

impl MultiplyValue {
    #[must_use]
    pub const fn new(factor: LevelBasedValue) -> Self {
        Self { factor }
    }

    #[must_use]
    pub fn process(&self, level: i32, current_value: f32) -> f32 {
        current_value * self.factor.calculate(level)
    }
}
