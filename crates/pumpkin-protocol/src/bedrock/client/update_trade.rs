// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_nbt::compound::NbtCompound;

use crate::{
    codec::{var_int::VarInt, var_long::VarLong},
    serial::PacketWrite,
};

/// Opens or refreshes a Bedrock merchant screen with its complete offer list.
#[derive(PacketWrite)]
#[packet(80)]
pub struct CUpdateTrade {
    /// Window identifier of the active trade screen.
    pub container_id: u8,
    /// Window type identifier (typically 15 for villager trading).
    pub r#type: u8,
    /// Count of trading options available.
    pub size: VarInt,
    /// Experience or progression tier of the trading merchant.
    pub trader_tier: VarInt,
    /// Unique identifier of the merchant entity.
    pub entity_unique_id: VarLong,
    /// Unique entity identifier of the last player to interact with this merchant.
    pub last_trading_player: VarLong,
    /// Visible title displayed at the top of the trading interface.
    pub display_name: String,
    /// Whether to display the modern villager trading UI.
    pub use_new_trade_screen: bool,
    /// Whether offer pricing incorporates demand and supply economics.
    pub using_economy_trade: bool,
    /// Network NBT compound containing full recipe and trade offer definitions.
    pub data: NbtCompound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Packet, serial::PacketWrite};

    #[test]
    fn update_trade_uses_current_packet_id_and_network_nbt() {
        assert_eq!(<CUpdateTrade as Packet>::PACKET_ID, 80);

        let mut offers = NbtCompound::new();
        offers.put_list("Recipes", Vec::new());
        let packet = CUpdateTrade {
            container_id: 1,
            r#type: 15,
            size: VarInt(0),
            trader_tier: VarInt(0),
            entity_unique_id: VarLong(2),
            last_trading_player: VarLong(3),
            display_name: "Villager".to_string(),
            use_new_trade_screen: true,
            using_economy_trade: true,
            data: offers,
        };

        let mut encoded = Vec::new();
        packet.write(&mut encoded).unwrap();
        assert!(encoded.ends_with(&[0]));
    }
}
