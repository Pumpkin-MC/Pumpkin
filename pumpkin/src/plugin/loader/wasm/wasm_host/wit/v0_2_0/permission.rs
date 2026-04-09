use crate::plugin::loader::wasm::wasm_host::{state::PluginHostState, wit::v0_2_0::pumpkin};

impl pumpkin::plugin::permission::Host for PluginHostState {}
