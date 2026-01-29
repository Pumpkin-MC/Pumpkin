use std::io::Write;

use crate::{
    WritingError,
    ser::{NetworkWriteExt, serializer::Serializer},
};
use pumpkin_data::packet::clientbound::PLAY_SET_EQUIPMENT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;
use serde::Serialize;

use crate::{
    ClientPacket,
    codec::{item_stack_seralizer::ItemStackSerializer, var_int::VarInt},
};

#[java_packet(PLAY_SET_EQUIPMENT)]
pub struct CSetEquipment {
    pub entity_id: VarInt,
    pub equipment: Vec<(i8, ItemStackSerializer<'static>)>,
}

impl CSetEquipment {
    #[must_use]
    pub const fn new(
        entity_id: VarInt,
        equipment: Vec<(i8, ItemStackSerializer<'static>)>,
    ) -> Self {
        Self {
            entity_id,
            equipment,
        }
    }
}

impl ClientPacket for CSetEquipment {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_var_int(&self.entity_id)?;
        for i in 0..self.equipment.len() {
            let equipment = &self.equipment[i];
            let slot = &equipment.0;
            if i == self.equipment.len() - 1 {
                write.write_i8(*slot)?;
            } else {
                write.write_i8(*slot | -128)?;
            }
            let mut serializer = Serializer::new(&mut write);
            equipment
                .1
                .serialize(&mut serializer)
                .expect("Could not serialize `EquipmentSlot`");
        }

        Ok(())
    }
}
