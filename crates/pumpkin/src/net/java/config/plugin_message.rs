#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_plugin_message(&self, plugin_message: SPluginMessage<'_>) {
        debug!("Handling plugin message");
        if plugin_message.channel.starts_with(BRAND_CHANNEL_PREFIX) {
            debug!("Got a client brand");
            match str::from_utf8(plugin_message.data) {
                Ok(brand) => self.brand.store(Arc::new(Some(brand.to_string()))),
                Err(e) => self.kick(TextComponent::text(e.to_string())).await,
            }
        }
    }
}
