#![allow(async_fn_in_trait)]

//! Plugin custom enchantment registration and builder utilities.
//!
//! This module provides a fluent, type-safe API for defining, registering, and querying
//! custom enchantments as well as applying custom enchantments to [`ItemStack`](crate::ItemStack)s.
//!
//! # Examples
//!
//! ## Defining and Registering a Custom Enchantment
//! ```rust,ignore
//! use pumpkin_plugin_api_v0_2::{
//!     enchantment::{AttributeModifierSlot, EnchantmentBuilder},
//!     text::TextComponent,
//!     Server,
//! };
//!
//! async fn register_enchantments(server: &Server) {
//!     let manager = server.get_enchantment_manager();
//!
//!     manager.register(
//!         EnchantmentBuilder::new("my_plugin:lifesteal", TextComponent::text("Life Steal"))
//!             .max_level(3)
//!             .anvil_cost(4)
//!             .supported_items("#minecraft:enchantable/weapon")
//!             .weight(2)
//!             .slots([AttributeModifierSlot::MainHand])
//!             .exclusive_with("custom:poison_touch")
//!     ).await.expect("failed to register custom enchantment");
//! }
//! ```
//!
//! ## Applying Custom Enchantments to an [`ItemStack`](crate::ItemStack)
//! ```rust,ignore
//! use pumpkin_plugin_api_v0_2::ItemStack;
//!
//! async fn give_sword() -> ItemStack {
//!     let sword = ItemStack::new("minecraft:diamond_sword", 1);
//!     sword.add_custom_enchantment("my_plugin:lifesteal".to_string(), 2).await;
//!     assert!(sword.has_custom_enchantment("my_plugin:lifesteal".to_string()).await);
//!     assert_eq!(sword.get_custom_enchantment_level("my_plugin:lifesteal".to_string()).await, Some(2));
//!     sword
//! }
//! ```

pub use crate::wit::pumpkin::plugin::enchantments::{
    AttributeModifierSlot, CustomEnchantment, Enchantment, EnchantmentManager,
};
pub use crate::wit::pumpkin::plugin::item_stack::CustomEnchantmentValue;
use crate::{Context, Server, TextComponent};
use std::fmt;

/// Errors that can occur when building or registering a custom enchantment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EnchantmentError {
    /// The enchantment identifier was empty.
    EmptyId,
    /// The enchantment maximum level was invalid (must be >= 1).
    InvalidMaxLevel,
    /// Registration with the server enchantment manager failed.
    RegistrationFailed(String),
}

impl fmt::Display for EnchantmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId => write!(f, "enchantment identifier cannot be empty"),
            Self::InvalidMaxLevel => write!(f, "enchantment max level must be at least 1"),
            Self::RegistrationFailed(msg) => {
                write!(f, "failed to register enchantment: {msg}")
            }
        }
    }
}

impl std::error::Error for EnchantmentError {}

/// Fluent builder for constructing and validating a [`CustomEnchantment`].
pub struct EnchantmentBuilder {
    id: String,
    description: TextComponent,
    max_level: u32,
    anvil_cost: u32,
    supported_items: String,
    weight: u32,
    slots: Vec<AttributeModifierSlot>,
    exclusive_set: Vec<String>,
}

impl EnchantmentBuilder {
    /// Creates a new enchantment builder with the given unique identifier and description.
    ///
    /// # Example
    /// ```rust,ignore
    /// let builder = EnchantmentBuilder::new("my_plugin:lifesteal", TextComponent::text("Life Steal"));
    /// ```
    #[must_use]
    pub fn new(id: impl Into<String>, description: impl Into<TextComponent>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            max_level: 1,
            anvil_cost: 4,
            supported_items: "#minecraft:enchantable/weapon".into(),
            weight: 5,
            slots: vec![AttributeModifierSlot::MainHand],
            exclusive_set: Vec::new(),
        }
    }

    /// Sets the maximum level of the enchantment (default: 1).
    #[must_use]
    pub const fn max_level(mut self, max_level: u32) -> Self {
        self.max_level = max_level;
        self
    }

    /// Sets the base anvil cost multiplier (default: 4).
    #[must_use]
    pub const fn anvil_cost(mut self, anvil_cost: u32) -> Self {
        self.anvil_cost = anvil_cost;
        self
    }

    /// Sets the supported items pattern or tag (e.g. `"#minecraft:enchantable/weapon"`).
    #[must_use]
    pub fn supported_items(mut self, items: impl Into<String>) -> Self {
        self.supported_items = items.into();
        self
    }

    /// Sets the weight / rarity of the enchantment (1..=10, higher = more common, default: 5).
    #[must_use]
    pub const fn weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    /// Adds a single active equipment slot for this enchantment.
    #[must_use]
    pub fn slot(mut self, slot: AttributeModifierSlot) -> Self {
        self.slots.push(slot);
        self
    }

    /// Replaces the active equipment slots for this enchantment.
    #[must_use]
    pub fn slots(mut self, slots: impl IntoIterator<Item = AttributeModifierSlot>) -> Self {
        self.slots = slots.into_iter().collect();
        self
    }

    /// Adds an exclusive / conflicting enchantment ID.
    #[must_use]
    pub fn exclusive_with(mut self, id: impl Into<String>) -> Self {
        self.exclusive_set.push(id.into());
        self
    }

    /// Replaces the exclusive / conflicting enchantment list.
    #[must_use]
    pub fn exclusive_set(mut self, set: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.exclusive_set = set.into_iter().map(Into::into).collect();
        self
    }

    /// Validates the enchantment parameters.
    ///
    /// # Errors
    /// Returns [`EnchantmentError`] if validation fails.
    pub fn validate(&self) -> Result<(), EnchantmentError> {
        if self.id.trim().is_empty() {
            return Err(EnchantmentError::EmptyId);
        }
        if self.max_level == 0 {
            return Err(EnchantmentError::InvalidMaxLevel);
        }
        Ok(())
    }

    /// Builds the validated [`CustomEnchantment`] record.
    ///
    /// # Errors
    /// Returns [`EnchantmentError`] if validation fails.
    pub fn build(self) -> Result<CustomEnchantment, EnchantmentError> {
        self.validate()?;
        Ok(CustomEnchantment {
            id: self.id,
            description: self.description,
            max_level: self.max_level,
            anvil_cost: self.anvil_cost,
            supported_items: self.supported_items,
            weight: self.weight,
            slots: self.slots,
            exclusive_set: self.exclusive_set,
        })
    }

    /// Registers this custom enchantment directly with the provided [`EnchantmentManager`].
    ///
    /// # Errors
    /// Returns [`EnchantmentError`] if validation or registration fails.
    pub async fn register(self, manager: &EnchantmentManager) -> Result<(), EnchantmentError> {
        manager.register(self).await
    }

    /// Registers this custom enchantment with the server.
    ///
    /// # Errors
    /// Returns [`EnchantmentError`] if validation or registration fails.
    pub async fn register_to_server(self, server: &Server) -> Result<(), EnchantmentError> {
        let manager = server.get_enchantment_manager();
        manager.register(self).await
    }

    /// Registers this custom enchantment with the plugin context.
    ///
    /// # Errors
    /// Returns [`EnchantmentError`] if validation or registration fails.
    pub async fn register_to_context(self, context: &Context) -> Result<(), EnchantmentError> {
        let manager = context.get_enchantment_manager();
        manager.register(self).await
    }
}

/// Trait for enchantment types that can be registered with an [`EnchantmentManager`].
pub trait RegistrableEnchantment {
    /// Registers this enchantment with the provided enchantment manager.
    ///
    /// # Errors
    /// Returns [`EnchantmentError`] if validation or registration fails.
    async fn register(self, manager: &EnchantmentManager) -> Result<(), EnchantmentError>;
}

impl RegistrableEnchantment for EnchantmentBuilder {
    async fn register(self, manager: &EnchantmentManager) -> Result<(), EnchantmentError> {
        let enchantment = self.build()?;
        manager
            .register_enchantment(enchantment)
            .await
            .map_err(EnchantmentError::RegistrationFailed)
    }
}

impl RegistrableEnchantment for CustomEnchantment {
    async fn register(self, manager: &EnchantmentManager) -> Result<(), EnchantmentError> {
        manager
            .register_enchantment(self)
            .await
            .map_err(EnchantmentError::RegistrationFailed)
    }
}

impl EnchantmentManager {
    /// Registers a custom enchantment with the server.
    ///
    /// Accepts [`EnchantmentBuilder`] or a [`CustomEnchantment`].
    ///
    /// # Errors
    /// Returns [`EnchantmentError`] if registration or validation fails.
    pub async fn register(
        &self,
        enchantment: impl RegistrableEnchantment,
    ) -> Result<(), EnchantmentError> {
        enchantment.register(self).await
    }

    /// Gets an enchantment definition by its ID (custom or vanilla).
    #[must_use]
    pub async fn get(&self, id: &str) -> Option<CustomEnchantment> {
        self.get_enchantment(id.to_string()).await
    }

    /// Checks if an enchantment ID is registered on the server.
    #[must_use]
    pub async fn has(&self, id: &str) -> bool {
        self.has_enchantment(id.to_string()).await
    }

    /// Returns all registered enchantment IDs (custom and vanilla).
    #[must_use]
    pub async fn get_all_ids(&self) -> Vec<String> {
        self.get_all_enchantment_ids().await
    }
}

impl Context {
    /// Returns the global enchantment manager for registering and querying custom enchantments.
    #[must_use]
    pub fn get_enchantment_manager(&self) -> EnchantmentManager {
        self.get_server().get_enchantment_manager()
    }

    /// Registers a custom enchantment with the server.
    ///
    /// # Errors
    /// Returns [`EnchantmentError`] if validation or registration fails.
    pub async fn register_enchantment(
        &self,
        enchantment: impl RegistrableEnchantment,
    ) -> Result<(), EnchantmentError> {
        self.get_enchantment_manager().register(enchantment).await
    }

    /// Gets an enchantment definition by its ID.
    #[must_use]
    pub async fn get_enchantment(&self, id: &str) -> Option<CustomEnchantment> {
        self.get_server().get_enchantment(id.to_string()).await
    }
}

impl Server {
    /// Registers a custom enchantment with the server.
    ///
    /// # Errors
    /// Returns [`EnchantmentError`] if validation or registration fails.
    pub async fn register_enchantment(
        &self,
        enchantment: impl RegistrableEnchantment,
    ) -> Result<(), EnchantmentError> {
        self.get_enchantment_manager().register(enchantment).await
    }
}

/// Converts a positive integer to a standard Roman numeral representation (e.g. `1 -> "I"`, `3 -> "III"`, `5 -> "V"`).
#[must_use]
pub fn to_roman_numeral(level: u32) -> String {
    match level {
        1 => "I".into(),
        2 => "II".into(),
        3 => "III".into(),
        4 => "IV".into(),
        5 => "V".into(),
        6 => "VI".into(),
        7 => "VII".into(),
        8 => "VIII".into(),
        9 => "IX".into(),
        10 => "X".into(),
        _ => level.to_string(),
    }
}
