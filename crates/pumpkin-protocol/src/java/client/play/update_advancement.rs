use crate::codec::item_stack_seralizer::ItemStackTemplateSerializer;
use crate::codec::var_int::VarInt;
use pumpkin_data::Advancement;
use pumpkin_data::advancement_data::AdvancementProgressData;
use pumpkin_data::packet::clientbound::PLAY_UPDATE_ADVANCEMENTS;
use pumpkin_macros::java_packet;
use pumpkin_util::identifier::Identifier;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_UPDATE_ADVANCEMENTS)]
#[allow(unused)]
pub struct CUpdateAdvancements {
    pub reset: bool,
    pub added: Vec<&'static Advancement>,
    pub removed: Vec<Identifier>,
    pub progress: Vec<AdvancementProgressData>,
    pub show_advancements: bool,
}

impl CUpdateAdvancements {
    #[must_use]
    #[allow(unused)]
    pub const fn new(
        reset: bool,
        added: Vec<&'static Advancement>,
        progress: Vec<AdvancementProgressData>,
        removed: Vec<Identifier>,
        show_advancements: bool,
    ) -> Self {
        Self {
            reset,
            added,
            removed,
            progress,
            show_advancements,
        }
    }
}

impl ClientPacket for CUpdateAdvancements {
    #[allow(clippy::unimplemented)]
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_bool(self.reset)?;

        write.write_var_int(&VarInt(self.added.len() as i32))?;
        for adv in &self.added {
            write.write_string(&adv.id.to_string())?;

            let has_parent = adv.parent.is_some();
            write.write_bool(has_parent)?;
            if let Some(ref p) = adv.parent {
                write.write_string(&p.to_string())?;
            }

            let has_display = adv.display.is_some();
            write.write_bool(has_display)?;
            if let Some(display) = adv.display {
                write.write_slice(&display.get_title().encode())?;
                write.write_slice(&display.get_description().encode())?;

                // Item icon
                ItemStackTemplateSerializer::from(display.item_icon.clone())
                    .write_with_version(&mut write, version)?;

                write.write_var_int(&VarInt(display.frame_type as i32))?;
                let flags = (display.has_background() as i32)
                    | ((display.show_toast as i32) << 1)
                    | ((display.hidden as i32) << 2);
                write.write_i32_be(flags)?;
                if let Some(bg) = display.background_texture {
                    write.write_string(bg)?;
                }
                write.write_f32_be(display.x)?;
                write.write_f32_be(display.y)?;
            }

            write.write_var_int(&VarInt(adv.requirements.len() as i32))?;
            for req in adv.requirements {
                write.write_var_int(&VarInt(req.len() as i32))?;
                for r in *req {
                    write.write_string(r)?;
                }
            }

            write.write_bool(adv.send_telemetry)?;
        }

        write.write_var_int(&VarInt(self.removed.len() as i32))?;
        for rem in &self.removed {
            write.write_string(&rem.to_string())?;
        }

        write.write_var_int(&VarInt(self.progress.len() as i32))?;
        for prog in &self.progress {
            write.write_string(&prog.id.to_string())?;
            write.write_var_int(&VarInt(prog.progress.len() as i32))?;
            for crit in &prog.progress {
                write.write_string(&crit.criterion_id)?;
                let has_date = crit.achieve_date.is_some();
                write.write_bool(has_date)?;
                if let Some(date) = crit.achieve_date {
                    write.write_i64_be(date)?;
                }
            }
        }

        write.write_bool(self.show_advancements)?;

        Ok(())
    }
}
