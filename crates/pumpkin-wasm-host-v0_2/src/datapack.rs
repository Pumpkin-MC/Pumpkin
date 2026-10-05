use super::run_blocking;
use crate::pumpkin::plugin::datapack::{
    DatapackInfo as WitDatapackInfo, DatapackManager as WitDatapackManager,
    EnablePosition as WitEnablePosition, Host as DatapackHost, HostDatapackManager,
    HostDatapackManagerWithStore,
};
use pumpkin_core::data::datapack::DatapackManager;
use pumpkin_wasm_host_common::state::PluginHostState;
use wasmtime::component::{Accessor, HasSelf, Resource};

impl DatapackHost for PluginHostState {}

impl HostDatapackManager for PluginHostState {
    fn drop(&mut self, rep: Resource<WitDatapackManager>) -> wasmtime::Result<()> {
        self.drop(rep)
    }

    fn list_all_packs(
        &mut self,
        _res: Resource<WitDatapackManager>,
    ) -> wasmtime::Result<Vec<WitDatapackInfo>> {
        let state = self;
        let server = state
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let packs = DatapackManager::list_all_packs(server);
        Ok(packs.into_iter().map(to_wit_datapack_info).collect())
    }

    fn list_enabled_packs(
        &mut self,
        _res: Resource<WitDatapackManager>,
    ) -> wasmtime::Result<Vec<WitDatapackInfo>> {
        let state = self;
        let server = state
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let packs = DatapackManager::list_enabled_packs(server);
        Ok(packs.into_iter().map(to_wit_datapack_info).collect())
    }

    fn list_available_packs(
        &mut self,
        _res: Resource<WitDatapackManager>,
    ) -> wasmtime::Result<Vec<WitDatapackInfo>> {
        let state = self;
        let server = state
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let packs = DatapackManager::list_available_packs(server);
        Ok(packs.into_iter().map(to_wit_datapack_info).collect())
    }

    fn get_pack(
        &mut self,
        _res: Resource<WitDatapackManager>,
        name: String,
    ) -> wasmtime::Result<Option<WitDatapackInfo>> {
        let state = self;
        let server = state
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        let pack = DatapackManager::get_pack_info(server, &name);
        Ok(pack.map(to_wit_datapack_info))
    }

    fn is_enabled(
        &mut self,
        _res: Resource<WitDatapackManager>,
        name: String,
    ) -> wasmtime::Result<bool> {
        let state = self;
        let server = state
            .server
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
        Ok(DatapackManager::is_pack_enabled(server, &name))
    }
}

impl HostDatapackManagerWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn enable_pack(
        accessor: &Accessor<PluginHostState, Self>,
        _res: Resource<WitDatapackManager>,
        name: String,
        position: WitEnablePosition,
    ) -> wasmtime::Result<Result<(), String>> {
        let server = accessor.with(|mut host| -> wasmtime::Result<_> {
            let state = host.get();
            let server = state
                .server
                .clone()
                .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
            Ok(server)
        })?;
        let position = to_data_enable_position(position);

        run_blocking(accessor, move || {
            DatapackManager::enable_pack(&server, &name, position)
        })
        .await
    }

    async fn disable_pack(
        accessor: &Accessor<PluginHostState, Self>,
        _res: Resource<WitDatapackManager>,
        name: String,
    ) -> wasmtime::Result<Result<(), String>> {
        let server = accessor.with(|mut host| -> wasmtime::Result<_> {
            let state = host.get();
            let server = state
                .server
                .clone()
                .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
            Ok(server)
        })?;

        run_blocking(accessor, move || {
            DatapackManager::disable_pack(&server, &name)
        })
        .await
    }

    async fn reload(
        accessor: &Accessor<PluginHostState, Self>,
        _res: Resource<WitDatapackManager>,
    ) -> wasmtime::Result<Result<(), String>> {
        let server = accessor.with(|mut host| -> wasmtime::Result<_> {
            let state = host.get();
            let server = state
                .server
                .clone()
                .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
            Ok(server)
        })?;

        run_blocking(accessor, move || DatapackManager::reload(&server)).await
    }

    async fn execute_function(
        accessor: &Accessor<PluginHostState, Self>,
        _res: Resource<WitDatapackManager>,
        name: String,
    ) -> wasmtime::Result<Result<u32, String>> {
        let server = accessor.with(|mut host| -> wasmtime::Result<_> {
            let state = host.get();
            let server = state
                .server
                .clone()
                .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
            Ok(server)
        })?;

        run_blocking(accessor, move || {
            DatapackManager::execute_function_from_console(&server, &name).map(|count| count as u32)
        })
        .await
    }
}

fn to_wit_datapack_info(info: pumpkin_core::data::datapack::DatapackInfo) -> WitDatapackInfo {
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
) -> pumpkin_core::data::datapack::DatapackEnablePosition {
    match pos {
        WitEnablePosition::First => pumpkin_core::data::datapack::DatapackEnablePosition::First,
        WitEnablePosition::Last => pumpkin_core::data::datapack::DatapackEnablePosition::Last,
        WitEnablePosition::Before(s) => {
            pumpkin_core::data::datapack::DatapackEnablePosition::Before(s)
        }
        WitEnablePosition::After(s) => {
            pumpkin_core::data::datapack::DatapackEnablePosition::After(s)
        }
    }
}
