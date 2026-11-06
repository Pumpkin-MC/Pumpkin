//! A v0.2 plugin using the non-event SDK helpers.
#![cfg(target_family = "wasm")]

use pumpkin_plugin_api_v0_2::{
    Context, Item, ItemStack, ItemStackExt, PersistentDataHolder, Plugin, PluginMetadata, Result,
    TextComponent, enchantment::EnchantmentBuilder, recipe::ShapedRecipeBuilder, register_plugin,
};

struct Example;

impl Plugin for Example {
    fn new() -> Self {
        Self
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "sdk-helpers".into(),
            version: "0.1.0".into(),
            authors: Vec::new(),
            description: "Registers a recipe and enchantment using the v0.2 SDK".into(),
            dependencies: Vec::new(),
            permissions: Vec::new(),
        }
    }

    async fn on_load(&self, context: Context) -> Result<()> {
        let output = ItemStack::of(Item::from_name("diamond").ok_or("unknown diamond item")?, 1);
        output.set_string("sdk-helpers", "origin", "recipe").await;
        ShapedRecipeBuilder::new("sdk_helpers:diamond", output)
            .pattern(["CC", "CC"])
            .key('C', "minecraft:coal")
            .register_to_context(&context)
            .await
            .map_err(|error| error.to_string())?;
        EnchantmentBuilder::new("sdk_helpers:example", TextComponent::text("Example"))
            .max_level(3)
            .register_to_context(&context)
            .await
            .map_err(|error| error.to_string())?;
        tracing::info!("SDK helper registrations completed");
        Ok(())
    }
}

register_plugin!(Example);
