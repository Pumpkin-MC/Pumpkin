use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_1_0::pumpkin::{
        self,
        plugin::common::{DamageType, DamageTypeData},
    },
};

impl pumpkin::plugin::common::HostDamageType for PluginHostState {
    async fn new(&mut self, _data: DamageTypeData) -> Resource<DamageType> {
        todo!()
    }

    async fn get_data(&mut self, _damage_type: Resource<DamageType>) -> DamageTypeData {
        todo!()
    }

    async fn set_data(&mut self, _damage_type: Resource<DamageType>, _data: DamageTypeData) {
        todo!()
    }

    async fn from_name(
        &mut self,
        _damage_type: Resource<DamageType>,
        _name: String,
    ) -> Resource<DamageType> {
        todo!()
    }

    async fn drop(&mut self, _rep: Resource<DamageType>) -> wasmtime::Result<()> {
        todo!()
    }
}
impl pumpkin::plugin::common::Host for PluginHostState {}
