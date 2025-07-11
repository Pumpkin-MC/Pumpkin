use pumpkin_data::packet::clientbound::PLAY_TAB_LIST;
use pumpkin_macros::packet;
use pumpkin_util::text::TextComponent;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[packet(PLAY_TAB_LIST)]
pub struct CTabList {
    pub header: TextComponent,
    pub footer: TextComponent,
}

impl CTabList {
    #[must_use]
    pub fn new(header: TextComponent, footer: TextComponent) -> Self {
        Self { header, footer }
    }

    #[must_use]
    pub fn with_header(self, header: TextComponent) -> Self {
        Self {
            header,
            footer: self.footer,
        }
    }

    #[must_use]
    pub fn with_footer(self, footer: TextComponent) -> Self {
        Self {
            header: self.header,
            footer,
        }
    }
}

impl Default for CTabList {
    fn default() -> Self {
        Self {
            header: TextComponent::text(""),
            footer: TextComponent::text(""),
        }
    }
}
