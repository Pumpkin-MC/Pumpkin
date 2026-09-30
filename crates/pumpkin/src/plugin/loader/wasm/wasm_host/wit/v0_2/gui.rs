use super::AccessorExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use wasmtime::component::Accessor;
use wasmtime::component::HasSelf;
use wasmtime::component::Resource;

use crate::plugin::api::gui::{PluginGui, PluginInventory};
use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_2::pumpkin::plugin::{
        gui::{self, Gui},
        item_stack::ItemStack as WitHostItemStack,
        screens::Screen as WitScreen,
    },
};
use pumpkin_data::screen::WindowType;

#[must_use]
pub const fn to_wit_screen(window_type: WindowType) -> WitScreen {
    match window_type {
        WindowType::Generic9x1 => WitScreen::Generic9x1,
        WindowType::Generic9x2 => WitScreen::Generic9x2,
        WindowType::Generic9x3 => WitScreen::Generic9x3,
        WindowType::Generic9x4 => WitScreen::Generic9x4,
        WindowType::Generic9x5 => WitScreen::Generic9x5,
        WindowType::Generic9x6 => WitScreen::Generic9x6,
        WindowType::Generic3x3 => WitScreen::Generic3x3,
        WindowType::Crafter3x3 => WitScreen::Crafter3x3,
        WindowType::Anvil => WitScreen::Anvil,
        WindowType::Beacon => WitScreen::Beacon,
        WindowType::BlastFurnace => WitScreen::BlastFurnace,
        WindowType::BrewingStand => WitScreen::BrewingStand,
        WindowType::Crafting => WitScreen::Crafting,
        WindowType::Enchantment => WitScreen::Enchantment,
        WindowType::Furnace => WitScreen::Furnace,
        WindowType::Grindstone => WitScreen::Grindstone,
        WindowType::Hopper => WitScreen::Hopper,
        WindowType::Lectern => WitScreen::Lectern,
        WindowType::Loom => WitScreen::Loom,
        WindowType::Merchant => WitScreen::Merchant,
        WindowType::ShulkerBox => WitScreen::ShulkerBox,
        WindowType::Smithing => WitScreen::Smithing,
        WindowType::Smoker => WitScreen::Smoker,
        WindowType::CartographyTable => WitScreen::CartographyTable,
        WindowType::Stonecutter => WitScreen::Stonecutter,
    }
}

#[must_use]
pub const fn from_wit_screen(screen: WitScreen) -> WindowType {
    match screen {
        WitScreen::Generic9x1 => WindowType::Generic9x1,
        WitScreen::Generic9x2 => WindowType::Generic9x2,
        WitScreen::Generic9x3 => WindowType::Generic9x3,
        WitScreen::Generic9x4 => WindowType::Generic9x4,
        WitScreen::Generic9x5 => WindowType::Generic9x5,
        WitScreen::Generic9x6 => WindowType::Generic9x6,
        WitScreen::Generic3x3 => WindowType::Generic3x3,
        WitScreen::Crafter3x3 => WindowType::Crafter3x3,
        WitScreen::Anvil => WindowType::Anvil,
        WitScreen::Beacon => WindowType::Beacon,
        WitScreen::BlastFurnace => WindowType::BlastFurnace,
        WitScreen::BrewingStand => WindowType::BrewingStand,
        WitScreen::Crafting => WindowType::Crafting,
        WitScreen::Enchantment => WindowType::Enchantment,
        WitScreen::Furnace => WindowType::Furnace,
        WitScreen::Grindstone => WindowType::Grindstone,
        WitScreen::Hopper => WindowType::Hopper,
        WitScreen::Lectern => WindowType::Lectern,
        WitScreen::Loom => WindowType::Loom,
        WitScreen::Merchant => WindowType::Merchant,
        WitScreen::ShulkerBox => WindowType::ShulkerBox,
        WitScreen::Smithing => WindowType::Smithing,
        WitScreen::Smoker => WindowType::Smoker,
        WitScreen::CartographyTable => WindowType::CartographyTable,
        WitScreen::Stonecutter => WindowType::Stonecutter,
    }
}

impl gui::Host for PluginHostState {}

impl gui::HostGui for PluginHostState {
    async fn drop(&mut self, rep: Resource<Gui>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl gui::HostGuiWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn create(
        accessor: &Accessor<PluginHostState, Self>,
        screen: WitScreen,
        title: Resource<
            crate::plugin::loader::wasm::wasm_host::wit::v0_2::pumpkin::plugin::text::TextComponent,
        >,
    ) -> wasmtime::Result<Resource<Gui>> {
        accessor.with(|mut host| {
            let state = host.get();
            let title = state.take(title)?;
            let window_type = from_wit_screen(screen);

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
                allow_grab_items: true,
                allow_put_items: true,
            }));

            state.add(gui)
        })
    }


    async fn get_inventory(accessor: &Accessor<PluginHostState, Self>,
        res: Resource<Gui>,
    ) -> wasmtime::Result<
        Resource<
            crate::plugin::loader::wasm::wasm_host::wit::v0_2::pumpkin::plugin::inventory::Inventory,
        >,
    >{
        let inv = {
            let gui_handle = accessor.get_res(&res)?;
            let gui = gui_handle.lock().await;
            gui.inventory.clone()
        };
        accessor
            .add_res(crate::plugin::loader::wasm::wasm_host::state::InventoryProvider::Generic(inv))
    }

    async fn set_item(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<Gui>,
        slot: u32,
        item: Resource<WitHostItemStack>,
    ) -> wasmtime::Result<()> {
        let item_stack = accessor.take_res(item)?.lock().await.clone();
        let gui_handle = accessor.get_res(&res)?;
        let gui = gui_handle.lock().await;
        let mut slots = gui
            .inventory
            .slots
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if (slot as usize) < slots.len() {
            slots[slot as usize] = item_stack;
        }
        Ok(())
    }

    async fn get_item(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<Gui>,
        slot: u32,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let stack = {
            let gui_handle = accessor.get_res(&res)?;
            let gui = gui_handle.lock().await;
            let slots = gui
                .inventory
                .slots
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if (slot as usize) < slots.len() {
                let stack = &slots[slot as usize];
                if stack.is_empty() {
                    None
                } else {
                    Some(stack.clone())
                }
            } else {
                None
            }
        };

        if let Some(stack) = stack {
            Ok(Some(accessor.add_res(Arc::new(Mutex::new(stack)))?))
        } else {
            Ok(None)
        }
    }

    async fn get_type(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<Gui>,
    ) -> wasmtime::Result<WitScreen> {
        let gui_handle = accessor.get_res(&res)?;
        let gui = gui_handle.lock().await;
        Ok(to_wit_screen(gui.window_type))
    }

    async fn get_title(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<Gui>,
    ) -> wasmtime::Result<
        Resource<
            crate::plugin::loader::wasm::wasm_host::wit::v0_2::pumpkin::plugin::text::TextComponent,
        >,
    > {
        let title = {
            let gui_handle = accessor.get_res(&res)?;
            let gui = gui_handle.lock().await;
            gui.title.clone()
        };
        accessor
            .add_res(title)
            .map_err(|_| wasmtime::Error::msg("Failed to add text component resource"))
    }

    async fn get_size(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<Gui>,
    ) -> wasmtime::Result<u32> {
        use pumpkin_inventory::Inventory;
        let gui_handle = accessor.get_res(&res)?;
        let gui = gui_handle.lock().await;
        Ok(gui.inventory.size() as u32)
    }

    async fn clear_items(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<Gui>,
    ) -> wasmtime::Result<()> {
        use pumpkin_inventory::Clearable;
        let gui_handle = accessor.get_res(&res)?;
        let gui = gui_handle.lock().await;
        gui.inventory.clear();
        Ok(())
    }

    async fn set_allow_grab_items(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<Gui>,
        allow: bool,
    ) -> wasmtime::Result<()> {
        let gui_handle = accessor.get_res(&res)?;
        let mut gui = gui_handle.lock().await;
        gui.allow_grab_items = allow;
        Ok(())
    }

    async fn get_allow_grab_items(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<Gui>,
    ) -> wasmtime::Result<bool> {
        let gui_handle = accessor.get_res(&res)?;
        let gui = gui_handle.lock().await;
        Ok(gui.allow_grab_items)
    }

    async fn set_allow_put_items(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<Gui>,
        allow: bool,
    ) -> wasmtime::Result<()> {
        let gui_handle = accessor.get_res(&res)?;
        let mut gui = gui_handle.lock().await;
        gui.allow_put_items = allow;
        Ok(())
    }

    async fn get_allow_put_items(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<Gui>,
    ) -> wasmtime::Result<bool> {
        let gui_handle = accessor.get_res(&res)?;
        let gui = gui_handle.lock().await;
        Ok(gui.allow_put_items)
    }
}
