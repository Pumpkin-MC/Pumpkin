use pumpkin_data::enchantment::LevelBasedValue;

/// Matches vanilla `net.minecraft.world.item.enchantment.effects.SetValue`.
#[derive(Clone, Debug, PartialEq)]
pub struct SetValue {
    pub value: LevelBasedValue,
}

impl SetValue {
    #[must_use]
    pub const fn new(value: LevelBasedValue) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn process(&self, level: i32) -> f32 {
        self.value.calculate(level)
    }
}
