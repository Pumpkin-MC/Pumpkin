use pumpkin_data::enchantment::LevelBasedValue;

/// Matches vanilla `net.minecraft.world.item.enchantment.effects.DamageEntity`.
#[derive(Clone, Debug, PartialEq)]
pub struct DamageEntity {
    pub damage: LevelBasedValue,
}

impl DamageEntity {
    #[must_use]
    pub const fn new(damage: LevelBasedValue) -> Self {
        Self { damage }
    }

    #[must_use]
    pub fn calculate_damage(&self, level: i32) -> f32 {
        self.damage.calculate(level)
    }
}
