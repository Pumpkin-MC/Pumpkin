#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub async fn handle_modal_form_response(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: pumpkin_protocol::bedrock::server::modal_form_response::SModalFormResponse<'_>,
    ) {
        let mut event =
            crate::plugin::api::events::player::bedrock_form_response::BedrockFormResponseEvent::new(
                player.clone(),
                packet.form_id.0,
                packet
                    .json_response
                    .filter(|data| data != "null")
                    .map(std::borrow::Cow::into_owned),
            );
        server.plugin_manager.fire(server, &mut event).await;
    }
}
