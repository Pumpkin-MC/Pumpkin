use pumpkin_data::packet::serverbound::PLAY_SEEN_ADVANCEMENTS;
use pumpkin_macros::java_packet;
use pumpkin_util::identifier::Identifier;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[java_packet(PLAY_SEEN_ADVANCEMENTS)]
pub enum SSeenAdvancement {
    OpenTab(Identifier),
    CloseTab,
}
