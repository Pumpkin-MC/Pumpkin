use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_SET_OBJECTIVE;
use pumpkin_macros::packet;
use pumpkin_util::text::TextComponent;

use crate::{
    ClientPacket, NumberFormat, VarInt,
    ser::{NetworkWriteExt, WritingError},
};

#[packet(PLAY_SET_OBJECTIVE)]
pub struct CUpdateObjectives {
    objective_name: String,
    mode: u8,
    display_name: TextComponent,
    render_type: VarInt,
    number_format: Option<NumberFormat>,
}

impl CUpdateObjectives {
    pub fn new(
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
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;

        write.write_string(&self.objective_name)?;
        write.write_u8_be(self.mode)?;
        if self.mode == 0 || self.mode == 2 {
            write.write_slice(&self.display_name.encode())?;
            write.write_var_int(&self.render_type)?;
            write.write_option(&self.number_format, |p, v| {
                match v {
                    NumberFormat::Blank => p.write_var_int(&VarInt(0)),
                    NumberFormat::Styled(style) => {
                        p.write_var_int(&VarInt(1))?;
                        // TODO
                        pumpkin_nbt::serializer::to_bytes_unnamed(style, p)
                            .map_err(|err| WritingError::Serde(err.to_string()))
                    }
                    NumberFormat::Fixed(text_component) => {
                        p.write_var_int(&VarInt(2))?;
                        p.write_slice(&text_component.encode())
                    }
                }
            })
        } else {
            Ok(())
        }
    }
}

pub enum Mode {
    Add,
    Remove,
    Update,
}

pub enum RenderType {
    Integer,
    Hearts,
}
