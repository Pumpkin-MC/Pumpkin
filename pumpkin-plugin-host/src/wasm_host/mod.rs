use std::{path::Path, sync::Arc};

use thiserror::Error;
use tokio::sync::Mutex;
use wasmtime::{Engine, Store};

use crate::{
    metadata::PluginMetadata,
    wasm_host::state::{ContextProvider, PluginHostState},
};

pub mod logging;
pub mod state;
pub mod wit;

#[derive(Error, Debug)]
pub enum PluginInitError {
    #[error("Engine creation failed")]
    EngineCreationFailed(wasmtime::Error),
    #[error("Failed to setup linker")]
    LinkerSetupFailed(wasmtime::Error),
    #[error("plugin API version mismatch received plugin with version `{0}`")]
    ApiVersionMismatch(String),
    #[error("plugin missing pumpkin:api-version custom section")]
    MissingApiVersionSection,
    #[error("failed to read payload for plugin")]
    FailedToReadPayload(#[from] wasmparser::BinaryReaderError),
    #[error("failed to read plugin bytes")]
    FailedToReadPluginBytes(#[from] std::io::Error),
    #[error("plugin failed to load")]
    PluginFailedToLoad(#[from] wasmtime::Error),
}

pub struct PluginRuntime {
    engine: Engine,
    linker_v0_1_0: wasmtime::component::Linker<PluginHostState>,
}

pub enum PluginInstance {
    V0_1_0(wit::v0_1_0::Plugin),
}

pub struct WasmPlugin {
    pub plugin: PluginInstance,
    pub store: Mutex<Store<PluginHostState>>,
}

impl PluginRuntime {
    pub fn new() -> Result<Self, PluginInitError> {
        let mut config = wasmtime::Config::new();
        config.async_support(true);
        config.wasm_component_model(true);
        let engine = Engine::new(&config).map_err(PluginInitError::EngineCreationFailed)?;

        let linker_v0_1_0 =
            wit::v0_1_0::setup_linker(&engine).map_err(PluginInitError::LinkerSetupFailed)?;

        Ok(Self {
            engine,
            linker_v0_1_0,
        })
    }

    pub async fn init_plugin<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(WasmPlugin, PluginMetadata), PluginInitError> {
        let wasm_bytes = std::fs::read(path)?;

        let api_version = probe_api_version_from_bytes(&wasm_bytes)?;

        if api_version != "0.1.0" {
            return Err(PluginInitError::ApiVersionMismatch(api_version));
        }

        let component = wasmtime::component::Component::new(&self.engine, &wasm_bytes)?;

        let plugin = match api_version.as_str() {
            "0.1.0" => {
                wit::v0_1_0::init_plugin(&self.engine, &self.linker_v0_1_0, component).await?
            }
            _ => return Err(PluginInitError::ApiVersionMismatch(api_version)),
        };

        Ok(plugin)
    }
}

/// Kind of a dumb solution, but in order to get the API version from a component, we define a custom section inside of the wasm binary itself, we then
/// parse the value in that section to get the API version.
fn probe_api_version_from_bytes(wasm_bytes: &[u8]) -> Result<String, PluginInitError> {
    let parser = wasmparser::Parser::new(0);
    for payload in parser.parse_all(wasm_bytes) {
        if let wasmparser::Payload::CustomSection(reader) = payload?
            && reader.name() == "pumpkin:api-version"
        {
            return Ok(String::from_utf8_lossy(reader.data()).to_string());
        }
    }
    Err(PluginInitError::MissingApiVersionSection)
}

impl WasmPlugin {
    pub async fn on_load(
        &mut self,
        context_provider: Arc<dyn ContextProvider + Send + Sync>,
    ) -> Result<Result<(), String>, wasmtime::Error> {
        let mut store = self.store.lock().await;

        match self.plugin {
            PluginInstance::V0_1_0(ref plugin) => {
                let context = store.data_mut().add_context(context_provider)?;
                plugin.call_on_load(&mut *store, context).await
            }
        }
    }

    pub async fn on_unload(
        &mut self,
        context_provider: Arc<dyn ContextProvider + Send + Sync>,
    ) -> Result<Result<(), String>, wasmtime::Error> {
        let mut store = self.store.lock().await;

        match self.plugin {
            PluginInstance::V0_1_0(ref plugin) => {
                let context = store.data_mut().add_context(context_provider)?;
                plugin.call_on_unload(&mut *store, context).await
            }
        }
    }
}
