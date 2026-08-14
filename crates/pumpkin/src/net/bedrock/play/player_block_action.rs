#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    pub async fn handle_player_block_action(
        &self,
        player: &Arc<Player>,
        server: &Server,
        packet: pumpkin_protocol::bedrock::server::player_auth_input::PlayerBlockAction,
    ) {
        use pumpkin_protocol::bedrock::server::player_action::Action as PlayerAction;
        let Ok(action) = PlayerAction::try_from(packet.action.0) else {
            return;
        };
        self.handle_player_action(
            player,
            server,
            SPlayerAction {
                runtime_id: VarULong(0), // Unused
                action,
                block_pos: packet.block_pos,
                result_pos: BlockPos::ZERO,
                face: packet.face,
            },
        )
        .await;
    }
}
