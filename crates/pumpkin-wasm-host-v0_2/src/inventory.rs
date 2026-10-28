use super::AccessorExt;
use crate::pumpkin::plugin::inventory::{HostInventoryWithStore, HostPlayerInventoryWithStore};

use std::sync::Arc;
use tokio::sync::Mutex;
use wasmtime::component::{Accessor, HasSelf, Resource};

use crate::pumpkin::plugin::{
    common::Hand as WitHand,
    inventory::{
        Host as InventoryHost, HostInventory, HostPlayerInventory, Inventory as WitInventory,
        PlayerInventory as WitPlayerInventory,
    },
    item_stack::ItemStack as WitHostItemStack,
};
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::{Clearable, Inventory};
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::{CSetContainerSlot, CSetPlayerInventory};
use pumpkin_wasm_host_common::state::{InventoryProvider, PluginHostState};

const fn from_wasm_hand(hand: WitHand) -> pumpkin_util::Hand {
    match hand {
        WitHand::Right => pumpkin_util::Hand::Right,
        WitHand::Left => pumpkin_util::Hand::Left,
    }
}

fn provider_size(provider: &InventoryProvider) -> usize {
    match provider {
        InventoryProvider::Generic(inv) => inv.size(),
        InventoryProvider::PlayerMain(_) => 36,
        InventoryProvider::PlayerEnderChest(_) => 27,
    }
}

fn provider_max_stack_size(provider: &InventoryProvider) -> u8 {
    match provider {
        InventoryProvider::Generic(inv) => inv.get_max_count_per_stack(),
        InventoryProvider::PlayerMain(player) => player.inventory().get_max_count_per_stack(),
        InventoryProvider::PlayerEnderChest(player) => {
            player.ender_chest_inventory().get_max_count_per_stack()
        }
    }
}

fn provider_get_slot(
    provider: &InventoryProvider,
    slot: usize,
) -> pumpkin_data::item_stack::ItemStack {
    match provider {
        InventoryProvider::Generic(inv) => inv.get_stack(slot),
        InventoryProvider::PlayerMain(player) => {
            if slot < 36 {
                player.inventory().get_stack(slot)
            } else {
                pumpkin_data::item_stack::ItemStack::EMPTY.clone()
            }
        }
        InventoryProvider::PlayerEnderChest(player) => {
            if slot < 27 {
                player.ender_chest_inventory().get_stack(slot)
            } else {
                pumpkin_data::item_stack::ItemStack::EMPTY.clone()
            }
        }
    }
}

fn provider_set_slot(
    provider: &InventoryProvider,
    slot: usize,
    stack: pumpkin_data::item_stack::ItemStack,
) {
    match provider {
        InventoryProvider::Generic(inv) => {
            inv.set_stack(slot, stack);
        }
        InventoryProvider::PlayerMain(player) => {
            if slot < 36 {
                player.inventory().set_stack(slot, stack.clone());
                player.sync_hand_slot(slot, stack);
            }
        }
        InventoryProvider::PlayerEnderChest(player) => {
            if slot < 27 {
                player.ender_chest_inventory().set_stack(slot, stack);
            }
        }
    }
}

fn item_id_matches(registry_key: &str, item_id: &str) -> bool {
    let normalized = item_id.strip_prefix("minecraft:").unwrap_or(item_id);
    registry_key == normalized || registry_key.strip_prefix("minecraft:") == Some(normalized)
}

fn insert_into_provider(
    provider: &InventoryProvider,
    mut stack: pumpkin_data::item_stack::ItemStack,
) -> Option<pumpkin_data::item_stack::ItemStack> {
    if stack.is_empty() {
        return None;
    }
    let size = provider_size(provider);
    let max_per_slot = stack
        .get_max_stack_size()
        .min(provider_max_stack_size(provider));

    if max_per_slot > 1 && stack.get_damage() == 0 {
        for slot in 0..size {
            if stack.is_empty() {
                break;
            }
            let mut existing = provider_get_slot(provider, slot);
            if !existing.is_empty()
                && existing.item_count < max_per_slot
                && existing.are_items_and_components_equal(&stack)
            {
                let transfer = stack.item_count.min(max_per_slot - existing.item_count);
                if transfer > 0 {
                    existing.increment(transfer);
                    stack.decrement(transfer);
                    provider_set_slot(provider, slot, existing);
                }
            }
        }
    }

    for slot in 0..size {
        if stack.is_empty() {
            break;
        }
        let existing = provider_get_slot(provider, slot);
        if existing.is_empty() {
            let transfer = stack.item_count.min(max_per_slot);
            if transfer > 0 {
                let inserted = stack.split(transfer);
                provider_set_slot(provider, slot, inserted);
            }
        }
    }

    if stack.is_empty() { None } else { Some(stack) }
}

impl InventoryHost for PluginHostState {}

impl HostInventory for PluginHostState {
    fn get_size(&mut self, res: Resource<WitInventory>) -> wasmtime::Result<u32> {
        let provider = self.get(&res)?;
        let size = match provider {
            InventoryProvider::Generic(inv) => inv.size() as u32,
            InventoryProvider::PlayerMain(_) => 36,
            InventoryProvider::PlayerEnderChest(_) => 27,
        };
        Ok(size)
    }

    fn is_empty(&mut self, res: Resource<WitInventory>) -> wasmtime::Result<bool> {
        let provider = self.get(&res)?;
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
        let provider = self.get(&res)?;
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
            Ok(Some(self.add(Arc::new(Mutex::new(stack)))?))
        } else {
            Ok(None)
        }
    }

    // Fixme: this method causes an unnecessary amount of resource table lookups
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

    fn get_storage_contents(
        &mut self,
        res: Resource<WitInventory>,
    ) -> wasmtime::Result<Vec<Option<Resource<WitHostItemStack>>>> {
        self.get_all_items(res)
    }

    fn count_item(
        &mut self,
        res: Resource<WitInventory>,
        item_id: String,
    ) -> wasmtime::Result<u32> {
        let provider = self.get(&res)?;
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

    fn contains_item(
        &mut self,
        res: Resource<WitInventory>,
        item_id: String,
    ) -> wasmtime::Result<bool> {
        let count = self.count_item(res, item_id)?;
        Ok(count > 0)
    }

    fn get_max_stack_size(&mut self, res: Resource<WitInventory>) -> wasmtime::Result<u8> {
        let provider = self.get(&res)?;
        Ok(provider_max_stack_size(provider))
    }

    fn first_empty(&mut self, res: Resource<WitInventory>) -> wasmtime::Result<Option<u32>> {
        let provider = self.get(&res)?;
        let size = provider_size(provider);
        for slot in 0..size {
            if provider_get_slot(provider, slot).is_empty() {
                return Ok(Some(slot as u32));
            }
        }
        Ok(None)
    }

    fn first_by_id(
        &mut self,
        res: Resource<WitInventory>,
        item_id: String,
    ) -> wasmtime::Result<Option<u32>> {
        let provider = self.get(&res)?;
        let size = provider_size(provider);
        for slot in 0..size {
            let s = provider_get_slot(provider, slot);
            if !s.is_empty() && item_id_matches(s.item.registry_key, &item_id) {
                return Ok(Some(slot as u32));
            }
        }
        Ok(None)
    }

    fn all_slots_by_id(
        &mut self,
        res: Resource<WitInventory>,
        item_id: String,
    ) -> wasmtime::Result<Vec<u32>> {
        let provider = self.get(&res)?;
        let size = provider_size(provider);
        let mut slots = Vec::new();
        for slot in 0..size {
            let s = provider_get_slot(provider, slot);
            if !s.is_empty() && item_id_matches(s.item.registry_key, &item_id) {
                slots.push(slot as u32);
            }
        }
        Ok(slots)
    }

    fn drop(&mut self, rep: Resource<WitInventory>) -> wasmtime::Result<()> {
        self.drop(rep)
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
                    player.sync_hand_slot(slot as usize, stack);
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
                    player.sync_hand_slot(
                        slot as usize,
                        pumpkin_data::item_stack::ItemStack::EMPTY.clone(),
                    );
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
                    player.try_send_slot_set_packet(&CSetPlayerInventory::new(
                        (slot as i32).into(),
                        &ItemStackSerializer::from(
                            pumpkin_data::item_stack::ItemStack::EMPTY.clone(),
                        ),
                    ));
                }
                player.sync_hand_slot(
                    player.inventory().get_selected_slot() as usize,
                    pumpkin_data::item_stack::ItemStack::EMPTY.clone(),
                );
            }
            InventoryProvider::PlayerEnderChest(player) => {
                player.ender_chest_inventory().clear();
            }
        }
        Ok(())
    }

    // Fixme: this method causes an unnecessary amount of resource table lookups
    async fn set_all_items(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
        items: Vec<Option<Resource<WitHostItemStack>>>,
    ) -> wasmtime::Result<()> {
        let size =
            accessor.with(|mut host| host.get().get_size(Resource::new_borrow(res.rep())))?;
        let mut items = items.into_iter();
        for (slot, item) in items.by_ref().take(size as usize).enumerate() {
            Self::set_item(accessor, Resource::new_borrow(res.rep()), slot as u32, item).await?;
        }
        for extra in items.flatten() {
            accessor.take_res(extra)?;
        }
        Ok(())
    }

    async fn set_storage_contents(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
        items: Vec<Option<Resource<WitHostItemStack>>>,
    ) -> wasmtime::Result<()> {
        Self::set_all_items(accessor, res, items).await
    }

    async fn add_item(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
        item: Resource<WitHostItemStack>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let stack = accessor.take_res(item)?.lock().await.clone();
        let provider = accessor.get_res(&res)?;
        let leftover = insert_into_provider(&provider, stack);
        if let Some(leftover) = leftover {
            Ok(Some(accessor.add_res(Arc::new(Mutex::new(leftover)))?))
        } else {
            Ok(None)
        }
    }

    async fn add_items(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
        items: Vec<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<Vec<Resource<WitHostItemStack>>> {
        let provider = accessor.get_res(&res)?;
        let mut leftovers = Vec::new();
        for item_res in items {
            let stack = accessor.take_res(item_res)?.lock().await.clone();
            if let Some(leftover) = insert_into_provider(&provider, stack) {
                leftovers.push(accessor.add_res(Arc::new(Mutex::new(leftover)))?);
            }
        }
        Ok(leftovers)
    }

    async fn remove_item_stack(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
        item: Resource<WitHostItemStack>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let mut target = accessor.take_res(item)?.lock().await.clone();
        if target.is_empty() {
            return Ok(None);
        }
        let provider = accessor.get_res(&res)?;
        let size = provider_size(&provider);

        for slot in (0..size).rev() {
            if target.is_empty() {
                break;
            }
            let mut existing = provider_get_slot(&provider, slot);
            if !existing.is_empty() && existing.are_items_and_components_equal(&target) {
                let to_remove = existing.item_count.min(target.item_count);
                existing.decrement(to_remove);
                target.decrement(to_remove);
                provider_set_slot(&provider, slot, existing);
            }
        }

        if target.is_empty() {
            Ok(None)
        } else {
            Ok(Some(accessor.add_res(Arc::new(Mutex::new(target)))?))
        }
    }

    async fn remove_item_stacks(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
        items: Vec<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<Vec<Resource<WitHostItemStack>>> {
        let mut leftovers = Vec::new();
        for item in items {
            if let Some(leftover) =
                Self::remove_item_stack(accessor, Resource::new_borrow(res.rep()), item).await?
            {
                leftovers.push(leftover);
            }
        }
        Ok(leftovers)
    }

    async fn remove_matching(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
        item: Resource<WitHostItemStack>,
    ) -> wasmtime::Result<u32> {
        let target = accessor.take_res(item)?.lock().await.clone();
        if target.is_empty() {
            return Ok(0);
        }
        let provider = accessor.get_res(&res)?;
        let size = provider_size(&provider);
        let mut remaining = u32::from(target.item_count);

        for slot in 0..size {
            if remaining == 0 {
                break;
            }
            let mut existing = provider_get_slot(&provider, slot);
            if !existing.is_empty() && existing.are_items_and_components_equal(&target) {
                let take = u32::from(existing.item_count).min(remaining);
                existing.decrement(take as u8);
                remaining -= take;
                let updated = if existing.is_empty() {
                    pumpkin_data::item_stack::ItemStack::EMPTY.clone()
                } else {
                    existing
                };
                provider_set_slot(&provider, slot, updated);
            }
        }

        Ok(remaining)
    }

    async fn remove_by_id(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
        item_id: String,
        amount: u32,
    ) -> wasmtime::Result<u32> {
        if amount == 0 {
            return Ok(0);
        }
        let provider = accessor.get_res(&res)?;
        let size = provider_size(&provider);
        let mut remaining = amount;

        for slot in 0..size {
            if remaining == 0 {
                break;
            }
            let mut existing = provider_get_slot(&provider, slot);
            if !existing.is_empty() && item_id_matches(existing.item.registry_key, &item_id) {
                let take = u32::from(existing.item_count).min(remaining);
                existing.decrement(take as u8);
                remaining -= take;
                let updated = if existing.is_empty() {
                    pumpkin_data::item_stack::ItemStack::EMPTY.clone()
                } else {
                    existing
                };
                provider_set_slot(&provider, slot, updated);
            }
        }

        Ok(remaining)
    }

    async fn remove_all_by_id(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
        item_id: String,
    ) -> wasmtime::Result<u32> {
        let provider = accessor.get_res(&res)?;
        let size = provider_size(&provider);
        let mut removed = 0u32;

        for slot in 0..size {
            let existing = provider_get_slot(&provider, slot);
            if !existing.is_empty() && item_id_matches(existing.item.registry_key, &item_id) {
                removed += u32::from(existing.item_count);
                provider_set_slot(
                    &provider,
                    slot,
                    pumpkin_data::item_stack::ItemStack::EMPTY.clone(),
                );
            }
        }

        Ok(removed)
    }

    async fn first_matching(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
        item: Resource<WitHostItemStack>,
        check_amount: bool,
    ) -> wasmtime::Result<Option<u32>> {
        let target = accessor.take_res(item)?.lock().await.clone();
        if target.is_empty() {
            return Ok(None);
        }
        let provider = accessor.get_res(&res)?;
        let size = provider_size(&provider);
        for slot in 0..size {
            let existing = provider_get_slot(&provider, slot);
            if !existing.is_empty()
                && existing.are_items_and_components_equal(&target)
                && (!check_amount || existing.item_count >= target.item_count)
            {
                return Ok(Some(slot as u32));
            }
        }
        Ok(None)
    }

    async fn contains_at_least(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
        item: Resource<WitHostItemStack>,
        amount: u32,
    ) -> wasmtime::Result<bool> {
        if amount == 0 {
            accessor.take_res(item)?;
            return Ok(true);
        }
        let count = Self::count_matching(accessor, res, item).await?;
        Ok(count >= amount)
    }

    async fn count_matching(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
        item: Resource<WitHostItemStack>,
    ) -> wasmtime::Result<u32> {
        let target = accessor.take_res(item)?.lock().await.clone();
        if target.is_empty() {
            return Ok(0);
        }
        let provider = accessor.get_res(&res)?;
        let size = provider_size(&provider);
        let mut total = 0u32;
        for slot in 0..size {
            let existing = provider_get_slot(&provider, slot);
            if !existing.is_empty() && existing.are_items_and_components_equal(&target) {
                total += u32::from(existing.item_count);
            }
        }
        Ok(total)
    }

    async fn has_space_for(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitInventory>,
        item: Resource<WitHostItemStack>,
    ) -> wasmtime::Result<bool> {
        let target = accessor.take_res(item)?.lock().await.clone();
        if target.is_empty() {
            return Ok(true);
        }
        let provider = accessor.get_res(&res)?;
        let size = provider_size(&provider);
        let max_per_slot = target
            .get_max_stack_size()
            .min(provider_max_stack_size(&provider));
        let mut remaining = u32::from(target.item_count);

        for slot in 0..size {
            if remaining == 0 {
                return Ok(true);
            }
            let existing = provider_get_slot(&provider, slot);
            if existing.is_empty() {
                remaining = remaining.saturating_sub(u32::from(max_per_slot));
            } else if max_per_slot > 1
                && target.get_damage() == 0
                && existing.item_count < max_per_slot
                && existing.are_items_and_components_equal(&target)
            {
                remaining = remaining.saturating_sub(u32::from(max_per_slot - existing.item_count));
            }
        }

        Ok(remaining == 0)
    }
}

impl HostPlayerInventory for PluginHostState {
    fn as_inventory(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Resource<WitInventory>> {
        let player = self.get(&res)?.clone();
        self.add(InventoryProvider::PlayerMain(player))
    }

    fn get_cursor_item(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = self.get(&res)?;
        let screen_handler_arc = player
            .current_screen_handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let handler = screen_handler_arc
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let cursor = handler
            .get_behaviour()
            .cursor_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        drop(handler);
        if cursor.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add(Arc::new(Mutex::new(cursor)))?))
        }
    }

    fn sync(&mut self, res: Resource<WitPlayerInventory>) -> wasmtime::Result<()> {
        let player = self.get(&res)?;
        player.sync_inventory_to_client();
        Ok(())
    }

    fn get_storage_contents(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Vec<Option<Resource<WitHostItemStack>>>> {
        let raw_items = {
            let player = self.get(&res)?;
            let inv = player.inventory();
            let mut list = Vec::with_capacity(36);
            for slot in 0..36 {
                let s = inv.get_stack(slot);
                list.push(if s.is_empty() { None } else { Some(s) });
            }
            list
        };
        let mut items = Vec::with_capacity(36);
        for item in raw_items {
            if let Some(s) = item {
                items.push(Some(self.add(Arc::new(Mutex::new(s)))?));
            } else {
                items.push(None);
            }
        }
        Ok(items)
    }

    fn get_armor_contents(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Vec<Option<Resource<WitHostItemStack>>>> {
        let raw_items = {
            let player = self.get(&res)?;
            let inv = player.inventory();
            let armor_slots = [36, 37, 38, 39];
            let mut list = Vec::with_capacity(4);
            for slot in armor_slots {
                let s = inv.get_stack(slot);
                list.push(if s.is_empty() { None } else { Some(s) });
            }
            list
        };
        let mut items = Vec::with_capacity(4);
        for item in raw_items {
            if let Some(s) = item {
                items.push(Some(self.add(Arc::new(Mutex::new(s)))?));
            } else {
                items.push(None);
            }
        }
        Ok(items)
    }

    fn get_main_hand(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        self.get_item_in_hand(res, WitHand::Right)
    }

    fn get_item_in_hand(
        &mut self,
        res: Resource<WitPlayerInventory>,
        hand: WitHand,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = self.get(&res)?;
        let hand = from_wasm_hand(hand);
        let stack = player.inventory().get_stack_in_hand(hand);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add(Arc::new(Mutex::new(stack)))?))
        }
    }

    fn get_selected_slot(&mut self, res: Resource<WitPlayerInventory>) -> wasmtime::Result<u8> {
        let player = self.get(&res)?;
        Ok(player.inventory().get_selected_slot())
    }

    fn set_selected_slot(
        &mut self,
        res: Resource<WitPlayerInventory>,
        slot: u8,
    ) -> wasmtime::Result<()> {
        let player = self.get(&res)?;
        if slot < 9 {
            player.inventory().set_selected_slot(slot);
        }
        Ok(())
    }

    fn get_helmet(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = self.get(&res)?;
        let stack = player.inventory().get_slot(39);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add(Arc::new(Mutex::new(stack)))?))
        }
    }

    fn get_chestplate(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = self.get(&res)?;
        let stack = player.inventory().get_slot(38);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add(Arc::new(Mutex::new(stack)))?))
        }
    }

    fn get_leggings(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = self.get(&res)?;
        let stack = player.inventory().get_slot(37);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add(Arc::new(Mutex::new(stack)))?))
        }
    }

    fn get_boots(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = self.get(&res)?;
        let stack = player.inventory().get_slot(36);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add(Arc::new(Mutex::new(stack)))?))
        }
    }

    fn get_off_hand(
        &mut self,
        res: Resource<WitPlayerInventory>,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = self.get(&res)?;
        let stack = player.inventory().get_slot(40);
        if stack.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add(Arc::new(Mutex::new(stack)))?))
        }
    }

    fn drop(&mut self, rep: Resource<WitPlayerInventory>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

fn update_cursor_stack(
    player: &pumpkin_core::entity::player::Player,
    stack: &pumpkin_data::item_stack::ItemStack,
) {
    let screen_handler_arc = player
        .current_screen_handler
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let mut handler = screen_handler_arc
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *handler
        .get_behaviour_mut()
        .cursor_stack
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = stack.clone();
}

impl HostPlayerInventoryWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn give_item(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitPlayerInventory>,
        item: Resource<WitHostItemStack>,
        drop_leftover: bool,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let mut stack = accessor.take_res(item)?.lock().await.clone();
        if stack.is_empty() {
            return Ok(None);
        }
        let player = accessor.get_res(&res)?;
        let inv = player.inventory();
        let max_per_slot = stack
            .get_max_stack_size()
            .min(inv.get_max_count_per_stack());

        if max_per_slot > 1 && stack.get_damage() == 0 {
            let selected = inv.get_selected_slot() as usize;
            let priority_slots = [selected, PlayerInventory::OFF_HAND_SLOT];
            for slot in priority_slots.into_iter().chain(0..36) {
                if stack.is_empty() {
                    break;
                }
                let mut existing = inv.get_stack(slot);
                if !existing.is_empty()
                    && existing.item_count < max_per_slot
                    && existing.are_items_and_components_equal(&stack)
                {
                    let transfer = stack.item_count.min(max_per_slot - existing.item_count);
                    if transfer > 0 {
                        existing.increment(transfer);
                        stack.decrement(transfer);
                        inv.set_stack(slot, existing.clone());
                        player.sync_hand_slot(slot, existing);
                    }
                }
            }
        }

        for slot in 0..36 {
            if stack.is_empty() {
                break;
            }
            let existing = inv.get_stack(slot);
            if existing.is_empty() {
                let transfer = stack.item_count.min(max_per_slot);
                if transfer > 0 {
                    let inserted = stack.split(transfer);
                    inv.set_stack(slot, inserted.clone());
                    player.sync_hand_slot(slot, inserted);
                }
            }
        }

        if stack.is_empty() {
            Ok(None)
        } else if drop_leftover {
            super::run_blocking(accessor, move || {
                player.drop_item(stack);
            })
            .await?;
            Ok(None)
        } else {
            Ok(Some(accessor.add_res(Arc::new(Mutex::new(stack)))?))
        }
    }

    async fn set_cursor_item(
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
        update_cursor_stack(&player, &stack);
        let serializer = ItemStackSerializer::from(stack);
        let packet = pumpkin_protocol::java::client::play::CSetCursorItem::new(&serializer);
        player.send_client_packet(&packet).await;
        Ok(())
    }

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
        player.sync_hand_slot(slot, stack);

        Ok(())
    }

    async fn set_storage_contents(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitPlayerInventory>,
        items: Vec<Option<Resource<WitHostItemStack>>>,
    ) -> wasmtime::Result<()> {
        let player = accessor.get_res(&res)?;
        let mut items = items.into_iter();
        for (slot, item) in items.by_ref().take(36).enumerate() {
            let stack = if let Some(item_res) = item {
                accessor.take_res(item_res)?.lock().await.clone()
            } else {
                pumpkin_data::item_stack::ItemStack::EMPTY.clone()
            };
            player.inventory().set_stack(slot, stack.clone());
            player.sync_hand_slot(slot, stack);
        }
        for extra in items.flatten() {
            accessor.take_res(extra)?;
        }
        Ok(())
    }

    async fn set_armor_contents(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitPlayerInventory>,
        items: Vec<Option<Resource<WitHostItemStack>>>,
    ) -> wasmtime::Result<()> {
        let mut iter = items.into_iter();
        let boots = iter.next().flatten();
        let leggings = iter.next().flatten();
        let chestplate = iter.next().flatten();
        let helmet = iter.next().flatten();
        for extra in iter.flatten() {
            accessor.take_res(extra)?;
        }
        Self::set_boots(accessor, Resource::new_borrow(res.rep()), boots).await?;
        Self::set_leggings(accessor, Resource::new_borrow(res.rep()), leggings).await?;
        Self::set_chestplate(accessor, Resource::new_borrow(res.rep()), chestplate).await?;
        Self::set_helmet(accessor, Resource::new_borrow(res.rep()), helmet).await?;
        Ok(())
    }

    async fn set_main_hand(
        accessor: &Accessor<PluginHostState, Self>,
        res: Resource<WitPlayerInventory>,
        item: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        Self::set_item_in_hand(accessor, res, WitHand::Right, item).await
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
