use crate::KnownPack;
use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_data::packet::serverbound::CONFIG_SELECT_KNOWN_PACKS;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONFIG_SELECT_KNOWN_PACKS)]
pub struct SKnownPacks<'a> {
    pub known_packs: Vec<KnownPack<'a>>,
}

impl<'a> ServerPacket<'a> for SKnownPacks<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let count = bytebuf.get_var_int()?.0;
        if !(0..=64).contains(&count) {
            return Err(ReadingError::Message(
                "Known packs count out of bounds".into(),
            ));
        }
        let mut known_packs = Vec::with_capacity(count as usize);
        for _ in 0..count {
            known_packs.push(KnownPack {
                namespace: bytebuf.get_str_borrowed()?,
                id: bytebuf.get_str_borrowed()?,
                version: bytebuf.get_str_borrowed()?,
            });
        }
        Ok(Self { known_packs })
    }
}
