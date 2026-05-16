use pumpkin_data::packet::serverbound::CONFIG_CUSTOM_CLICK_ACTION;
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;
use serde::Deserialize;

#[derive(Deserialize)]
#[java_packet(CONFIG_CUSTOM_CLICK_ACTION)]
pub struct SCustomClickAction {
    pub action_id: ResourceLocation,
    pub payload: Option<Box<[u8]>>,
}
