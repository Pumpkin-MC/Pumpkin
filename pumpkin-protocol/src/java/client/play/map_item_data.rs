use crate::{ClientPacket, VarInt, WritingError, ser::NetworkWriteExt};
use pumpkin_data::packet::clientbound::PLAY_MAP_ITEM_DATA;
use pumpkin_macros::java_packet;
use std::io::Write;

#[java_packet(PLAY_MAP_ITEM_DATA)]
pub struct CMapItemData<'a> {
    pub map_id: VarInt,
    pub scale: i8,
    pub locked: bool,
    pub icons: Option<&'a [MapIcon]>,
    pub data: Option<MapPatch<'a>>,
}

pub struct MapIcon {
    pub icon_type: VarInt,
    pub x: i8,
    pub z: i8,
    pub direction: i8,
    pub display_name: Option<String>,
}

pub struct MapPatch<'a> {
    pub columns: u8,
    pub rows: u8,
    pub x: i8,
    pub z: i8,
    pub data: &'a [u8],
}

impl ClientPacket for CMapItemData<'_> {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        _version: &pumpkin_util::version::MinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_var_int(&self.map_id)?;
        write.write_i8(self.scale)?;
        write.write_bool(self.locked)?;

        if let Some(icons) = self.icons {
            write.write_bool(true)?;
            write.write_var_int(&VarInt(icons.len() as i32))?;
            for icon in icons {
                write.write_var_int(&icon.icon_type)?;
                write.write_i8(icon.x)?;
                write.write_i8(icon.z)?;
                write.write_i8(icon.direction)?;
                if let Some(name) = &icon.display_name {
                    write.write_bool(true)?;
                    write.write_string(&format!("{{\"text\":\"{name}\"}}"))?;
                } else {
                    write.write_bool(false)?;
                }
            }
        } else {
            write.write_bool(false)?;
        }

        if let Some(patch) = &self.data {
            write.write_u8(patch.columns)?;
            if patch.columns > 0 {
                write.write_u8(patch.rows)?;
                write.write_i8(patch.x)?;
                write.write_i8(patch.z)?;
                write.write_var_int(&VarInt(patch.data.len() as i32))?;
                write.write_all(patch.data).map_err(WritingError::IoError)?;
            }
        } else {
            write.write_u8(0)?;
        }

        Ok(())
    }
}
