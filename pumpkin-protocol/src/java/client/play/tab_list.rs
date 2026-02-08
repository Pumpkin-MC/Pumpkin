use pumpkin_data::packet::clientbound::PLAY_TAB_LIST;
use pumpkin_macros::java_packet;
use pumpkin_util::text::TextComponent;
use serde::Serialize;

/// Sets the header and footer text displayed in the player list (tab menu).
#[derive(Serialize)]
#[java_packet(PLAY_TAB_LIST)]
pub struct CTabList<'a> {
    pub header: &'a TextComponent,
    pub footer: &'a TextComponent,
}

impl<'a> CTabList<'a> {
    #[must_use]
    pub const fn new(header: &'a TextComponent, footer: &'a TextComponent) -> Self {
        Self { header, footer }
    }
}
