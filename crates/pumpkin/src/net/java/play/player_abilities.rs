#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_player_abilities(
        &self,
        player: &Arc<Player>,
        player_abilities: &SPlayerAbilities,
        server: &Arc<Server>,
    ) {
        let (flying, allow_flying) = {
            let abilities = player
                .abilities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            (abilities.flying, abilities.allow_flying)
        };

        // Set the flying ability
        let new_flying = player_abilities.is_flying() && allow_flying;
        if flying != new_flying {
            send_cancellable_blocking! {{
                server;
                PlayerToggleFlightEvent::new(player.clone(), new_flying);
                'after: {
                    if event.is_flying {
                        player.living_entity.fall_distance.store(0.0);
                    }
                    player.abilities.lock().unwrap_or_else(std::sync::PoisonError::into_inner).flying = event.is_flying;
                }
                'cancelled: {
                    player.send_abilities_update();
                }
            }}
        }
    }
}
