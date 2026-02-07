use crate::ResourceLocation;
use pumpkin_macros::java_packet;
use serde::Serialize;
use pumpkin_data::packet::clientbound::PLAY_SELECT_ADVANCEMENTS_TAB;

#[derive(Serialize)]
#[java_packet(PLAY_SELECT_ADVANCEMENTS_TAB)]
pub struct CSelectAdvancementsTab {
    pub tab_id: Option<ResourceLocation>,
}

impl CSelectAdvancementsTab {
    pub fn new(tab_id: Option<ResourceLocation>) -> Self {
        Self { tab_id }
    }
}
