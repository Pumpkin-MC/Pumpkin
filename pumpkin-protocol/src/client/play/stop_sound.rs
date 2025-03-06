use crate::bytebuf::ByteBufMut;
use crate::codec::var_int::VarInt;
use crate::{ClientPacket, codec::identifier::Identifier};
use pumpkin_data::{packet::clientbound::PLAY_STOP_SOUND, sound::SoundCategory};
use pumpkin_macros::packet;

#[packet(PLAY_STOP_SOUND)]
pub struct CStopSound {
    sound_id: Option<Identifier>,
    category: Option<SoundCategory>,
}

impl CStopSound {
    pub fn new(sound_id: Option<Identifier>, category: Option<SoundCategory>) -> Self {
        Self { sound_id, category }
    }
}

impl ClientPacket for CStopSound {
    fn write(&self, bytebuf: &mut impl bytes::BufMut) {
        const NO_CATEGORY_NO_SOUND: u8 = 0;
        const CATEGORY_ONLY: u8 = 1;
        const SOUND_ONLY: u8 = 2;
        const CATEGORY_AND_SOUND: u8 = 3;

        match (self.category, &self.sound_id) {
            (Some(category), Some(sound_id)) => {
                bytebuf.put_u8(CATEGORY_AND_SOUND);
                bytebuf.put_var_int(&VarInt(category as i32));
                bytebuf.put_identifier(sound_id);
            }
            (Some(category), None) => {
                bytebuf.put_u8(CATEGORY_ONLY);
                bytebuf.put_var_int(&VarInt(category as i32));
            }
            (None, Some(sound_id)) => {
                bytebuf.put_u8(SOUND_ONLY);
                bytebuf.put_identifier(sound_id);
            }
            (None, None) => {
                bytebuf.put_u8(NO_CATEGORY_NO_SOUND);
            }
        }
    }
}
