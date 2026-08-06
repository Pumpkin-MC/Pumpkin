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
    /// Vanilla `MerchantOffer::getModifiedCostCount` (`MerchantOffer.java:123-127`): the
    /// actual price charged/displayed, layering demand-driven inflation and the
    /// reputation/hero-of-the-village `special_price` discount on top of the trade
    /// definition's base count. Demand only ever raises the price (`max(0, ...)`);
    /// `special_price` can be negative.
    #[must_use]
    pub fn modified_cost_count(
        base_count: i32,
        demand: i32,
        price_multiplier: f32,
        special_price: i32,
        max_stack_size: i32,
    ) -> i32 {
        #[allow(clippy::cast_possible_truncation)]
        let demand_diff = ((base_count as f32) * (demand as f32) * price_multiplier)
            .floor()
            .max(0.0) as i32;
        (base_count + demand_diff + special_price).clamp(1, max_stack_size.max(1))
    }

    /// Vanilla `MerchantOffer::updateDemand` (`MerchantOffer.java:145-147`).
    pub const fn update_demand(&mut self) {
        self.demand += self.uses - (self.max_uses - self.uses);
    }

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
        // Vanilla `MerchantOffer.writeToStream` (`MerchantOffer.java:242`) writes
        // `offer.isOutOfStock()` (`uses >= maxUses`) in this slot, not `rewardExp` -- that
        // field is never sent over the wire at all, only through NBT (`rewardExp` key).
        write.write_bool(self.uses >= self.max_uses)?;
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

#[cfg(test)]
mod test {
    use super::MerchantOffer;

    #[test]
    fn modified_cost_count_applies_demand_and_special_price() {
        // No demand or discount: unchanged.
        assert_eq!(MerchantOffer::modified_cost_count(10, 0, 0.05, 0, 64), 10);
        // Demand raises price: floor(10 * 3 * 0.05) = 1.
        assert_eq!(MerchantOffer::modified_cost_count(10, 3, 0.05, 0, 64), 11);
        // Negative demand never lowers price below base (max(0, ...)).
        assert_eq!(MerchantOffer::modified_cost_count(10, -5, 0.05, 0, 64), 10);
        // Special price (reputation discount) can reduce below base.
        assert_eq!(MerchantOffer::modified_cost_count(10, 0, 0.05, -4, 64), 6);
    }

    #[test]
    fn modified_cost_count_clamps_between_one_and_max_stack() {
        assert_eq!(MerchantOffer::modified_cost_count(1, 0, 0.05, -10, 64), 1);
        assert_eq!(MerchantOffer::modified_cost_count(60, 20, 0.5, 0, 64), 64);
    }

    #[test]
    fn update_demand_matches_vanilla_formula() {
        let mut offer = MerchantOffer {
            base_cost_a: crate::codec::item_stack_seralizer::ItemStackSerializer(
                std::borrow::Cow::Owned(pumpkin_data::item_stack::ItemStack::EMPTY.clone()),
            ),
            output: crate::codec::item_stack_seralizer::ItemStackSerializer(
                std::borrow::Cow::Owned(pumpkin_data::item_stack::ItemStack::EMPTY.clone()),
            ),
            cost_b: None,
            is_disabled: false,
            uses: 12,
            max_uses: 12,
            xp: 2,
            special_price: 0,
            price_multiplier: 0.05,
            demand: 0,
        };
        // demand += uses - (max_uses - uses) = 12 - 0 = 12
        offer.update_demand();
        assert_eq!(offer.demand, 12);

        offer.uses = 0;
        offer.update_demand();
        // demand += 0 - 12 = 12 - 12 = 0
        assert_eq!(offer.demand, 0);
    }

    fn test_offer(is_disabled: bool, uses: i32, max_uses: i32) -> MerchantOffer {
        MerchantOffer {
            base_cost_a: crate::codec::item_stack_seralizer::ItemStackSerializer(
                std::borrow::Cow::Owned(pumpkin_data::item_stack::ItemStack::EMPTY.clone()),
            ),
            output: crate::codec::item_stack_seralizer::ItemStackSerializer(
                std::borrow::Cow::Owned(pumpkin_data::item_stack::ItemStack::EMPTY.clone()),
            ),
            cost_b: None,
            is_disabled,
            uses,
            max_uses,
            xp: 2,
            special_price: 0,
            price_multiplier: 0.05,
            demand: 0,
        }
    }

    fn write_bytes(offer: &MerchantOffer) -> Vec<u8> {
        let mut buf = Vec::new();
        offer
            .write(
                &mut buf,
                pumpkin_util::version::JavaMinecraftVersion::V_1_21_11,
            )
            .unwrap();
        buf
    }

    #[test]
    fn wire_write_ignores_is_disabled_reward_exp_flag() {
        // `is_disabled` (populated from `!rewardExp` elsewhere) must have no effect on the
        // wire encoding: vanilla `MerchantOffer.writeToStream` (`MerchantOffer.java:242`)
        // never sends `rewardExp` over the network at all.
        let a = write_bytes(&test_offer(false, 3, 12));
        let b = write_bytes(&test_offer(true, 3, 12));
        assert_eq!(a, b);
    }

    #[test]
    fn wire_write_sends_is_out_of_stock_not_is_disabled() {
        // Vanilla `MerchantOffer.writeToStream` (`MerchantOffer.java:242`) writes
        // `offer.isOutOfStock()`, i.e. `uses >= maxUses`, in the boolean slot right after the
        // optional second cost. `is_disabled` true here must not force that byte to 1, and
        // `uses >= max_uses` must force it to 1 even when `is_disabled` is false.
        let not_exhausted = write_bytes(&test_offer(true, 3, 12));
        let exhausted = write_bytes(&test_offer(false, 12, 12));

        // The flag is derived from `uses`, so flipping it also changes the `uses` i32 field
        // written right after it -- but the flag byte itself always comes first, so it is
        // always the first differing byte.
        assert_eq!(not_exhausted.len(), exhausted.len());
        let diff_positions: Vec<usize> = not_exhausted
            .iter()
            .zip(exhausted.iter())
            .enumerate()
            .filter(|(_, (x, y))| x != y)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(diff_positions.len(), 2);
        let pos = diff_positions[0];
        assert_eq!(not_exhausted[pos], 0);
        assert_eq!(exhausted[pos], 1);
    }
}
