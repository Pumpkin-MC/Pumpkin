use super::AccessorExt;
use crate::pumpkin::plugin::{
    common::Hand as WitHand,
    inventory::{
        Host as InventoryHost, HostInventory, HostInventoryWithStore, HostPlayerInventory,
        HostPlayerInventoryWithStore, Inventory as WitInventory,
        PlayerInventory as WitPlayerInventory,
    },
    item_stack::ItemStack as WitHostItemStack,
};
use pumpkin_inventory::{Clearable, Inventory, player::player_inventory::PlayerInventory};
use pumpkin_protocol::{
    codec::item_stack_seralizer::ItemStackSerializer, java::client::play::CSetContainerSlot,
};
use pumpkin_wasm_host_common::state::{InventoryProvider, PluginHostState};
use std::sync::Arc;
use tokio::sync::Mutex;
use wasmtime::component::{Accessor, HasSelf, Resource};

const fn from_wasm_hand(hand: WitHand) -> pumpkin_util::Hand {
    match hand {
        WitHand::Right => pumpkin_util::Hand::Right,
        WitHand::Left => pumpkin_util::Hand::Left,
    }
}

impl InventoryHost for PluginHostState {}

impl HostInventory for PluginHostState {
    fn drop(&mut self, rep: Resource<WitInventory>) -> wasmtime::Result<()> {
        self.drop(rep)
    }

    fn get_size(&mut self, res: Resource<WitInventory>) -> wasmtime::Result<u32> {
        let state = self;
        let provider = state.get(&res)?;
        let size = match provider {
            InventoryProvider::Generic(inv) => inv.size() as u32,
            InventoryProvider::PlayerMain(_) => 36,
            InventoryProvider::PlayerEnderChest(_) => 27,
        };
        Ok(size)
    }

    fn is_empty(&mut self, res: Resource<WitInventory>) -> wasmtime::Result<bool> {
        let state = self;
        let provider = state.get(&res)?;
        let empty = match provider {
            InventoryProvider::Generic(inv) => inv.is_empty(),
            InventoryProvider::PlayerMain(player) => {
                let inv = player.inventory();
                (0..36).all(|slot| inv.get_stack(slot).is_empty())
            }
            InventoryProvider::PlayerEnderChest(player) => {
                let ec = player.ender_chest_inventory();
                (0..27).all(|slot| ec.get_stack(slot).is_empty())
            }
        };
        Ok(empty)
    }

    fn get_item(
        &mut self,
        res: Resource<WitInventory>,
        slot: u32,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let state = self;
        let provider = state.get(&res)?;
        let stack = match provider {
            InventoryProvider::Generic(inv) => {
                let s = inv.get_stack(slot as usize);
                if s.is_empty() { None } else { Some(s) }
            }
            InventoryProvider::PlayerMain(player) => {
                if slot < 36 {
                    let s = player.inventory().get_stack(slot as usize);
                    if s.is_empty() { None } else { Some(s) }
                } else {
                    None
                }
            }
            InventoryProvider::PlayerEnderChest(player) => {
                if slot < 27 {
                    let s = player.ender_chest_inventory().get_stack(slot as usize);
                    if s.is_empty() { None } else { Some(s) }
                } else {
                    None
                }
            }
        };

        if let Some(stack) = stack {
            Ok(Some(state.add(Arc::new(Mutex::new(stack)))?))
        } else {
            Ok(None)
        }
    }

    fn count_item(
        &mut self,
        res: Resource<WitInventory>,
        item_id: String,
    ) -> wasmtime::Result<u32> {
        let state = self;
        let provider = state.get(&res)?;
        let mut total = 0u32;
        let is_matching =
            |key: &str| key == item_id || key.strip_prefix("minecraft:") == Some(&item_id);
        match provider {
            InventoryProvider::Generic(inv) => {
                for slot in 0..inv.size() {
                    let s = inv.get_stack(slot);
                    if !s.is_empty() && is_matching(s.item.registry_key) {
                        total += u32::from(s.item_count);
                    }
                }
            }
            InventoryProvider::PlayerMain(player) => {
                let inv = player.inventory();
                for slot in 0..36 {
                    let s = inv.get_stack(slot);
                    if !s.is_empty() && is_matching(s.item.registry_key) {
                        total += u32::from(s.item_count);
                    }
                }
            }
            InventoryProvider::PlayerEnderChest(player) => {
                let ec = player.ender_chest_inventory();
                for slot in 0..27 {
                    let s = ec.get_stack(slot);
                    if !s.is_empty() && is_matching(s.item.registry_key) {
                        total += u32::from(s.item_count);
                    }
                }
            }
        }
        Ok(total)
    }

    fn get_all_items(
        &mut self,
        res: Resource<WitInventory>,
    ) -> wasmtime::Result<Vec<Option<Resource<WitHostItemStack>>>> {
        let size = self.get_size(Resource::new_borrow(res.rep()))?;
        let mut items = Vec::with_capacity(size as usize);
        for slot in 0..size {
            let item = self.get_item(Resource::new_borrow(res.rep()), slot)?;
            items.push(item);
        }
        Ok(items)
    }

    fn contains_item(
        &mut self,
        res: Resource<WitInventory>,
        item_id: String,
    ) -> wasmtime::Result<bool> {
        let count = self.count_item(res, item_id)?;
        Ok(count > 0)
    }
}

impl HostInventoryWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn set_item(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
        slot: u32,
        item: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let stack = if let Some(stack_res) = item {
            accessor.take_res(stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };

        let provider = accessor.get_res(&res)?;
        match provider {
            InventoryProvider::Generic(inv) => {
                inv.set_stack(slot as usize, stack);
            }
            InventoryProvider::PlayerMain(player) => {
                if slot < 36 {
                    player.inventory().set_stack(slot as usize, stack.clone());
                    let stack_serializer = ItemStackSerializer::from(stack);
                    let packet = CSetContainerSlot::new(0, 0, slot as i16, &stack_serializer);
                    player.send_client_packet(&packet).await;
                }
            }
            InventoryProvider::PlayerEnderChest(player) => {
                if slot < 27 {
                    player
                        .ender_chest_inventory()
                        .set_stack(slot as usize, stack);
                }
            }
        }
        Ok(())
    }

    async fn remove_item(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
        slot: u32,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let provider = accessor.get_res(&res)?;
        let old_stack = match provider {
            InventoryProvider::Generic(inv) => {
                let s = inv.remove_stack(slot as usize);
                if s.is_empty() { None } else { Some(s) }
            }
            InventoryProvider::PlayerMain(player) => {
                if slot < 36 {
                    let s = player.inventory().get_stack(slot as usize);
                    player.inventory().set_stack(
                        slot as usize,
                        pumpkin_data::item_stack::ItemStack::EMPTY.clone(),
                    );
                    let empty_serializer = ItemStackSerializer::from(
                        pumpkin_data::item_stack::ItemStack::EMPTY.clone(),
                    );
                    let packet = CSetContainerSlot::new(0, 0, slot as i16, &empty_serializer);
                    player.send_client_packet(&packet).await;
                    if s.is_empty() { None } else { Some(s) }
                } else {
                    None
                }
            }
            InventoryProvider::PlayerEnderChest(player) => {
                if slot < 27 {
                    let s = player.ender_chest_inventory().get_stack(slot as usize);
                    player.ender_chest_inventory().set_stack(
                        slot as usize,
                        pumpkin_data::item_stack::ItemStack::EMPTY.clone(),
                    );
                    if s.is_empty() { None } else { Some(s) }
                } else {
                    None
                }
            }
        };

        if let Some(stack) = old_stack {
            Ok(Some(accessor.add_res(Arc::new(Mutex::new(stack)))?))
        } else {
            Ok(None)
        }
    }

    async fn clear(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
    ) -> wasmtime::Result<()> {
        let provider = accessor.get_res(&res)?;
        match provider {
            InventoryProvider::Generic(inv) => {
                inv.clear();
            }
            InventoryProvider::PlayerMain(player) => {
                for slot in 0..36 {
                    player
                        .inventory()
                        .set_stack(slot, pumpkin_data::item_stack::ItemStack::EMPTY.clone());
                    let empty_serializer = ItemStackSerializer::from(
                        pumpkin_data::item_stack::ItemStack::EMPTY.clone(),
                    );
                    let packet = CSetContainerSlot::new(0, 0, slot as i16, &empty_serializer);
                    player.send_client_packet(&packet).await;
                }
            }
            InventoryProvider::PlayerEnderChest(player) => {
                player.ender_chest_inventory().clear();
            }
        }
        Ok(())
    }

    // Fixme: this method causes an unnecessary amount of resource table lookups

    // Fixme: this method causes an unnecessary amount of resource table lookups
    async fn set_all_items(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
        items: Vec<Option<Resource<WitHostItemStack>>>,
    ) -> wasmtime::Result<()> {
        let size =
            accessor.with(|mut host| host.get().get_size(Resource::new_borrow(res.rep())))?;
        for (slot, item) in items.into_iter().take(size as usize).enumerate() {
            Self::set_item(accessor, Resource::new_borrow(res.rep()), slot as u32, item).await?;
        }
        Ok(())
    }
}

impl HostPlayerInventory for PluginHostState {
    fn drop(&mut self, rep: Resource<WitPlayerInventory>) -> wasmtime::Result<()> {
        self.drop(rep)
    }

    fn as_inventory(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Resource<WitInventory>> {
        let state = self;
        let player = state.get(&res)?.clone();
        state.add(InventoryProvider::PlayerMain(player))
    }

    fn get_item_in_hand(
        &mut self,
        res: Resource<WitPlayerInventory>,
        hand: WitHand,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let state = self;
        let player = state.get(&res)?;
        let hand = from_wasm_hand(hand);
        let stack = player.inventory().get_stack_in_hand(hand);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(state.add(Arc::new(Mutex::new(stack)))?))
        }
    }

    fn get_selected_slot(&mut self, res: Resource<WitPlayerInventory>) -> wasmtime::Result<u8> {
        let state = self;
        let player = state.get(&res)?;
        Ok(player.inventory().get_selected_slot())
    }

    fn set_selected_slot(
        &mut self,
        res: Resource<WitPlayerInventory>,
        slot: u8,
    ) -> wasmtime::Result<()> {
        let state = self;
        let player = state.get(&res)?;
        if slot < 9 {
            player.inventory().set_selected_slot(slot);
        }
        Ok(())
    }

    fn get_helmet(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let state = self;
        let player = state.get(&res)?;
        let stack = player.inventory().get_slot(39);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(state.add(Arc::new(Mutex::new(stack)))?))
        }
    }

    fn get_chestplate(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let state = self;
        let player = state.get(&res)?;
        let stack = player.inventory().get_slot(38);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(state.add(Arc::new(Mutex::new(stack)))?))
        }
    }

    fn get_leggings(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let state = self;
        let player = state.get(&res)?;
        let stack = player.inventory().get_slot(37);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(state.add(Arc::new(Mutex::new(stack)))?))
        }
    }

    fn get_boots(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let state = self;
        let player = state.get(&res)?;
        let stack = player.inventory().get_slot(36);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(state.add(Arc::new(Mutex::new(stack)))?))
        }
    }

    fn get_off_hand(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let state = self;
        let player = state.get(&res)?;
        let stack = player.inventory().get_slot(40);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(state.add(Arc::new(Mutex::new(stack)))?))
        }
    }
}

impl HostPlayerInventoryWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn set_item_in_hand(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitPlayerInventory>,
        hand: WitHand,
        item: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let stack = if let Some(stack_res) = item {
            accessor.take_res(stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };
        let player = accessor.get_res(&res)?;

        let hand = from_wasm_hand(hand);
        let slot = match hand {
            pumpkin_util::Hand::Right => player.inventory().get_selected_slot() as usize,
            pumpkin_util::Hand::Left => PlayerInventory::OFF_HAND_SLOT,
        };

        player.inventory().set_stack(slot, stack.clone());

        // Sync to client
        let stack_serializer = ItemStackSerializer::from(stack);
        let packet = CSetContainerSlot::new(0, 0, slot as i16, &stack_serializer);
        player.send_client_packet(&packet).await;

        Ok(())
    }

    async fn set_helmet(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitPlayerInventory>,
        item: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let stack = if let Some(stack_res) = item {
            accessor.take_res(stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };
        let player = accessor.get_res(&res)?;
        player.inventory().set_slot(39, stack.clone());
        let stack_serializer = ItemStackSerializer::from(stack);
        let packet = CSetContainerSlot::new(0, 0, 5, &stack_serializer);
        player.send_client_packet(&packet).await;
        Ok(())
    }

    async fn set_chestplate(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitPlayerInventory>,
        item: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let stack = if let Some(stack_res) = item {
            accessor.take_res(stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };
        let player = accessor.get_res(&res)?;
        player.inventory().set_slot(38, stack.clone());
        let stack_serializer = ItemStackSerializer::from(stack);
        let packet = CSetContainerSlot::new(0, 0, 6, &stack_serializer);
        player.send_client_packet(&packet).await;
        Ok(())
    }

    async fn set_leggings(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitPlayerInventory>,
        item: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let stack = if let Some(stack_res) = item {
            accessor.take_res(stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };
        let player = accessor.get_res(&res)?;
        player.inventory().set_slot(37, stack.clone());
        let stack_serializer = ItemStackSerializer::from(stack);
        let packet = CSetContainerSlot::new(0, 0, 7, &stack_serializer);
        player.send_client_packet(&packet).await;
        Ok(())
    }

    async fn set_boots(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitPlayerInventory>,
        item: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let stack = if let Some(stack_res) = item {
            accessor.take_res(stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };
        let player = accessor.get_res(&res)?;
        player.inventory().set_slot(36, stack.clone());
        let stack_serializer = ItemStackSerializer::from(stack);
        let packet = CSetContainerSlot::new(0, 0, 8, &stack_serializer);
        player.send_client_packet(&packet).await;
        Ok(())
    }

    async fn set_off_hand(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitPlayerInventory>,
        item: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let stack = if let Some(stack_res) = item {
            accessor.take_res(stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };
        let player = accessor.get_res(&res)?;
        player.inventory().set_slot(40, stack.clone());
        let stack_serializer = ItemStackSerializer::from(stack);
        let packet = CSetContainerSlot::new(0, 0, 45, &stack_serializer);
        player.send_client_packet(&packet).await;
        Ok(())
    }

    // Fixme: this method causes an unnecessary amount of resource table lookups
    async fn clear_armor(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<()> {
        Self::set_helmet(accessor, Resource::new_borrow(res.rep()), None).await?;
        Self::set_chestplate(accessor, Resource::new_borrow(res.rep()), None).await?;
        Self::set_leggings(accessor, Resource::new_borrow(res.rep()), None).await?;
        Self::set_boots(accessor, Resource::new_borrow(res.rep()), None).await?;
        Ok(())
    }

    async fn clear_main(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<()> {
        let inv = accessor.with(|mut host| host.get().as_inventory(res))?;
        let rep = inv.rep();
        Self::clear(accessor, Resource::new_borrow(rep)).await?;
        accessor.take_res::<WitInventory>(Resource::new_own(rep))?;
        Ok(())
    }

    // Fixme: this method causes an unnecessary amount of resource table lookups
    async fn clear_all(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<()> {
        Self::clear_main(accessor, Resource::new_borrow(res.rep())).await?;
        Self::clear_armor(accessor, Resource::new_borrow(res.rep())).await?;
        Self::set_off_hand(accessor, Resource::new_borrow(res.rep()), None).await?;
        Ok(())
    }
}
