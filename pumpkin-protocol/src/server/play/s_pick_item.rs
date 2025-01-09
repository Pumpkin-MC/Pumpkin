use pumpkin_macros::server_packet;
use pumpkin_util::math::position::WorldPosition;
use serde::Deserialize;

#[derive(Deserialize)]
#[server_packet("play:pick_item_from_block")]
pub struct SPickItemFromBlock {
    pub pos: WorldPosition,
    pub include_data: bool,
}

#[derive(Deserialize)]
#[server_packet("play:pick_item_from_entity")]
pub struct SPickItemFromEntity {
    pub id: i32,
    pub include_data: bool,
}
