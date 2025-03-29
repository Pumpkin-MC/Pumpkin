use std::borrow::Cow;

use pumpkin_data::packet::serverbound::PLAY_CHAT_COMMAND;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[packet(PLAY_CHAT_COMMAND)]
pub struct SChatCommand<'a> {
    pub command: Cow<'a, str>,
}
