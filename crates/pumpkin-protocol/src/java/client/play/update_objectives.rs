use std::io::Write;

use pumpkin_data::packet::clientbound::play::SET_OBJECTIVE;
use pumpkin_macros::java_packet;
use pumpkin_util::{text::TextComponent, version::JavaMinecraftVersion};

use crate::{ClientPacket, NumberFormat, VarInt, WritingError, ser::NetworkWriteExt};

#[java_packet(SET_OBJECTIVE)]
pub struct CUpdateObjectives {
    pub objective_name: String,
    pub mode: u8,
    pub display_name: TextComponent,
    pub render_type: VarInt,
    pub number_format: Option<NumberFormat>,
}

impl CUpdateObjectives {
    #[must_use]
    pub const fn new(
        objective_name: String,
        mode: Mode,
        display_name: TextComponent,
        render_type: RenderType,
        number_format: Option<NumberFormat>,
    ) -> Self {
        Self {
            objective_name,
            mode: mode as u8,
            display_name,
            render_type: VarInt(render_type as i32),
            number_format,
        }
    }
}

impl ClientPacket for CUpdateObjectives {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_string(&self.objective_name)?;
        write.write_u8(self.mode)?;
        if self.mode == 0 || self.mode == 2 {
            if *version < JavaMinecraftVersion::V_1_13 {
                write.write_string(&self.display_name.clone().to_pretty_console())?;
                let render_str = if self.render_type.0 == 1 {
                    "hearts"
                } else {
                    "integer"
                };
                write.write_string(render_str)?;
            } else {
                write.write_component(&self.display_name, version)?;
                write.write_var_int(&self.render_type)?;
                if *version >= JavaMinecraftVersion::V_1_20_3 {
                    write.write_option(&self.number_format, |p, v| {
                        match v {
                            NumberFormat::Blank => p.write_var_int(&VarInt(0)),
                            NumberFormat::Styled(_style) => {
                                p.write_var_int(&VarInt(1))?;
                                let comp = pumpkin_nbt::compound::NbtCompound::new();
                                // Write style properties if any
                                let bytes = pumpkin_nbt::Nbt::from(comp).write_unnamed();
                                p.write_all(&bytes).map_err(WritingError::IoError)
                            }
                            NumberFormat::Fixed(text_component) => {
                                p.write_var_int(&VarInt(2))?;
                                p.write_component(text_component, version)
                            }
                        }
                    })?;
                }
            }
        }
        Ok(())
    }
}

pub enum Mode {
    Add,
    Remove,
    Update,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderType {
    Integer,
    Hearts,
}
