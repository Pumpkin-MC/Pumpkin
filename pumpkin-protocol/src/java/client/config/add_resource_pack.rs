use pumpkin_util::text::TextComponent;

use pumpkin_macros::packet;
use serde::Serialize;

use pumpkin_data::packet::clientbound::CONFIG_RESOURCE_PACK_PUSH;

#[derive(Serialize)]
#[packet(CONFIG_RESOURCE_PACK_PUSH)]
pub struct CConfigAddResourcePack<'a> {
    #[serde(with = "uuid::serde::compact")]
    pub uuid: &'a uuid::Uuid,
    pub url: &'a str,
    pub hash: &'a str, // max 40
    pub forced: bool,
    pub prompt_message: Option<TextComponent>,
}

impl<'a> CConfigAddResourcePack<'a> {
    #[must_use]
    pub const fn new(
        uuid: &'a uuid::Uuid,
        url: &'a str,
        hash: &'a str,
        forced: bool,
        prompt_message: Option<TextComponent>,
    ) -> Self {
        Self {
            uuid,
            url,
            hash,
            forced,
            prompt_message,
        }
    }
}
