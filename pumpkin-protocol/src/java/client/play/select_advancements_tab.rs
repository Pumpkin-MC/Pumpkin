use pumpkin_data::packet::clientbound::PLAY_SELECT_ADVANCEMENTS_TAB;
use pumpkin_macros::java_packet;
use pumpkin_util::identifier::Identifier;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(PLAY_SELECT_ADVANCEMENTS_TAB)]
pub struct CSelectAdvancementsTab {
    pub tab_id: Option<Identifier>,
}

impl CSelectAdvancementsTab {
    #[must_use]
    pub const fn new(tab_id: Option<Identifier>) -> Self {
        Self { tab_id }
    }
}
