use std::io::Write;

use pumpkin_data::packet::clientbound::CONFIG_SELECT_KNOWN_PACKS;
use pumpkin_macros::packet;

use crate::{
    ClientPacket, KnownPack,
    ser::{NetworkWriteExt, WritingError},
};

#[packet(CONFIG_SELECT_KNOWN_PACKS)]
pub struct CKnownPacks<'a> {
    pub known_packs: &'a [KnownPack<'a>],
}

impl<'a> CKnownPacks<'a> {
    pub fn new(known_packs: &'a [KnownPack]) -> Self {
        Self { known_packs }
    }
}

impl ClientPacket for CKnownPacks<'_> {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;
        write.write_list::<KnownPack>(self.known_packs, |p, v| {
            p.write_string(v.namespace)?;
            p.write_string(v.id)?;
            p.write_string(v.version)
        })
    }
}
