use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_MERCHANT_OFFERS;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;
use pumpkin_world::item::ItemStack;

use crate::{
    ClientPacket, VarInt, WritingError, codec::item_stack_seralizer::ItemStackSerializer,
    ser::NetworkWriteExt,
};

/// A single trade offer in the merchant GUI.
pub struct MerchantTrade<'a> {
    pub input1: &'a ItemStack,
    pub input2: &'a ItemStack,
    pub output: &'a ItemStack,
    pub uses: i32,
    pub max_uses: i32,
    pub xp_reward: i32,
    pub special_price: i32,
    pub price_multiplier: f32,
    pub demand: i32,
}

/// Sends the full list of trades to the client for the merchant GUI.
#[java_packet(PLAY_MERCHANT_OFFERS)]
pub struct CMerchantOffers<'a> {
    pub window_id: VarInt,
    pub trades: &'a [MerchantTrade<'a>],
    pub villager_level: VarInt,
    pub villager_xp: VarInt,
    pub is_regular_villager: bool,
    pub can_restock: bool,
}

impl ClientPacket for CMerchantOffers<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_var_int(&self.window_id)?;
        let trade_count = i32::try_from(self.trades.len()).map_err(|_| {
            WritingError::Message(format!("{} trades do not fit in VarInt", self.trades.len()))
        })?;
        write.write_var_int(&VarInt(trade_count))?;

        for trade in self.trades {
            ItemStackSerializer(std::borrow::Cow::Borrowed(trade.input1))
                .write_with_version(&mut write, version)?;
            ItemStackSerializer(std::borrow::Cow::Borrowed(trade.output))
                .write_with_version(&mut write, version)?;
            ItemStackSerializer(std::borrow::Cow::Borrowed(trade.input2))
                .write_with_version(&mut write, version)?;
            write.write_bool(trade.uses >= trade.max_uses)?; // out_of_stock
            write.write_i32_be(trade.uses)?;
            write.write_i32_be(trade.max_uses)?;
            write.write_i32_be(trade.xp_reward)?;
            write.write_i32_be(trade.special_price)?;
            write.write_f32_be(trade.price_multiplier)?;
            write.write_i32_be(trade.demand)?;
        }

        write.write_var_int(&self.villager_level)?;
        write.write_var_int(&self.villager_xp)?;
        write.write_bool(self.is_regular_villager)?;
        write.write_bool(self.can_restock)?;

        Ok(())
    }
}
