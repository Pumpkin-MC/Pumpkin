use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketRead};

/// Sent by the client when selecting a recipe in the stonecutter container.
#[derive(Debug, PartialEq, Eq, PacketRead)]
#[packet(332)]
pub struct SStonecutterSetRecipe {
    /// Container ID of the stonecutter.
    pub container_id: u8,
    /// Selected recipe index.
    pub recipe_index: VarInt,
}

#[cfg(test)]
mod tests {
    use crate::serial::PacketWrite;

    use super::*;

    #[test]
    fn decodes_stonecutter_set_recipe() {
        let mut encoded = Vec::new();
        12u8.write(&mut encoded).unwrap();
        VarInt(5).write(&mut encoded).unwrap();

        let mut reader = encoded.as_slice();
        let packet = SStonecutterSetRecipe::read(&mut reader).expect("read stonecutter set recipe");
        assert_eq!(packet.container_id, 12);
        assert_eq!(packet.recipe_index.0, 5);
    }
}
