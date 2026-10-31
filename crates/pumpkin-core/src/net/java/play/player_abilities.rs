#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_player_abilities(
        &self,
        player: &Arc<Player>,
        player_abilities: &SPlayerAbilities,
        server: &Arc<Server>,
    ) {
        player.apply_flight_input(server, player_abilities.is_flying());
    }
}
