//! Window property definitions.
//!
//! This module defines container-specific UI properties that need to be synchronized
//! between server and client. These include progress bars, fuel indicators, and
//! other visual elements in container screens.
//!
//! # Window Properties
//!
//! Properties are identified by a unique ID and sent to the client to update
//! the container's visual state:
//! - Furnace: Fire icon animation, smelting progress
//! - Enchantment table: Level requirements, available enchantments
//! - Brewing stand: Brew time, fuel level
//! - Anvil: Repair cost
//!
//! See the Minecraft wiki for property ID mappings.

/// Trait for types that can be converted to window property IDs.
pub trait WindowPropertyTrait {
    /// Converts this property to its protocol ID.
    fn to_id(self) -> i16;
}

/// A window property with a specific value.
///
/// Used to send property updates to the client (e.g., furnace progress bar).
pub struct WindowProperty<T: WindowPropertyTrait> {
    /// The property type being tracked (e.g., furnace fire icon, progress arrow).
    window_property: T,
    /// The current value of the property.
    value: i16,
}

impl<T: WindowPropertyTrait> WindowProperty<T> {
    /// Creates a new window property.
    ///
    /// # Arguments
    /// - `window_property` - The property type
    /// - `value` - The property value
    #[must_use]
    pub const fn new(window_property: T, value: i16) -> Self {
        Self {
            window_property,
            value,
        }
    }

    /// Converts this property to a tuple of (id, value).
    #[must_use]
    pub fn into_tuple(self) -> (i16, i16) {
        (self.window_property.to_id(), self.value)
    }
}

/// Furnace window properties.
pub enum Furnace {
    /// Fire icon animation level (0-250).
    FireIcon,
    /// Maximum fuel burn time.
    MaximumFuelBurnTime,
    /// Arrow progress animation (0-250).
    ProgressArrow,
    /// Maximum smelting progress time.
    MaximumProgress,
}

/// Enchantment table window properties.
pub enum EnchantmentTable {
    /// Experience level requirement for a specific slot.
    LevelRequirement { slot: u8 },
    /// Random seed for enchantment generation.
    EnchantmentSeed,
    /// Enchantment ID for a specific slot.
    EnchantmentId { slot: u8 },
    /// Enchantment level for a specific slot.
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

/// Beacon window properties.
pub enum Beacon {
    /// Effect power level (1-4).
    PowerLevel,
    /// First selected potion effect ID.
    FirstPotionEffect,
    /// Second selected potion effect ID.
    SecondPotionEffect,
}

/// Anvil window properties.
pub enum Anvil {
    /// Total repair cost in experience levels.
    RepairCost,
}

/// Brewing stand window properties.
pub enum BrewingStand {
    /// Brewing progress (0-400).
    BrewTime,
    /// Fuel time remaining (0-20).
    FuelTime,
}

/// Stonecutter window properties.
pub enum Stonecutter {
    /// ID of the selected recipe.
    SelectedRecipe,
}

/// Loom window properties.
pub enum Loom {
    /// ID of the selected pattern.
    SelectedPattern,
}

/// Lectern window properties.
pub enum Lectern {
    /// Current page number being viewed.
    PageNumber,
}
