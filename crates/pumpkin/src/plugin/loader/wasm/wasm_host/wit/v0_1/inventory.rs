use std::sync::Arc;
use tokio::sync::Mutex;
use wasmtime::component::Resource;

use crate::entity::player::Player;
use crate::plugin::loader::wasm::wasm_host::{
    state::{InventoryProvider, InventoryResource, PlayerInventoryResource, PluginHostState},
    wit::v0_1::pumpkin::plugin::{
        common::Hand as WitHand,
        inventory::{
            Host as InventoryHost, HostInventory, HostPlayerInventory, Inventory as WitInventory,
            PlayerInventory as WitPlayerInventory,
        },
        item_stack::ItemStack as WitHostItemStack,
    },
};
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::CSetContainerSlot;
use pumpkin_world::inventory::{Clearable, Inventory};

const fn from_wasm_hand(hand: WitHand) -> pumpkin_util::Hand {
    match hand {
        WitHand::Right => pumpkin_util::Hand::Right,
        WitHand::Left => pumpkin_util::Hand::Left,
    }
}

impl InventoryHost for PluginHostState {}

impl PluginHostState {
    fn get_inventory_provider(
        &self,
        res: &Resource<WitInventory>,
    ) -> wasmtime::Result<InventoryProvider> {
        let r = self
            .resource_table
            .get::<InventoryResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(r.provider.clone())
    }

    fn get_player_inventory_player(
        &self,
        res: &Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Arc<Player>> {
        let r = self
            .resource_table
            .get::<PlayerInventoryResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(r.provider.clone())
    }
}

impl HostInventory for PluginHostState {
    async fn get_size(&mut self, res: Resource<WitInventory>) -> wasmtime::Result<u32> {
        let provider = self.get_inventory_provider(&res)?;
        let size = match provider {
            InventoryProvider::Generic(inv) => inv.size() as u32,
            InventoryProvider::PlayerMain(_) => 36,
            InventoryProvider::PlayerEnderChest(_) => 27,
        };
        Ok(size)
    }

    async fn is_empty(&mut self, res: Resource<WitInventory>) -> wasmtime::Result<bool> {
        let provider = self.get_inventory_provider(&res)?;
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

    async fn get_item(
        &mut self,
        res: Resource<WitInventory>,
        slot: u32,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let provider = self.get_inventory_provider(&res)?;
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
            Ok(Some(self.add_item_stack(Arc::new(Mutex::new(stack)))?))
        } else {
            Ok(None)
        }
    }

    async fn set_item(
        &mut self,
        res: Resource<WitInventory>,
        slot: u32,
        item: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let stack = if let Some(stack_res) = item {
            self.get_item_stack(&stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };

        let provider = self.get_inventory_provider(&res)?;
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
        &mut self,
        res: Resource<WitInventory>,
        slot: u32,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let provider = self.get_inventory_provider(&res)?;
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
            Ok(Some(self.add_item_stack(Arc::new(Mutex::new(stack)))?))
        } else {
            Ok(None)
        }
    }

    async fn clear(&mut self, res: Resource<WitInventory>) -> wasmtime::Result<()> {
        let provider = self.get_inventory_provider(&res)?;
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

    async fn get_all_items(
        &mut self,
        res: Resource<WitInventory>,
    ) -> wasmtime::Result<Vec<Option<Resource<WitHostItemStack>>>> {
        let size = self.get_size(Resource::new_own(res.rep())).await?;
        let mut items = Vec::with_capacity(size as usize);
        for slot in 0..size {
            let item = self.get_item(Resource::new_own(res.rep()), slot).await?;
            items.push(item);
        }
        Ok(items)
    }

    async fn set_all_items(
        &mut self,
        res: Resource<WitInventory>,
        items: Vec<Option<Resource<WitHostItemStack>>>,
    ) -> wasmtime::Result<()> {
        let size = self.get_size(Resource::new_own(res.rep())).await?;
        for (slot, item) in items.into_iter().take(size as usize).enumerate() {
            self.set_item(Resource::new_own(res.rep()), slot as u32, item)
                .await?;
        }
        Ok(())
    }

    async fn count_item(
        &mut self,
        res: Resource<WitInventory>,
        item_id: String,
    ) -> wasmtime::Result<u32> {
        let provider = self.get_inventory_provider(&res)?;
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

    async fn contains_item(
        &mut self,
        res: Resource<WitInventory>,
        item_id: String,
    ) -> wasmtime::Result<bool> {
        let count = self.count_item(res, item_id).await?;
        Ok(count > 0)
    }

    async fn drop(&mut self, rep: Resource<WitInventory>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<InventoryResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl HostPlayerInventory for PluginHostState {
    async fn as_inventory(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Resource<WitInventory>> {
        let player = self.get_player_inventory_player(&res)?;
        self.add_inventory(InventoryProvider::PlayerMain(player))
    }

    async fn get_item_in_hand(
        &mut self,
        res: Resource<WitPlayerInventory>,
        hand: WitHand,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = self.get_player_inventory_player(&res)?;
        let hand = from_wasm_hand(hand);
        let stack = player.inventory().get_stack_in_hand(hand);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add_item_stack(Arc::new(Mutex::new(stack)))?))
        }
    }

    async fn set_item_in_hand(
        &mut self,
        res: Resource<WitPlayerInventory>,
        hand: WitHand,
        item: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let player = self.get_player_inventory_player(&res)?;
        let stack = if let Some(stack_res) = item {
            self.get_item_stack(&stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };

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

    async fn get_selected_slot(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<u8> {
        let player = self.get_player_inventory_player(&res)?;
        Ok(player.inventory().get_selected_slot())
    }

    async fn set_selected_slot(
        &mut self,
        res: Resource<WitPlayerInventory>,
        slot: u8,
    ) -> wasmtime::Result<()> {
        let player = self.get_player_inventory_player(&res)?;
        if slot < 9 {
            player.inventory().set_selected_slot(slot);
        }
        Ok(())
    }

    async fn get_helmet(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = self.get_player_inventory_player(&res)?;
        let stack = player.inventory().get_slot(39);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add_item_stack(Arc::new(Mutex::new(stack)))?))
        }
    }

    async fn set_helmet(
        &mut self,
        res: Resource<WitPlayerInventory>,
        item: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let player = self.get_player_inventory_player(&res)?;
        let stack = if let Some(stack_res) = item {
            self.get_item_stack(&stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };
        player.inventory().set_slot(39, stack.clone());
        let stack_serializer = ItemStackSerializer::from(stack);
        let packet = CSetContainerSlot::new(0, 0, 5, &stack_serializer);
        player.send_client_packet(&packet).await;
        Ok(())
    }

    async fn get_chestplate(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = self.get_player_inventory_player(&res)?;
        let stack = player.inventory().get_slot(38);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add_item_stack(Arc::new(Mutex::new(stack)))?))
        }
    }

    async fn set_chestplate(
        &mut self,
        res: Resource<WitPlayerInventory>,
        item: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let player = self.get_player_inventory_player(&res)?;
        let stack = if let Some(stack_res) = item {
            self.get_item_stack(&stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };
        player.inventory().set_slot(38, stack.clone());
        let stack_serializer = ItemStackSerializer::from(stack);
        let packet = CSetContainerSlot::new(0, 0, 6, &stack_serializer);
        player.send_client_packet(&packet).await;
        Ok(())
    }

    async fn get_leggings(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = self.get_player_inventory_player(&res)?;
        let stack = player.inventory().get_slot(37);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add_item_stack(Arc::new(Mutex::new(stack)))?))
        }
    }

    async fn set_leggings(
        &mut self,
        res: Resource<WitPlayerInventory>,
        item: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let player = self.get_player_inventory_player(&res)?;
        let stack = if let Some(stack_res) = item {
            self.get_item_stack(&stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };
        player.inventory().set_slot(37, stack.clone());
        let stack_serializer = ItemStackSerializer::from(stack);
        let packet = CSetContainerSlot::new(0, 0, 7, &stack_serializer);
        player.send_client_packet(&packet).await;
        Ok(())
    }

    async fn get_boots(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = self.get_player_inventory_player(&res)?;
        let stack = player.inventory().get_slot(36);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add_item_stack(Arc::new(Mutex::new(stack)))?))
        }
    }

    async fn set_boots(
        &mut self,
        res: Resource<WitPlayerInventory>,
        item: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let player = self.get_player_inventory_player(&res)?;
        let stack = if let Some(stack_res) = item {
            self.get_item_stack(&stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };
        player.inventory().set_slot(36, stack.clone());
        let stack_serializer = ItemStackSerializer::from(stack);
        let packet = CSetContainerSlot::new(0, 0, 8, &stack_serializer);
        player.send_client_packet(&packet).await;
        Ok(())
    }

    async fn get_off_hand(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = self.get_player_inventory_player(&res)?;
        let stack = player.inventory().get_slot(40);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add_item_stack(Arc::new(Mutex::new(stack)))?))
        }
    }

    async fn set_off_hand(
        &mut self,
        res: Resource<WitPlayerInventory>,
        item: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let player = self.get_player_inventory_player(&res)?;
        let stack = if let Some(stack_res) = item {
            self.get_item_stack(&stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };
        player.inventory().set_slot(40, stack.clone());
        let stack_serializer = ItemStackSerializer::from(stack);
        let packet = CSetContainerSlot::new(0, 0, 45, &stack_serializer);
        player.send_client_packet(&packet).await;
        Ok(())
    }

    async fn clear_armor(&mut self, res: Resource<WitPlayerInventory>) -> wasmtime::Result<()> {
        self.set_helmet(Resource::new_own(res.rep()), None).await?;
        self.set_chestplate(Resource::new_own(res.rep()), None)
            .await?;
        self.set_leggings(Resource::new_own(res.rep()), None)
            .await?;
        self.set_boots(Resource::new_own(res.rep()), None).await?;
        Ok(())
    }

    async fn clear_main(&mut self, res: Resource<WitPlayerInventory>) -> wasmtime::Result<()> {
        let inv = self.as_inventory(res).await?;
        self.clear(inv).await
    }

    async fn clear_all(&mut self, res: Resource<WitPlayerInventory>) -> wasmtime::Result<()> {
        self.clear_main(Resource::new_own(res.rep())).await?;
        self.clear_armor(Resource::new_own(res.rep())).await?;
        self.set_off_hand(Resource::new_own(res.rep()), None)
            .await?;
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<WitPlayerInventory>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<PlayerInventoryResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
