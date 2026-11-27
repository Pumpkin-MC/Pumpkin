#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub fn handle_sign_update(&self, player: &Player, sign_data: &SUpdateSign<'_>) {
        // `ServerGamePacketListenerImpl#updateSignText` resets the action time before validating.
        player.update_last_action_time();

        player.update_sign_text(
            sign_data.location,
            sign_data.is_front_text,
            [
                sign_data.line_1,
                sign_data.line_2,
                sign_data.line_3,
                sign_data.line_4,
            ],
        );
    }
}
