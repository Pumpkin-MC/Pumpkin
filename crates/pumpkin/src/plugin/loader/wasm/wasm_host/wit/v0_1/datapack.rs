use crate::data::datapack::DatapackManager;
use crate::plugin::loader::wasm::wasm_host::state::PluginHostState;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::datapack::{
    DatapackInfo as WitDatapackInfo, DatapackManager as WitDatapackManager,
    EnablePosition as WitEnablePosition, Host as DatapackHost, HostDatapackManager,
};
use wasmtime::component::Resource;

impl DatapackHost for PluginHostState {}

impl HostDatapackManager for PluginHostState {
    async fn list_all_packs(
        &mut self,
        _res: Resource<WitDatapackManager>,
    ) -> wasmtime::Result<Vec<WitDatapackInfo>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let packs = DatapackManager::list_all_packs(server);
        Ok(packs.into_iter().map(to_wit_datapack_info).collect())
    }

    async fn list_enabled_packs(
        &mut self,
        _res: Resource<WitDatapackManager>,
    ) -> wasmtime::Result<Vec<WitDatapackInfo>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let packs = DatapackManager::list_enabled_packs(server);
        Ok(packs.into_iter().map(to_wit_datapack_info).collect())
    }

    async fn list_available_packs(
        &mut self,
        _res: Resource<WitDatapackManager>,
    ) -> wasmtime::Result<Vec<WitDatapackInfo>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let packs = DatapackManager::list_available_packs(server);
        Ok(packs.into_iter().map(to_wit_datapack_info).collect())
    }

    async fn get_pack(
        &mut self,
        _res: Resource<WitDatapackManager>,
        name: String,
    ) -> wasmtime::Result<Option<WitDatapackInfo>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let pack = DatapackManager::get_pack_info(server, &name);
        Ok(pack.map(to_wit_datapack_info))
    }

    async fn is_enabled(
        &mut self,
        _res: Resource<WitDatapackManager>,
        name: String,
    ) -> wasmtime::Result<bool> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        Ok(DatapackManager::is_pack_enabled(server, &name))
    }

    async fn enable_pack(
        &mut self,
        _res: Resource<WitDatapackManager>,
        name: String,
        position: WitEnablePosition,
    ) -> wasmtime::Result<Result<(), String>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let pos = to_data_enable_position(position);
        Ok(DatapackManager::enable_pack(server, &name, pos))
    }

    async fn disable_pack(
        &mut self,
        _res: Resource<WitDatapackManager>,
        name: String,
    ) -> wasmtime::Result<Result<(), String>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        Ok(DatapackManager::disable_pack(server, &name))
    }

    async fn reload(
        &mut self,
        _res: Resource<WitDatapackManager>,
    ) -> wasmtime::Result<Result<(), String>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        Ok(DatapackManager::reload(server))
    }

    async fn execute_function(
        &mut self,
        _res: Resource<WitDatapackManager>,
        name: String,
    ) -> wasmtime::Result<Result<u32, String>> {
        let server = self
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let result = DatapackManager::execute_function_from_console(server, &name);
        Ok(result.map(|count| count as u32))
    }

    async fn drop(&mut self, rep: Resource<WitDatapackManager>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<crate::plugin::loader::wasm::wasm_host::state::DatapackManagerResource>(
            Resource::new_own(rep.rep()),
        );
        Ok(())
    }
}

fn to_wit_datapack_info(info: crate::data::datapack::DatapackInfo) -> WitDatapackInfo {
    WitDatapackInfo {
        id: info.id,
        name: info.name,
        description: info.description,
        pack_format: info.pack_format,
        is_enabled: info.is_enabled,
        recipe_count: info.recipe_count as u32,
        function_count: info.function_count as u32,
    }
}

fn to_data_enable_position(
    pos: WitEnablePosition,
) -> crate::data::datapack::DatapackEnablePosition {
    match pos {
        WitEnablePosition::First => crate::data::datapack::DatapackEnablePosition::First,
        WitEnablePosition::Last => crate::data::datapack::DatapackEnablePosition::Last,
        WitEnablePosition::Before(s) => crate::data::datapack::DatapackEnablePosition::Before(s),
        WitEnablePosition::After(s) => crate::data::datapack::DatapackEnablePosition::After(s),
    }
}
