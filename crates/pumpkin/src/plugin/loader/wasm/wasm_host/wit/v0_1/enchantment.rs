use crate::plugin::loader::wasm::wasm_host::state::PluginHostState;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::enchantments::{
    AttributeModifierSlot as WitAttributeModifierSlot, CustomEnchantment as WitCustomEnchantment,
    EnchantmentManager as WitEnchantmentManager, HostEnchantmentManager,
};
use crate::server::enchantment::CustomEnchantmentEntry;
use pumpkin_data::enchantment::{AttributeModifierSlot, Enchantment};
use pumpkin_util::text::TextComponent;
use wasmtime::component::Resource;

impl HostEnchantmentManager for PluginHostState {
    async fn register_enchantment(
        &mut self,
        _res: Resource<WitEnchantmentManager>,
        enchantment: WitCustomEnchantment,
    ) -> wasmtime::Result<Result<(), String>> {
        let description =
            super::player::text_component_from_resource(self, &enchantment.description);
        let entry = CustomEnchantmentEntry {
            id: enchantment.id,
            description,
            max_level: enchantment.max_level,
            anvil_cost: enchantment.anvil_cost,
            supported_items: enchantment.supported_items,
            weight: enchantment.weight,
            slots: enchantment.slots.into_iter().map(to_data_slot).collect(),
            exclusive_set: enchantment.exclusive_set,
        };

        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        Ok(server.enchantment_manager.register(entry).await)
    }

    async fn get_enchantment(
        &mut self,
        _res: Resource<WitEnchantmentManager>,
        id: String,
    ) -> wasmtime::Result<Option<WitCustomEnchantment>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;

        if let Some(entry) = server.enchantment_manager.get(&id).await {
            let description = self.add_text_component(entry.description)?;
            return Ok(Some(WitCustomEnchantment {
                id: entry.id,
                description,
                max_level: entry.max_level,
                anvil_cost: entry.anvil_cost,
                supported_items: entry.supported_items,
                weight: entry.weight,
                slots: entry.slots.iter().map(to_wit_slot).collect(),
                exclusive_set: entry.exclusive_set,
            }));
        }

        if let Some(vanilla) = find_vanilla_enchantment(&id) {
            let description =
                self.add_text_component(TextComponent::translate(vanilla.description, []))?;
            return Ok(Some(WitCustomEnchantment {
                id: vanilla.name.to_string(),
                description,
                max_level: vanilla.max_level.max(1) as u32,
                anvil_cost: vanilla.anvil_cost,
                supported_items: vanilla
                    .supported_items
                    .0
                    .first()
                    .copied()
                    .unwrap_or("")
                    .to_string(),
                weight: vanilla.weight.max(1) as u32,
                slots: vanilla.slots.iter().map(to_wit_slot).collect(),
                exclusive_set: vanilla.exclusive_set.map_or_else(Vec::new, |tag| {
                    tag.0.iter().map(|s| (*s).to_string()).collect()
                }),
            }));
        }

        Ok(None)
    }

    async fn has_enchantment(
        &mut self,
        _res: Resource<WitEnchantmentManager>,
        id: String,
    ) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        if server.enchantment_manager.has(&id).await {
            return Ok(true);
        }
        Ok(find_vanilla_enchantment(&id).is_some())
    }

    async fn get_all_enchantment_ids(
        &mut self,
        _res: Resource<WitEnchantmentManager>,
    ) -> wasmtime::Result<Vec<String>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let mut ids = server.enchantment_manager.get_all_ids().await;
        for enc in Enchantment::ALL {
            ids.push(enc.name.to_string());
        }
        Ok(ids)
    }

    async fn drop(&mut self, _rep: Resource<WitEnchantmentManager>) -> wasmtime::Result<()> {
        Ok(())
    }
}

#[must_use]
pub fn find_vanilla_enchantment(id: &str) -> Option<&'static Enchantment> {
    Enchantment::from_name(id).or_else(|| {
        id.strip_prefix("minecraft:")
            .and_then(Enchantment::from_name)
    })
}

#[must_use]
pub const fn to_data_slot(slot: WitAttributeModifierSlot) -> AttributeModifierSlot {
    match slot {
        WitAttributeModifierSlot::Any => AttributeModifierSlot::Any,
        WitAttributeModifierSlot::MainHand => AttributeModifierSlot::MainHand,
        WitAttributeModifierSlot::OffHand => AttributeModifierSlot::OffHand,
        WitAttributeModifierSlot::Hand => AttributeModifierSlot::Hand,
        WitAttributeModifierSlot::Feet => AttributeModifierSlot::Feet,
        WitAttributeModifierSlot::Legs => AttributeModifierSlot::Legs,
        WitAttributeModifierSlot::Chest => AttributeModifierSlot::Chest,
        WitAttributeModifierSlot::Head => AttributeModifierSlot::Head,
        WitAttributeModifierSlot::Armor => AttributeModifierSlot::Armor,
        WitAttributeModifierSlot::Body => AttributeModifierSlot::Body,
        WitAttributeModifierSlot::Saddle => AttributeModifierSlot::Saddle,
    }
}

#[must_use]
pub const fn to_wit_slot(slot: &AttributeModifierSlot) -> WitAttributeModifierSlot {
    match slot {
        AttributeModifierSlot::Any => WitAttributeModifierSlot::Any,
        AttributeModifierSlot::MainHand => WitAttributeModifierSlot::MainHand,
        AttributeModifierSlot::OffHand => WitAttributeModifierSlot::OffHand,
        AttributeModifierSlot::Hand => WitAttributeModifierSlot::Hand,
        AttributeModifierSlot::Feet => WitAttributeModifierSlot::Feet,
        AttributeModifierSlot::Legs => WitAttributeModifierSlot::Legs,
        AttributeModifierSlot::Chest => WitAttributeModifierSlot::Chest,
        AttributeModifierSlot::Head => WitAttributeModifierSlot::Head,
        AttributeModifierSlot::Armor => WitAttributeModifierSlot::Armor,
        AttributeModifierSlot::Body => WitAttributeModifierSlot::Body,
        AttributeModifierSlot::Saddle => WitAttributeModifierSlot::Saddle,
    }
}
