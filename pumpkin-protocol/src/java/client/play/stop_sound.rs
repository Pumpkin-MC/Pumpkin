use crate::codec::var_int::VarInt;
use crate::ser::NetworkWriteExt;
use crate::{ClientPacket, WritingError};
use pumpkin_data::{packet::clientbound::PLAY_STOP_SOUND, sound::SoundCategory};
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::version::MinecraftVersion;
use std::io::Write;

#[java_packet(PLAY_STOP_SOUND)]
pub struct CStopSound {
    pub sound_id: Option<ResourceLocation>,
    pub category: Option<SoundCategory>,
}

impl CStopSound {
    #[must_use]
    pub const fn new(sound_id: Option<ResourceLocation>, category: Option<SoundCategory>) -> Self {
        Self { sound_id, category }
    }
}

impl ClientPacket for CStopSound {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        const NO_CATEGORY_NO_SOUND: u8 = 0;
        const CATEGORY_ONLY: u8 = 1;
        const SOUND_ONLY: u8 = 2;
        const CATEGORY_AND_SOUND: u8 = 3;
        let mut write = write;

        match (self.category, &self.sound_id) {
            (Some(category), Some(sound_id)) => {
                write.write_u8(CATEGORY_AND_SOUND)?;
                write.write_var_int(&VarInt(category as i32))?;
                write.write_string(sound_id)
            }
            (Some(category), None) => {
                write.write_u8(CATEGORY_ONLY)?;
                write.write_var_int(&VarInt(category as i32))
            }
            (None, Some(sound_id)) => {
                write.write_u8(SOUND_ONLY)?;
                write.write_string(sound_id)
            }
            (None, None) => write.write_u8(NO_CATEGORY_NO_SOUND),
        }
    }
}
