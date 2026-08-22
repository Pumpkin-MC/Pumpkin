use std::io::Write;

use pumpkin_data::packet::clientbound::play::MAP_ITEM_DATA;
use pumpkin_macros::java_packet;
use pumpkin_util::{text::TextComponent, version::JavaMinecraftVersion};

use crate::{ClientPacket, VarInt, WritingError, ser::NetworkWriteExt};

#[java_packet(MAP_ITEM_DATA)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CMapItemData<'a> {
    pub map_id: VarInt,
    pub scale: i8,
    pub tracking_position: bool,
    pub locked: bool,
    pub icons: Option<&'a [MapIcon]>,
    pub data: Option<MapPatch<'a>>,
}

impl<'a> CMapItemData<'a> {
    #[must_use]
    pub const fn new(
        map_id: VarInt,
        scale: i8,
        tracking_position: bool,
        locked: bool,
        icons: Option<&'a [MapIcon]>,
        data: Option<MapPatch<'a>>,
    ) -> Self {
        Self {
            map_id,
            scale,
            tracking_position,
            locked,
            icons,
            data,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapIcon {
    pub icon_type: VarInt,
    pub x: i8,
    pub z: i8,
    pub direction: i8,
    pub display_name: Option<TextComponent>,
}

impl MapIcon {
    #[must_use]
    pub const fn new(
        icon_type: VarInt,
        x: i8,
        z: i8,
        direction: i8,
        display_name: Option<TextComponent>,
    ) -> Self {
        Self {
            icon_type,
            x,
            z,
            direction,
            display_name,
        }
    }

    pub fn write_with_version(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let v1_13 = *version >= JavaMinecraftVersion::V_1_13;
        if v1_13 {
            write.write_var_int(&self.icon_type)?;
            write.write_i8(self.x)?;
            write.write_i8(self.z)?;
            write.write_i8(self.direction)?;
            if let Some(display_name) = &self.display_name {
                write.write_bool(true)?;
                write.write_component(display_name, version)?;
            } else {
                write.write_bool(false)?;
            }
        } else {
            let type_id = (self.icon_type.0 as u8) & 0x0F;
            let direction = (self.direction as u8) & 0x0F;
            write.write_u8((type_id << 4) | direction)?;
            write.write_i8(self.x)?;
            write.write_i8(self.z)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MapPatch<'a> {
    pub columns: u8,
    pub rows: u8,
    pub x: i8,
    pub z: i8,
    pub data: &'a [u8],
}

impl<'a> MapPatch<'a> {
    #[must_use]
    pub const fn new(columns: u8, rows: u8, x: i8, z: i8, data: &'a [u8]) -> Self {
        Self {
            columns,
            rows,
            x,
            z,
            data,
        }
    }
}

impl ClientPacket for CMapItemData<'_> {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_var_int(&self.map_id)?;
        write.write_i8(self.scale)?;

        if *version >= JavaMinecraftVersion::V_1_9 && *version < JavaMinecraftVersion::V_1_17 {
            write.write_bool(self.tracking_position)?;
        }

        if *version >= JavaMinecraftVersion::V_1_14 {
            write.write_bool(self.locked)?;
        }

        if let Some(icons) = self.icons {
            if *version >= JavaMinecraftVersion::V_1_17 {
                write.write_bool(true)?;
            }
            write.write_var_int(&VarInt(icons.len() as i32))?;
            for icon in icons {
                icon.write_with_version(&mut write, version)?;
            }
        } else if *version >= JavaMinecraftVersion::V_1_17 {
            write.write_bool(false)?;
        } else {
            write.write_var_int(&VarInt(0))?;
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
