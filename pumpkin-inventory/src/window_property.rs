pub trait WindowPropertyTrait {
    fn to_id(self) -> i16;
}

pub struct WindowProperty<T: WindowPropertyTrait> {
    window_property: T,
    value: i16,
}

impl<T: WindowPropertyTrait> WindowProperty<T> {
    pub fn new(window_property: T, value: i16) -> Self {
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
        use EnchantmentTable::*;

        (match self {
            LevelRequirement { slot } => slot,
            EnchantmentSeed => 3,
            EnchantmentId { slot } => 4 + slot,
            EnchantmentLevel { slot } => 7 + slot,
        }) as i16
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
