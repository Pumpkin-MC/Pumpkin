use pumpkin_data::packet::serverbound::STATUS_STATUS_REQUEST;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(STATUS_STATUS_REQUEST)]
pub struct SStatusRequest;
