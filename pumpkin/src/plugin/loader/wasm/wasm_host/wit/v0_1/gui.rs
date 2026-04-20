use std::sync::Arc;
use tokio::sync::Mutex;
use wasmtime::component::Resource;

use crate::plugin::api::gui::{PluginGui, PluginInventory};
use crate::plugin::loader::wasm::wasm_host::{
    state::{GuiResource, PluginHostState},
    wit::v0_1::pumpkin::plugin::{
        common::ItemStack as WitItemStack,
        gui::{self, Gui, GuiType},
    },
};
use pumpkin_data::item_stack::ItemStack;

impl PluginHostState {
    fn get_gui_res(&self, res: &Resource<Gui>) -> wasmtime::Result<&GuiResource> {
        self.resource_table
            .get::<GuiResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }
}

impl gui::Host for PluginHostState {}

impl gui::HostGui for PluginHostState {
    async fn new(
        &mut self,
        gui_type: GuiType,
        title: Resource<
            crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::text::TextComponent,
        >,
    ) -> wasmtime::Result<Resource<Gui>> {
        let title = self.get_text_provider(&title)?;
        let window_type = match gui_type {
            GuiType::Generic9x1 => pumpkin_data::screen::WindowType::Generic9x1,
            GuiType::Generic9x2 => pumpkin_data::screen::WindowType::Generic9x2,
            GuiType::Generic9x3 => pumpkin_data::screen::WindowType::Generic9x3,
            GuiType::Generic9x4 => pumpkin_data::screen::WindowType::Generic9x4,
            GuiType::Generic9x5 => pumpkin_data::screen::WindowType::Generic9x5,
            GuiType::Generic9x6 => pumpkin_data::screen::WindowType::Generic9x6,
            GuiType::Generic3x3 => pumpkin_data::screen::WindowType::Generic3x3,
            GuiType::Crafter3x3 => pumpkin_data::screen::WindowType::Crafter3x3,
            GuiType::Anvil => pumpkin_data::screen::WindowType::Anvil,
            GuiType::Beacon => pumpkin_data::screen::WindowType::Beacon,
            GuiType::BlastFurnace => pumpkin_data::screen::WindowType::BlastFurnace,
            GuiType::BrewingStand => pumpkin_data::screen::WindowType::BrewingStand,
            GuiType::Crafting => pumpkin_data::screen::WindowType::Crafting,
            GuiType::Enchantment => pumpkin_data::screen::WindowType::Enchantment,
            GuiType::Furnace => pumpkin_data::screen::WindowType::Furnace,
            GuiType::Grindstone => pumpkin_data::screen::WindowType::Grindstone,
            GuiType::Hopper => pumpkin_data::screen::WindowType::Hopper,
            GuiType::Lectern => pumpkin_data::screen::WindowType::Lectern,
            GuiType::Loom => pumpkin_data::screen::WindowType::Loom,
            GuiType::Merchant => pumpkin_data::screen::WindowType::Merchant,
            GuiType::ShulkerBox => pumpkin_data::screen::WindowType::ShulkerBox,
            GuiType::Smithing => pumpkin_data::screen::WindowType::Smithing,
            GuiType::Smoker => pumpkin_data::screen::WindowType::Smoker,
            GuiType::CartographyTable => pumpkin_data::screen::WindowType::CartographyTable,
            GuiType::Stonecutter => pumpkin_data::screen::WindowType::Stonecutter,
        };

        let size = match window_type {
            pumpkin_data::screen::WindowType::Generic9x2 => 18,
            pumpkin_data::screen::WindowType::Generic9x4 => 36,
            pumpkin_data::screen::WindowType::Generic9x5 => 45,
            pumpkin_data::screen::WindowType::Generic9x6 => 54,
            pumpkin_data::screen::WindowType::Generic3x3 => 9,
            pumpkin_data::screen::WindowType::Generic9x1
            | pumpkin_data::screen::WindowType::Hopper => 5,
            _ => 27, // Default
        };

        let gui = Arc::new(Mutex::new(PluginGui {
            window_type,
            title,
            inventory: Arc::new(PluginInventory::new(size)),
        }));

        self.add_gui(gui)
    }

    async fn set_item(
        &mut self,
        res: Resource<Gui>,
        slot: u32,
        item: WitItemStack,
    ) -> wasmtime::Result<()> {
        let gui = self.get_gui_res(&res)?.provider.lock().await;
        if (slot as usize) < gui.inventory.slots.len() {
            let item_id = item.registry_key;
            let item_data = pumpkin_data::item::Item::from_registry_key(&item_id)
                .ok_or_else(|| wasmtime::Error::msg("Invalid item"))?;
            let item_stack = ItemStack::new(item.count, item_data);
            *gui.inventory.slots[slot as usize].lock().await = item_stack;
        }
        Ok(())
    }

    async fn get_item(
        &mut self,
        res: Resource<Gui>,
        slot: u32,
    ) -> wasmtime::Result<Option<WitItemStack>> {
        let gui = self.get_gui_res(&res)?.provider.lock().await;
        if (slot as usize) < gui.inventory.slots.len() {
            let stack = gui.inventory.slots[slot as usize].lock().await;
            if stack.is_empty() {
                Ok(None)
            } else {
                Ok(Some(WitItemStack {
                    registry_key: stack.get_item().registry_key.to_string(),
                    count: stack.item_count,
                }))
            }
        } else {
            Ok(None)
        }
    }

    async fn get_type(&mut self, res: Resource<Gui>) -> wasmtime::Result<GuiType> {
        let gui = self.get_gui_res(&res)?.provider.lock().await;
        Ok(match gui.window_type {
            pumpkin_data::screen::WindowType::Generic9x1 => GuiType::Generic9x1,
            pumpkin_data::screen::WindowType::Generic9x2 => GuiType::Generic9x2,
            pumpkin_data::screen::WindowType::Generic9x3 => GuiType::Generic9x3,
            pumpkin_data::screen::WindowType::Generic9x4 => GuiType::Generic9x4,
            pumpkin_data::screen::WindowType::Generic9x5 => GuiType::Generic9x5,
            pumpkin_data::screen::WindowType::Generic9x6 => GuiType::Generic9x6,
            pumpkin_data::screen::WindowType::Generic3x3 => GuiType::Generic3x3,
            pumpkin_data::screen::WindowType::Crafter3x3 => GuiType::Crafter3x3,
            pumpkin_data::screen::WindowType::Anvil => GuiType::Anvil,
            pumpkin_data::screen::WindowType::Beacon => GuiType::Beacon,
            pumpkin_data::screen::WindowType::BlastFurnace => GuiType::BlastFurnace,
            pumpkin_data::screen::WindowType::BrewingStand => GuiType::BrewingStand,
            pumpkin_data::screen::WindowType::Crafting => GuiType::Crafting,
            pumpkin_data::screen::WindowType::Enchantment => GuiType::Enchantment,
            pumpkin_data::screen::WindowType::Furnace => GuiType::Furnace,
            pumpkin_data::screen::WindowType::Grindstone => GuiType::Grindstone,
            pumpkin_data::screen::WindowType::Hopper => GuiType::Hopper,
            pumpkin_data::screen::WindowType::Lectern => GuiType::Lectern,
            pumpkin_data::screen::WindowType::Loom => GuiType::Loom,
            pumpkin_data::screen::WindowType::Merchant => GuiType::Merchant,
            pumpkin_data::screen::WindowType::ShulkerBox => GuiType::ShulkerBox,
            pumpkin_data::screen::WindowType::Smithing => GuiType::Smithing,
            pumpkin_data::screen::WindowType::Smoker => GuiType::Smoker,
            pumpkin_data::screen::WindowType::CartographyTable => GuiType::CartographyTable,
            pumpkin_data::screen::WindowType::Stonecutter => GuiType::Stonecutter,
        })
    }

    async fn get_title(
        &mut self,
        res: Resource<Gui>,
    ) -> wasmtime::Result<
        Resource<
            crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::text::TextComponent,
        >,
    > {
        let title = {
            let gui = self.get_gui_res(&res)?.provider.lock().await;
            gui.title.clone()
        };
        self.add_text_component(title)
            .map_err(|_| wasmtime::Error::msg("Failed to add text component resource"))
    }

    async fn get_size(&mut self, res: Resource<Gui>) -> wasmtime::Result<u32> {
        use pumpkin_world::inventory::Inventory;
        let gui = self.get_gui_res(&res)?.provider.lock().await;
        Ok(gui.inventory.size() as u32)
    }

    async fn clear_items(&mut self, res: Resource<Gui>) -> wasmtime::Result<()> {
        use pumpkin_world::inventory::Clearable;
        let gui = self.get_gui_res(&res)?.provider.lock().await;
        gui.inventory.clear().await;
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<Gui>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<GuiResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}
