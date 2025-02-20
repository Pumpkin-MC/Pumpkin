use crate::bytebuf::{serializer::Serializer, ByteBufMut};
use bytes::BytesMut;
use pumpkin_data::packet::clientbound::PLAY_SET_EQUIPMENT;
use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::{
    codec::{slot::Slot, var_int::VarInt},
    ClientPacket,
};

#[client_packet(PLAY_SET_EQUIPMENT)]
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
    fn write(&self, bytebuf: &mut impl bytes::BufMut) {
        bytebuf.put_var_int(&self.entity_id);
        for i in 0..self.equipment.len() {
            let equipment = &self.equipment[i];
            let slot = &equipment.0;
            if i != self.equipment.len() - 1 {
                bytebuf.put_i8(-128);
            } else {
                bytebuf.put_i8(*slot as i8);
            }
            let buf = BytesMut::new();
            let mut serializer = Serializer::new(buf);
            equipment
                .1
                .serialize(&mut serializer)
                .expect("Could not serialize packet");
            bytebuf.put(serializer.output);
        }
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
