use std::{fs, path::Path, sync::Arc};
use thiserror::Error;
use tokio::sync::Mutex;
use wasmtime::{Cache, CacheConfig, Engine, Store, component::Component};

use crate::plugin::{Context, PluginMetadata, loader::wasm::wasm_host::state::PluginHostState};

pub mod args;
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
    cache_dir: std::path::PathBuf,
    linker_v0_1_0: wasmtime::component::Linker<PluginHostState>,
}

pub enum PluginInstance {
    V0_1_0(wit::v0_1_0::Plugin),
}

pub struct WasmPlugin {
    pub plugin_instance: PluginInstance,
    pub store: Mutex<Store<PluginHostState>>,
}

impl PluginRuntime {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, PluginInitError> {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(true);
        let mut path = std::path::absolute(path.as_ref()).expect("Failed to get absolute path");
        path.pop();
        path.push("cache");
        let mut cache_config = CacheConfig::new();
        cache_config.with_directory(&path);
        config.cache(Some(
            Cache::new(cache_config).expect("Failed to create cache"),
        ));
        let engine = Engine::new(&config).map_err(PluginInitError::EngineCreationFailed)?;

        let linker_v0_1_0 =
            wit::v0_1_0::setup_linker(&engine).map_err(PluginInitError::LinkerSetupFailed)?;

        Ok(Self {
            engine,
            cache_dir: path,
            linker_v0_1_0,
        })
    }

    pub async fn init_plugin<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(Arc<WasmPlugin>, PluginMetadata), PluginInitError> {
        let wasm_bytes = std::fs::read(&path)?;

        let api_version = probe_api_version_from_bytes(&wasm_bytes)?;

        if api_version != "0.1.0" {
            return Err(PluginInitError::ApiVersionMismatch(api_version));
        }

        let component = load_component(&self.engine, &wasm_bytes, path.as_ref(), &self.cache_dir)?;

        let (wasm_plugin, metadata) = match api_version.as_str() {
            "0.1.0" => {
                wit::v0_1_0::init_plugin(&self.engine, &self.linker_v0_1_0, component).await?
            }
            _ => return Err(PluginInitError::ApiVersionMismatch(api_version)),
        };
        let wasm_plugin = Arc::new(wasm_plugin);
        wasm_plugin.store.lock().await.data_mut().plugin = Some(Arc::downgrade(&wasm_plugin));

        Ok((wasm_plugin, metadata))
    }
}

fn cache_key(wasm_path: &Path) -> Result<String, std::io::Error> {
    let metadata = fs::metadata(wasm_path)?;
    let file_name = wasm_path.file_stem().unwrap().to_string_lossy();
    let len = metadata.len();
    let modified = metadata
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Ok(format!(
        "{file_name}-{len}-{modified}-{}.cwasm",
        env!("CARGO_PKG_VERSION"),
    ))
}

fn load_component(
    engine: &Engine,
    wasm_bytes: &[u8],
    wasm_path: &Path,
    cache_dir: &Path,
) -> Result<Component, PluginInitError> {
    let cache_name = cache_key(wasm_path)?;
    let cache_path = cache_dir.join(cache_name);

    if cache_path.exists() {
        match unsafe { Component::deserialize_file(engine, &cache_path) } {
            Ok(component) => return Ok(component),
            Err(_) => {
                let _ = fs::remove_file(&cache_path);
            }
        }
    }

    let component = Component::new(engine, wasm_bytes)?;
    fs::write(&cache_path, component.serialize()?)?;
    Ok(component)
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
        &self,
        context: Arc<Context>,
    ) -> Result<Result<(), String>, wasmtime::Error> {
        let mut store = self.store.lock().await;

        store.data_mut().server = Some(context.server.clone());

        match self.plugin_instance {
            PluginInstance::V0_1_0(ref plugin) => {
                let context = store.data_mut().add_context(context)?;
                plugin.call_on_load(&mut *store, context).await
            }
        }
    }

    pub async fn on_unload(
        &self,
        context: Arc<Context>,
    ) -> Result<Result<(), String>, wasmtime::Error> {
        let mut store = self.store.lock().await;

        match self.plugin_instance {
            PluginInstance::V0_1_0(ref plugin) => {
                let context = store.data_mut().add_context(context)?;
                plugin.call_on_unload(&mut *store, context).await
            }
        }
    }
}

pub trait DowncastResourceExt<E> {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a E;
    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut E;
    fn consume(self, state: &mut PluginHostState) -> E;
}
