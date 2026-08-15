#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_player_abilities(
        &self,
        player: &Arc<Player>,
        player_abilities: SPlayerAbilities,
        server: &Arc<Server>,
    ) {
        let (flying, allow_flying) = {
            let abilities = player.abilities.lock().await;
            (abilities.flying, abilities.allow_flying)
        };

        // Set the flying ability
        let new_flying = player_abilities.flags & 0x02 != 0 && allow_flying;
        if flying != new_flying {
            send_cancellable! {{
                server;
                PlayerToggleFlightEvent::new(player.clone(), new_flying);
                'after: {
                    if event.is_flying {
                        player.living_entity.fall_distance.store(0.0);
                    }
                    player.abilities.lock().await.flying = event.is_flying;
                }
                'cancelled: {
                    player.send_abilities_update().await;
                }
            }}
        }
    }
}
