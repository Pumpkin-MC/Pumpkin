pub trait WindowPropertyTrait {
    fn to_id(self) -> i16;
}

pub struct WindowProperty<T: WindowPropertyTrait> {
    window_property: T,
    value: i16,
}

impl<T: WindowPropertyTrait> WindowProperty<T> {
    pub const fn new(window_property: T, value: i16) -> Self {
        Self {
            window_property,
            value,
        }
    }

    pub fn into_tuple(self) -> (i16, i16) {
        (self.window_property.to_id(), self.value)
    }
}
pub enum Furnace {
    FireIcon,
    MaximumFuelBurnTime,
    ProgressArrow,
    MaximumProgress,
}

pub enum EnchantmentTable {
    LevelRequirement { slot: u8 },
    EnchantmentSeed,
    EnchantmentId { slot: u8 },
    EnchantmentLevel { slot: u8 },
}

// TODO: No more magic numbers
impl WindowPropertyTrait for EnchantmentTable {
    fn to_id(self) -> i16 {
        use EnchantmentTable::{
            EnchantmentId, EnchantmentLevel, EnchantmentSeed, LevelRequirement,
        };

        i16::from(match self {
            LevelRequirement { slot } => slot,
            EnchantmentSeed => 3,
            EnchantmentId { slot } => 4 + slot,
            EnchantmentLevel { slot } => 7 + slot,
        })
    }
}
pub enum Beacon {
    PowerLevel,
    FirstPotionEffect,
    SecondPotionEffect,
}

pub enum Anvil {
    RepairCost,
}

pub enum BrewingStand {
    BrewTime,
    FuelTime,
}

pub enum Stonecutter {
    SelectedRecipe,
}

pub enum Loom {
    SelectedPattern,
}

pub enum Lectern {
    PageNumber,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enchantment_table_level_requirement_slot_0() {
        let id = EnchantmentTable::LevelRequirement { slot: 0 }.to_id();
        assert_eq!(id, 0);
    }

    #[test]
    fn enchantment_table_level_requirement_slot_2() {
        let id = EnchantmentTable::LevelRequirement { slot: 2 }.to_id();
        assert_eq!(id, 2);
    }

    #[test]
    fn enchantment_table_seed() {
        let id = EnchantmentTable::EnchantmentSeed.to_id();
        assert_eq!(id, 3);
    }

    #[test]
    fn enchantment_table_id_slot_0() {
        let id = EnchantmentTable::EnchantmentId { slot: 0 }.to_id();
        assert_eq!(id, 4);
    }

    #[test]
    fn enchantment_table_level_slot_0() {
        let id = EnchantmentTable::EnchantmentLevel { slot: 0 }.to_id();
        assert_eq!(id, 7);
    }

    #[test]
    fn window_property_into_tuple() {
        let prop = WindowProperty::new(EnchantmentTable::EnchantmentSeed, 42);
        let (id, value) = prop.into_tuple();
        assert_eq!(id, 3);
        assert_eq!(value, 42);
    }
}
