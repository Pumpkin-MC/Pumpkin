#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_change_game_mode(
        &self,
        player: &Arc<Player>,
        change_game_mode: SChangeGameMode,
    ) {
        if player.permission_lvl.load() >= PermissionLvl::Two {
            player.set_gamemode(change_game_mode.game_mode).await;
            let gamemode_string = format!("gameMode.{}", change_game_mode.game_mode.name());
            player
                .send_system_message(&TextComponent::translate_cross(
                    translation::java::COMMANDS_GAMEMODE_SUCCESS_SELF,
                    translation::bedrock::COMMANDS_GAMEMODE_SUCCESS_SELF,
                    [TextComponent::translate_cross(
                        gamemode_string.clone(),
                        gamemode_string,
                        [],
                    )],
                ))
                .await;
        }
    }
}
