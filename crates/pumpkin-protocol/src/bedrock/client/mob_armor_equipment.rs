// Last verified for v2225

use pumpkin_macros::packet;

use crate::{
    bedrock::network_item::NetworkItemStackDescriptor, codec::var_ulong::VarULong,
    serial::PacketWrite,
};

/// Sent by the server to synchronize the armor and body equipment worn by an actor.
#[derive(PacketWrite, Debug)]
#[packet(32)]
pub struct CMobArmorEquipment {
    /// Runtime entity ID of the equipped actor.
    pub target_runtime_id: VarULong,
    /// Item equipped in the head (helmet) slot.
    pub head: NetworkItemStackDescriptor,
    /// Item equipped in the chest (chestplate/elytra) slot.
    pub chest: NetworkItemStackDescriptor,
    /// Item equipped in the legs (leggings) slot.
    pub legs: NetworkItemStackDescriptor,
    /// Item equipped in the feet (boots) slot.
    pub feet: NetworkItemStackDescriptor,
    /// Item equipped in the body (horse/wolf armor) slot.
    pub body: NetworkItemStackDescriptor,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mob_armor_equipment_empty_encodes_five_empty_descriptors() {
        let packet = CMobArmorEquipment {
            target_runtime_id: VarULong(7),
            head: NetworkItemStackDescriptor::default(),
            chest: NetworkItemStackDescriptor::default(),
            legs: NetworkItemStackDescriptor::default(),
            feet: NetworkItemStackDescriptor::default(),
            body: NetworkItemStackDescriptor::default(),
        };
        let mut buf = Vec::new();
        packet.write(&mut buf).unwrap();
        // target_runtime_id = 7 (0x07), followed by 5 empty 8-byte ItemStackWrapper descriptors
        let mut expected = vec![0x07];
        expected.extend_from_slice(&[0x00; 5 * 8]);
        assert_eq!(buf, expected);
    }
}
