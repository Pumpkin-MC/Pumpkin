use pumpkin_util::text::TextComponent;

use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("config:resource_pack_push")]
pub struct CConfigAddResourcePack<'a> {
    uuid: uuid::Uuid,
    url: &'a str,
    hash: &'a str, // max 40
    forced: bool,
    prompt_message: Option<TextComponent>,
}

impl<'a> CConfigAddResourcePack<'a> {
    pub fn new(
        uuid: uuid::Uuid,
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
