// Last verified for v2225

use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

/// Sent by the server to update container property data (e.g. furnace burn/cook progress or brewing stand timers).
#[derive(PacketWrite, Debug)]
#[packet(51)]
pub struct CContainerSetData {
    /// Window ID assigned to the open container session.
    pub container_id: u8,
    /// Property key identifier within the container.
    pub id: VarInt,
    /// Updated value for the property.
    pub value: VarInt,
}

impl CContainerSetData {
    pub const FURNACE_TICK_COUNT: i32 = 0;
    pub const FURNACE_LIT_TIME: i32 = 1;
    pub const FURNACE_LIT_DURATION: i32 = 2;
    pub const FURNACE_STORED_XP: i32 = 3;
    pub const FURNACE_FUEL_AUX: i32 = 4;

    pub const BREWING_STAND_BREW_TIME: i32 = 0;
    pub const BREWING_STAND_FUEL_AMOUNT: i32 = 1;
    pub const BREWING_STAND_FUEL_TOTAL: i32 = 2;

    #[must_use]
    pub const fn new(container_id: u8, id: i32, value: i32) -> Self {
        Self {
            container_id,
            id: VarInt(id),
            value: VarInt(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_set_data_encodes_expected_bytes() {
        let packet = CContainerSetData::new(2, CContainerSetData::FURNACE_LIT_TIME, 200);
        let mut buf = Vec::new();
        packet.write(&mut buf).unwrap();
        // container_id = 2 (0x02), id = 1 -> ZigZag 2 (0x02), value = 200 -> ZigZag 400 (0x90, 0x03)
        assert_eq!(buf, vec![0x02, 0x02, 0x90, 0x03]);
    }
}
