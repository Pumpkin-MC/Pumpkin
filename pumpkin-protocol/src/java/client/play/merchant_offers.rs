use pumpkin_data::packet::clientbound::PLAY_MERCHANT_OFFERS;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[derive(Clone)]
pub struct MerchantOffer {
    pub base_cost_a: ItemStackSerializer<'static>, // TODO: item cost
    pub output: ItemStackSerializer<'static>,
    pub cost_b: Option<ItemStackSerializer<'static>>, // TODO: item cost
    pub is_disabled: bool,
    pub uses: i32,
    pub max_uses: i32,
    pub xp: i32,
    pub special_price: i32,
    pub price_multiplier: f32,
    pub demand: i32,
}

impl MerchantOffer {
    fn write(
        &self,
        mut write: impl std::io::Write,
        version: JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        self.base_cost_a.write_with_version(&mut write, &version)?;
        self.output.write_with_version(&mut write, &version)?;
        write.write_option(&self.cost_b, |w, cost_b| {
            cost_b.write_with_version(w, &version)
        })?;
        write.write_bool(self.is_disabled)?;
        write.write_i32_be(self.uses)?;
        write.write_i32_be(self.max_uses)?;
        write.write_i32_be(self.xp)?;
        write.write_i32_be(self.special_price)?;
        write.write_f32_be(self.price_multiplier)?;
        write.write_i32_be(self.demand)?;
        Ok(())
    }
}

#[java_packet(PLAY_MERCHANT_OFFERS)]
pub struct CMerchantOffers {
    pub window_id: VarInt,
    pub offers: Vec<MerchantOffer>,
    pub villager_level: VarInt,
    pub experience: VarInt,
    pub is_regular_villager: bool,
    pub can_restock: bool,
}

impl CMerchantOffers {
    #[must_use]
    pub const fn new(
        window_id: VarInt,
        offers: Vec<MerchantOffer>,
        villager_level: VarInt,
        experience: VarInt,
        is_regular_villager: bool,
        can_restock: bool,
    ) -> Self {
        Self {
            window_id,
            offers,
            villager_level,
            experience,
            is_regular_villager,
            can_restock,
        }
    }
}

impl ClientPacket for CMerchantOffers {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.window_id)?;
        write.write_var_int(&VarInt(self.offers.len() as i32))?;
        for offer in &self.offers {
            offer.write(&mut write, *version)?;
        }
        write.write_var_int(&self.villager_level)?;
        write.write_var_int(&self.experience)?;
        write.write_bool(self.is_regular_villager)?;
        write.write_bool(self.can_restock)?;
        Ok(())
    }
}
