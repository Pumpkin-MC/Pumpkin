use std::io::Write;

use crate::ClientPacket;
use crate::codec::var_int::VarInt;
use crate::ser::{NetworkWriteExt, WritingError};
use pumpkin_data::{packet::clientbound::PLAY_STOP_SOUND, sound::SoundCategory};
use pumpkin_macros::packet;
use pumpkin_util::resource_location::ResourceLocation;

#[packet(PLAY_STOP_SOUND)]
pub struct CStopSound {
    sound_id: Option<ResourceLocation>,
    category: Option<SoundCategory>,
}

impl CStopSound {
    pub fn new(sound_id: Option<ResourceLocation>, category: Option<SoundCategory>) -> Self {
        Self { sound_id, category }
    }
}

impl ClientPacket for CStopSound {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;

        const NO_CATEGORY_NO_SOUND: u8 = 0;
        const CATEGORY_ONLY: u8 = 1;
        const SOUND_ONLY: u8 = 2;
        const CATEGORY_AND_SOUND: u8 = 3;

        match (self.category, &self.sound_id) {
            (Some(category), Some(sound_id)) => {
                write.write_u8_be(CATEGORY_AND_SOUND)?;
                write.write_var_int(&VarInt(category as i32))?;
                write.write_resource_location(sound_id)
            }
            (Some(category), None) => {
                write.write_u8_be(CATEGORY_ONLY)?;
                write.write_var_int(&VarInt(category as i32))
            }
            (None, Some(sound_id)) => {
                write.write_u8_be(SOUND_ONLY)?;
                write.write_resource_location(sound_id)
            }
            (None, None) => write.write_u8_be(NO_CATEGORY_NO_SOUND),
        }
    }
}
