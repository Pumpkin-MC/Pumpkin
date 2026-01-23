use pumpkin_data::packet::clientbound::CONFIG_SELECT_KNOWN_PACKS;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::KnownPack;

#[derive(Serialize)]
#[packet(CONFIG_SELECT_KNOWN_PACKS)]
pub struct CKnownPacks<'a> {
    pub known_packs: &'a [KnownPack<'a>],
}

impl<'a> CKnownPacks<'a> {
    #[must_use]
    pub const fn new(known_packs: &'a [KnownPack]) -> Self {
        Self { known_packs }
    }
}
