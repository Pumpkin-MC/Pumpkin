use pumpkin_data::packet::clientbound::PLAY_SET_OBJECTIVE;
use pumpkin_macros::packet;
use pumpkin_util::text::TextComponent;

use crate::{
    ClientPacket, NumberFormat, VarInt,
    ser::{NetworkWrite, WritingError},
};

#[packet(PLAY_SET_OBJECTIVE)]
pub struct CUpdateObjectives<'a> {
    objective_name: &'a str,
    mode: u8,
    display_name: TextComponent,
    render_type: VarInt,
    number_format: Option<NumberFormat>,
}

impl<'a> CUpdateObjectives<'a> {
    pub fn new(
        objective_name: &'a str,
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

impl ClientPacket for CUpdateObjectives<'_> {
    fn write(&self, write: impl NetworkWrite) -> Result<(), WritingError> {
        let mut write = write;

        write.write_string(self.objective_name)?;
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
                        let mut style_buf = Vec::new();
                        pumpkin_nbt::serializer::to_bytes_unnamed(style, &mut style_buf).unwrap();
                        p.write_slice(&style_buf)
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
