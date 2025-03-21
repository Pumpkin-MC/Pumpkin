use std::io::Write;

use crate::ser::{NetworkWriteExt, WritingError, serializer::Serializer};
use pumpkin_data::packet::clientbound::PLAY_SET_EQUIPMENT;
use pumpkin_macros::packet;
use serde::Serialize;

use crate::{
    ClientPacket,
    codec::{slot::Slot, var_int::VarInt},
};

#[packet(PLAY_SET_EQUIPMENT)]
pub struct CSetEquipment {
    entity_id: VarInt,
    equipment: Vec<(EquipmentSlot, Slot)>,
}

impl CSetEquipment {
    pub fn new(entity_id: VarInt, equipment: Vec<(EquipmentSlot, Slot)>) -> Self {
        Self {
            entity_id,
            equipment,
        }
    }
}

impl ClientPacket for CSetEquipment {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;

        write.write_var_int(&self.entity_id)?;
        for i in 0..self.equipment.len() {
            let equipment = &self.equipment[i];
            let slot = &equipment.0;
            if i != self.equipment.len() - 1 {
                write.write_i8_be(-128)?;
            } else {
                write.write_i8_be(*slot as i8)?;
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

#[derive(Clone, Copy)]
pub enum EquipmentSlot {
    MainHand,
    OffHand,
    Feet,
    Legs,
    Chest,
    Head,
    Body,
}
