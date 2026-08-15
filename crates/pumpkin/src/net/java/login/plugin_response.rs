#[allow(clippy::wildcard_imports)]
use super::*;

impl PendingConnection {
    pub async fn handle_plugin_response(
        &mut self,
        server: &Server,
        plugin_response: SLoginPluginResponse,
    ) {
        debug!("Handling plugin");
        let velocity_config = &server.advanced_config.networking.proxy.velocity;
        if velocity_config.enabled {
            match velocity::receive_velocity_plugin_response(
                self.address.port(),
                velocity_config,
                plugin_response,
            ) {
                Ok((profile, new_address)) => {
                    self.finish_login(&profile).await;
                    self.gameprofile = Some(profile);
                    self.address = new_address;
                }
                Err(error) => self.kick(TextComponent::text(error.to_string())).await,
            }
        }
    }
}
