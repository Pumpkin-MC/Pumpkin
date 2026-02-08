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

impl WindowPropertyTrait for Beacon {
    fn to_id(self) -> i16 {
        match self {
            Self::PowerLevel => 0,
            Self::FirstPotionEffect => 1,
            Self::SecondPotionEffect => 2,
        }
    }
}

pub enum Anvil {
    RepairCost,
}

impl WindowPropertyTrait for Anvil {
    fn to_id(self) -> i16 {
        match self {
            Self::RepairCost => 0,
        }
    }
}

pub enum BrewingStand {
    BrewTime,
    FuelTime,
}

impl WindowPropertyTrait for BrewingStand {
    fn to_id(self) -> i16 {
        match self {
            Self::BrewTime => 0,
            Self::FuelTime => 1,
        }
    }
}

pub enum Stonecutter {
    SelectedRecipe,
}

impl WindowPropertyTrait for Stonecutter {
    fn to_id(self) -> i16 {
        match self {
            Self::SelectedRecipe => 0,
        }
    }
}

pub enum Loom {
    SelectedPattern,
}

impl WindowPropertyTrait for Loom {
    fn to_id(self) -> i16 {
        match self {
            Self::SelectedPattern => 0,
        }
    }
}

pub enum Lectern {
    PageNumber,
}

impl WindowPropertyTrait for Lectern {
    fn to_id(self) -> i16 {
        match self {
            Self::PageNumber => 0,
        }
    }
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

    #[test]
    fn beacon_power_level() {
        assert_eq!(Beacon::PowerLevel.to_id(), 0);
    }

    #[test]
    fn beacon_first_potion_effect() {
        assert_eq!(Beacon::FirstPotionEffect.to_id(), 1);
    }

    #[test]
    fn beacon_second_potion_effect() {
        assert_eq!(Beacon::SecondPotionEffect.to_id(), 2);
    }

    #[test]
    fn anvil_repair_cost() {
        assert_eq!(Anvil::RepairCost.to_id(), 0);
    }

    #[test]
    fn brewing_stand_brew_time() {
        assert_eq!(BrewingStand::BrewTime.to_id(), 0);
    }

    #[test]
    fn brewing_stand_fuel_time() {
        assert_eq!(BrewingStand::FuelTime.to_id(), 1);
    }

    #[test]
    fn stonecutter_selected_recipe() {
        assert_eq!(Stonecutter::SelectedRecipe.to_id(), 0);
    }

    #[test]
    fn loom_selected_pattern() {
        assert_eq!(Loom::SelectedPattern.to_id(), 0);
    }

    #[test]
    fn lectern_page_number() {
        assert_eq!(Lectern::PageNumber.to_id(), 0);
    }
}
