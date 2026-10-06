use crate::item_stack::from_wit_data_component;
use crate::pumpkin::plugin::item_registry::{
    Host, ItemDefinition as WitItemDefinition, ItemEntry as WitItemEntry,
};
use pumpkin_core::plugin::permissions;
use pumpkin_data::dynamic_tag;
use pumpkin_data::item::Item;
use pumpkin_data::item_registry::ItemRegistration;
use pumpkin_data::tag::RegistryKey;
use pumpkin_protocol::codec::data_component::deserialize;
use pumpkin_wasm_host_common::state::PluginHostState;
use tracing::{info, warn};

fn to_wit_entry(item: &Item) -> WitItemEntry {
    WitItemEntry {
        key: item.resource_location().into_owned(),
        id: u32::from(item.id),
    }
}

/// Whether the plugin may register items right now.
fn check_item_registration_allowed(state: &PluginHostState) -> Result<(), String> {
    if !state
        .permissions
        .iter()
        .any(|p| p == permissions::REGISTRY_ITEMS)
    {
        return Err(format!(
            "registering items needs the '{}' permission",
            permissions::REGISTRY_ITEMS
        ));
    }
    if Item::is_dynamic_registry_frozen() {
        return Err("items can only be registered before players connect".to_string());
    }
    Ok(())
}

impl Host for PluginHostState {
    fn register_item(
        &mut self,
        definition: WitItemDefinition,
    ) -> wasmtime::Result<Result<u32, String>> {
        if let Err(error) = check_item_registration_allowed(self) {
            return Ok(Err(error));
        }

        let mut components = Vec::with_capacity(definition.components.len());
        for entry in definition.components {
            let id = from_wit_data_component(entry.component);
            let mut cursor = std::io::Cursor::new(entry.value);
            match deserialize(id, &mut cursor) {
                Ok(component) => components.push((id, component)),
                // Not every component has a reader yet; skip it and keep the readable ones.
                Err(error) => {
                    warn!(
                        "Plugin {} registered item {} with component {} that cannot be read, \
                         skipping the component: {error}",
                        self.name.as_deref().unwrap_or("<unknown>"),
                        definition.key,
                        id.to_name(),
                    );
                }
            }
        }

        let result = Item::register_dynamic(ItemRegistration {
            key: definition.key,
            components,
            max_stack_size: definition.max_stack_size,
        });
        Ok(match result {
            Ok(item) => {
                info!(
                    "Plugin {} registered item {} with id {}",
                    self.name.as_deref().unwrap_or("<unknown>"),
                    item.registry_key,
                    item.id
                );
                Ok(u32::from(item.id))
            }
            Err(error) => Err(error.to_string()),
        })
    }

    fn register_item_tag(
        &mut self,
        tag: String,
        entries: Vec<String>,
    ) -> wasmtime::Result<Result<(), String>> {
        if let Err(error) = check_item_registration_allowed(self) {
            return Ok(Err(error));
        }
        dynamic_tag::register_tag(RegistryKey::Item, &tag, entries);
        Ok(Ok(()))
    }

    fn get_item_id(&mut self, key: String) -> wasmtime::Result<Option<u32>> {
        Ok(Item::from_registry_key(&key).map(|item| u32::from(item.id)))
    }

    fn get_item_key(&mut self, id: u32) -> wasmtime::Result<Option<String>> {
        Ok(u16::try_from(id)
            .ok()
            .and_then(Item::from_id)
            .map(|item| item.resource_location().into_owned()))
    }

    fn get_vanilla_item_count(&mut self) -> wasmtime::Result<u32> {
        Ok(u32::from(Item::vanilla_count()))
    }

    fn get_vanilla_items(&mut self) -> wasmtime::Result<Vec<WitItemEntry>> {
        Ok((0..Item::vanilla_count())
            .filter_map(Item::from_vanilla_id)
            .map(to_wit_entry)
            .collect())
    }

    fn get_registered_items(&mut self) -> wasmtime::Result<Vec<WitItemEntry>> {
        Ok(Item::dynamic_items()
            .into_iter()
            .map(to_wit_entry)
            .collect())
    }
}
