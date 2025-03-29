use pumpkin::plugin::Context;
use pumpkin_api_macros::{plugin_impl, plugin_method};

#[plugin_impl]
pub struct MyPlugin {}

impl MyPlugin {
    pub fn new() -> Self {
        MyPlugin {}
    }
}

impl Default for MyPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[plugin_method]
async fn on_load(&mut self, server: &Context) -> Result<(), String> {
    Ok(())
}
