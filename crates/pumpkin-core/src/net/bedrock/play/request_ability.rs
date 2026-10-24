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
                    {
                        let mut abilities = player
                            .abilities
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        if abilities.allow_flying {
                            abilities.flying = requested_flying;
                        } else {
                            abilities.flying = false;
                        }
                    }
                    player.send_abilities_update();
                }
            }
            _ => {
                debug!("Received RequestAbility packet for unhandled ability {ability_id}");
            }
        }
    }
}
