#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub fn handle_request_ability(
        &self,
        player: &Arc<Player>,
        packet: &pumpkin_protocol::bedrock::server::request_ability::SRequestAbility,
    ) {
        player.update_last_action_time();
        let ability_id = packet.ability.0;
        match ability_id {
            9 => {
                // Flying
                if let pumpkin_protocol::bedrock::server::request_ability::AbilityValue::Bool(
                    requested_flying,
                ) = packet.value
                {
                    let allow_flying = player
                        .abilities
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .allow_flying;
                    player.set_flying(allow_flying && requested_flying);
                }
            }
            _ => {
                debug!("Received RequestAbility packet for unhandled ability {ability_id}");
            }
        }
    }
}
