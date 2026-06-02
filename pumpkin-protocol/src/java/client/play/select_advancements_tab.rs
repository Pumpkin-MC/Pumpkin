use crate::ResourceLocation;
use pumpkin_data::packet::clientbound::PLAY_SELECT_ADVANCEMENTS_TAB;
use pumpkin_macros::java_packet;
use serde::Serialize;
use pumpkin_util::identifier::Identifier;

#[derive(Serialize)]
#[java_packet(PLAY_SELECT_ADVANCEMENTS_TAB)]
#[allow(unused)]
pub struct CSelectAdvancementsTab {
    pub tab_id: Option<Identifier>,
}

impl CSelectAdvancementsTab {
    #[allow(unused)]
    pub const fn new(tab_id: Option<Identifier>) -> Self {
        Self { tab_id }
    }
}
