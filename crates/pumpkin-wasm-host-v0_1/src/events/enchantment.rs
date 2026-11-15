use pumpkin_data::item_stack::ItemStack;
use std::sync::Arc;
use tokio::sync::Mutex;
use wasmtime::component::Resource;

use crate::{
    events::{ToFromWasmEvent, consume_player},
    item_stack::{from_wit_enchantment, to_wit_enchantment},
    pumpkin::plugin::event::{
        EnchantItemEventData, EnchantmentOffer, EnchantmentValue as WitEnchantmentValue, Event,
        PrepareItemEnchantEventData,
    },
};
use pumpkin_core::plugin::{EnchantItemEvent, PrepareItemEnchantEvent};
use pumpkin_wasm_host_common::state::PluginHostState;

fn consume_item_stack(
    state: &mut PluginHostState,
    item: &Resource<crate::pumpkin::plugin::item_stack::ItemStack>,
) -> ItemStack {
    let mutex = state
        .resource_table
        .delete::<Arc<Mutex<ItemStack>>>(Resource::new_own(item.rep()))
        .expect("invalid item stack resource handle");
    mutex.try_lock().expect("lock item stack").clone()
}

impl ToFromWasmEvent for PrepareItemEnchantEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let item = state
            .add(Arc::new(Mutex::new(self.item.clone())))
            .expect("failed to add item stack resource");

        let offers = self
            .offers
            .iter()
            .map(|o| EnchantmentOffer {
                cost: o.cost,
                enchantment_id: o.enchantment_id,
                enchantment_level: o.enchantment_level,
            })
            .collect();

        Event::PrepareItemEnchantEvent(PrepareItemEnchantEventData {
            player,
            item,
            offers,
            bookshelf_count: self.bookshelf_count,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PrepareItemEnchantEvent(data) => {
                let offers = data
                    .offers
                    .into_iter()
                    .map(|o| {
                        pumpkin_core::plugin::EnchantmentOffer::new(
                            o.cost,
                            o.enchantment_id,
                            o.enchantment_level,
                        )
                    })
                    .collect();
                Self {
                    player: consume_player(state, &data.player),
                    item: consume_item_stack(state, &data.item),
                    offers,
                    bookshelf_count: data.bookshelf_count,
                    cancelled: data.cancelled,
                }
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for EnchantItemEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let item = state
            .add(Arc::new(Mutex::new(self.item.clone())))
            .expect("failed to add item stack resource");

        let enchantments_to_add = self
            .enchantments_to_add
            .iter()
            .map(|ev| WitEnchantmentValue {
                enchantment: to_wit_enchantment(ev.enchantment),
                level: ev.level,
            })
            .collect();

        Event::EnchantItemEvent(EnchantItemEventData {
            player,
            item,
            option: self.option,
            cost: self.cost,
            enchantments_to_add,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::EnchantItemEvent(data) => {
                let enchantments_to_add = data
                    .enchantments_to_add
                    .into_iter()
                    .map(|ev| {
                        pumpkin_core::plugin::EnchantmentValue::new(
                            from_wit_enchantment(ev.enchantment),
                            ev.level,
                        )
                    })
                    .collect();
                Self {
                    player: consume_player(state, &data.player),
                    item: consume_item_stack(state, &data.item),
                    option: data.option,
                    cost: data.cost,
                    enchantments_to_add,
                    cancelled: data.cancelled,
                }
            }
            _ => panic!("unexpected event type"),
        }
    }
}
