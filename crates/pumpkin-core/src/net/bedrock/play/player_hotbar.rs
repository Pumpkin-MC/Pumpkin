#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_protocol::bedrock::server::player_hotbar::SPlayerHotbar;

impl BedrockClient {
    pub fn handle_player_hotbar(
        &self,
        server: &Arc<Server>,
        player: &Arc<Player>,
        packet: &SPlayerHotbar,
    ) {
        let slot = packet.selected_slot.0;
        if slot >= 9 {
            return;
        }
        self.select_hotbar_slot(server, player, slot as u8, packet.container_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_protocol::serial::PacketRead;
    use std::io::Cursor;

    #[test]
    fn player_hotbar_decodes_selected_slot() {
        let bytes = [3u8, 0, 1];
        let mut cursor = Cursor::new(&bytes[..]);
        let packet = SPlayerHotbar::read(&mut cursor).expect("failed to decode SPlayerHotbar");
        assert_eq!(packet.selected_slot.0, 3);
        assert_eq!(packet.container_id, 0);
        assert!(packet.should_select_slot);
    }
}
