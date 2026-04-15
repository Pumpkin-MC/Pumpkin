use pumpkin_data::packet::clientbound::PLAY_TAB_LIST;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::java_packet;
use serde::Serialize;

/// Updates the header and footer of the player list (tab menu).
///
/// This packet is used to display custom text above and below the player list
/// when a player presses the TAB key.
#[derive(Serialize)]
#[java_packet(PLAY_TAB_LIST)]
pub struct CTabList<'a> {
    /// The header text displayed at the top of the player list.
    pub header: &'a TextComponent,
    /// The footer text displayed at the bottom of the player list.
    pub footer: &'a TextComponent,
}

impl<'a> CTabList<'a> {
    #[must_use]
    pub const fn new(header: &'a TextComponent, footer: &'a TextComponent) -> Self {
        Self { header, footer }
    }
}
