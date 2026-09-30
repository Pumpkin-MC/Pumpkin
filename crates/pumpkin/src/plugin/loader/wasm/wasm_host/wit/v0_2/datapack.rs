use super::run_blocking;
use crate::data::datapack::DatapackManager;
use crate::plugin::loader::wasm::wasm_host::state::PluginHostState;
use crate::plugin::loader::wasm::wasm_host::wit::v0_2::pumpkin::plugin::datapack::{
    DatapackInfo as WitDatapackInfo, DatapackManager as WitDatapackManager,
    EnablePosition as WitEnablePosition, Host as DatapackHost, HostDatapackManager,
    HostDatapackManagerWithStore,
};
use wasmtime::component::Accessor;
use wasmtime::component::{HasSelf, Resource};

impl DatapackHost for PluginHostState {}

impl HostDatapackManager for PluginHostState {
    async fn drop(&mut self, rep: Resource<WitDatapackManager>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}

impl HostDatapackManagerWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn list_all_packs(
        accessor: &Accessor<PluginHostState, Self>,
        _res: Resource<WitDatapackManager>,
    ) -> wasmtime::Result<Vec<WitDatapackInfo>> {
        accessor.with(|mut host| {
            let state = host.get();
            let server = state
                .server
                .as_ref()
                .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
            let packs = DatapackManager::list_all_packs(server);
            Ok(packs.into_iter().map(to_wit_datapack_info).collect())
        })
    }

    async fn list_enabled_packs(
        accessor: &Accessor<PluginHostState, Self>,
        _res: Resource<WitDatapackManager>,
    ) -> wasmtime::Result<Vec<WitDatapackInfo>> {
        accessor.with(|mut host| {
            let state = host.get();
            let server = state
                .server
                .as_ref()
                .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
            let packs = DatapackManager::list_enabled_packs(server);
            Ok(packs.into_iter().map(to_wit_datapack_info).collect())
        })
    }

    async fn list_available_packs(
        accessor: &Accessor<PluginHostState, Self>,
        _res: Resource<WitDatapackManager>,
    ) -> wasmtime::Result<Vec<WitDatapackInfo>> {
        accessor.with(|mut host| {
            let state = host.get();
            let server = state
                .server
                .as_ref()
                .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
            let packs = DatapackManager::list_available_packs(server);
            Ok(packs.into_iter().map(to_wit_datapack_info).collect())
        })
    }

    async fn get_pack(
        accessor: &Accessor<PluginHostState, Self>,
        _res: Resource<WitDatapackManager>,
        name: String,
    ) -> wasmtime::Result<Option<WitDatapackInfo>> {
        accessor.with(|mut host| {
            let state = host.get();
            let server = state
                .server
                .as_ref()
                .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
            let pack = DatapackManager::get_pack_info(server, &name);
            Ok(pack.map(to_wit_datapack_info))
        })
    }

    async fn is_enabled(
        accessor: &Accessor<PluginHostState, Self>,
        _res: Resource<WitDatapackManager>,
        name: String,
    ) -> wasmtime::Result<bool> {
        accessor.with(|mut host| {
            let state = host.get();
            let server = state
                .server
                .as_ref()
                .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
            Ok(DatapackManager::is_pack_enabled(server, &name))
        })
    }
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

        run_blocking(move || DatapackManager::enable_pack(&server, &name, position)).await
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

        run_blocking(move || DatapackManager::disable_pack(&server, &name)).await
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

        run_blocking(move || DatapackManager::reload(&server)).await
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

        run_blocking(move || {
            DatapackManager::execute_function_from_console(&server, &name).map(|count| count as u32)
        })
        .await
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
