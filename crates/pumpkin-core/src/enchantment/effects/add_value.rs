use pumpkin_data::enchantment::LevelBasedValue;

#[derive(Clone, Debug, PartialEq)]
pub struct AddValue {
    pub value: LevelBasedValue,
}

impl AddValue {
    #[must_use]
    pub const fn new(value: LevelBasedValue) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn process(&self, level: i32, current_value: f32) -> f32 {
        current_value + self.value.calculate(level)
    }
}
