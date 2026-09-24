use pumpkin::plugin::Plugin;
use pumpkin_api_macros::{plugin_impl, plugin_method};

#[plugin_method]
async fn on_ipc_message(&self, _sender: &str, _message: &[u8]) -> Result<Vec<u8>, String> {
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
    Ok(Vec::new())
}

#[plugin_impl]
struct TestPlugin;

impl TestPlugin {
    const fn new() -> Self {
        Self
    }
}

#[test]
fn plugin_method_runs_io_on_the_global_runtime() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread().build()?;
    let plugin = TestPlugin;

    let result = runtime.block_on(plugin.on_ipc_message("test", &[]));

    assert!(result.is_ok(), "plugin method failed: {result:?}");
    Ok(())
}
