use std::sync::Arc;
use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::{ItemStackResource, PluginHostState},
    wit::v0_1::pumpkin::plugin::item::{Host, HostPdcItemStack, PdcItemStack},
};

impl Host for PluginHostState {}

fn item_stack_from_resource(
    state: &PluginHostState,
    item: &Resource<PdcItemStack>,
) -> wasmtime::Result<Arc<tokio::sync::Mutex<pumpkin_data::item_stack::ItemStack>>> {
    state
        .resource_table
        .get::<ItemStackResource>(&Resource::new_own(item.rep()))
        .map_err(|_| wasmtime::Error::msg("invalid pdc-item-stack resource handle"))
        .map(|resource| resource.provider.clone())
}

impl HostPdcItemStack for PluginHostState {
    async fn get_registry_key(&mut self, item: Resource<PdcItemStack>) -> wasmtime::Result<String> {
        let item_arc = item_stack_from_resource(self, &item)?;
        let stack = item_arc.lock().await;
        Ok(stack.item.registry_key.to_string())
    }

    async fn get_count(&mut self, item: Resource<PdcItemStack>) -> wasmtime::Result<u8> {
        let item_arc = item_stack_from_resource(self, &item)?;
        let stack = item_arc.lock().await;
        Ok(stack.item_count)
    }

    async fn pdc_has(
        &mut self,
        item: Resource<PdcItemStack>,
        key: String,
    ) -> wasmtime::Result<Result<bool, String>> {
        let item_arc = item_stack_from_resource(self, &item)?;
        let stack = item_arc.lock().await;
        Ok(stack.pdc_has(&key))
    }

    async fn pdc_get(
        &mut self,
        item: Resource<PdcItemStack>,
        key: String,
    ) -> wasmtime::Result<Result<Option<Vec<u8>>, String>> {
        let item_arc = item_stack_from_resource(self, &item)?;
        let stack = item_arc.lock().await;
        Ok(stack.pdc_get(&key))
    }

    async fn pdc_set(
        &mut self,
        item: Resource<PdcItemStack>,
        key: String,
        value: Vec<u8>,
    ) -> wasmtime::Result<Result<(), String>> {
        let item_arc = item_stack_from_resource(self, &item)?;
        let mut stack = item_arc.lock().await;
        Ok(stack.pdc_set(key, value))
    }

    async fn pdc_remove(
        &mut self,
        item: Resource<PdcItemStack>,
        key: String,
    ) -> wasmtime::Result<Result<bool, String>> {
        let item_arc = item_stack_from_resource(self, &item)?;
        let mut stack = item_arc.lock().await;
        Ok(stack.pdc_remove(&key))
    }

    async fn pdc_keys(&mut self, item: Resource<PdcItemStack>) -> wasmtime::Result<Vec<String>> {
        let item_arc = item_stack_from_resource(self, &item)?;
        let stack = item_arc.lock().await;
        Ok(stack.pdc_keys())
    }

    async fn drop(&mut self, rep: Resource<PdcItemStack>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<ItemStackResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
