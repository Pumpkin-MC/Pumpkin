use pumpkin_data::packet::clientbound::PLAY_SERVER_DATA;
use pumpkin_macros::java_packet;
use pumpkin_util::text::TextComponent;
use serde::Serialize;

/// Sends the server's MOTD and icon to the client during play.
#[derive(Serialize)]
#[java_packet(PLAY_SERVER_DATA)]
pub struct CServerData<'a> {
    pub motd: &'a TextComponent,
    pub icon: &'a Option<Vec<u8>>,
    pub enforces_secure_chat: bool,
}

impl<'a> CServerData<'a> {
    #[must_use]
    pub const fn new(
        motd: &'a TextComponent,
        icon: &'a Option<Vec<u8>>,
        enforces_secure_chat: bool,
    ) -> Self {
        Self {
            motd,
            icon,
            enforces_secure_chat,
        }
    }
}
