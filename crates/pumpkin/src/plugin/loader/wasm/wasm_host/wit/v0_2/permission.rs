use crate::plugin::loader::wasm::wasm_host::{state::PluginHostState, wit::v0_2::pumpkin};

impl pumpkin::plugin::permission::Host for PluginHostState {}
