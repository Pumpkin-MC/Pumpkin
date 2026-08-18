use crate::plugin::loader::wasm::wasm_host::{
    signature, state::PluginHostState, wit::v0_1::pumpkin,
};

impl pumpkin::plugin::marketplace::Host for PluginHostState {
    async fn get_public_key(&mut self) -> wasmtime::Result<Option<String>> {
        Ok(signature::fetch_market_public_key().ok())
    }
}
