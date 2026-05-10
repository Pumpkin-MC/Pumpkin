use pumpkin_data::packet::clientbound::PLAY_TAB_LIST;
use pumpkin_macros::java_packet;
use pumpkin_util::text::TextComponent;
use serde::Serialize;

/// Updates the header and footer of the player list (Tab List).
#[derive(Serialize)]
#[java_packet(PLAY_TAB_LIST)]
pub struct CTabList<'a> {
    /// The text to be displayed at the top of the player list.
    pub header: &'a TextComponent,
    /// The text to be displayed at the bottom of the player list.
    pub footer: &'a TextComponent,
}

impl<'a> CTabList<'a> {
    #[must_use]
    pub const fn new(header: &'a TextComponent, footer: &'a TextComponent) -> Self {
        Self { header, footer }
    }
}
