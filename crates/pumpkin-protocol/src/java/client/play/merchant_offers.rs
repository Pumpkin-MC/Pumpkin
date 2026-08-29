use pumpkin_data::packet::clientbound::play::MERCHANT_OFFERS;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[derive(Clone)]
pub struct MerchantOffer {
    pub base_cost_a: ItemStackSerializer<'static>,
    pub output: ItemStackSerializer<'static>,
    pub cost_b: Option<ItemStackSerializer<'static>>,
    pub reward_exp: bool,
    pub uses: i32,
    pub max_uses: i32,
    pub xp: i32,
    pub special_price: i32,
    pub price_multiplier: f32,
    pub demand: i32,
}

impl MerchantOffer {
    #[must_use]
    pub const fn is_out_of_stock(&self) -> bool {
        self.uses >= self.max_uses
    }

    #[must_use]
    pub const fn needs_restock(&self) -> bool {
        self.uses > 0
    }

    pub const fn update_demand(&mut self) {
        self.demand += self.uses - (self.max_uses - self.uses);
    }

    pub const fn reset_uses(&mut self) {
        self.uses = 0;
    }

    pub fn write(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        if *version >= JavaMinecraftVersion::V_1_20_5 {
            self.base_cost_a
                .write_item_cost_with_version(&mut write, version)?;
            self.output.write_with_version(&mut write, version)?;
            write.write_option(&self.cost_b, |w, cost_b| {
                cost_b.write_item_cost_with_version(w, version)
            })?;
        } else {
            self.base_cost_a.write_with_version(&mut write, version)?;
            self.output.write_with_version(&mut write, version)?;
            if let Some(cost_b) = &self.cost_b {
                cost_b.write_with_version(&mut write, version)?;
            } else {
                write.write_bool(false)?;
            }
        }
        write.write_bool(self.reward_exp)?;
        write.write_i32_be(self.uses)?;
        write.write_i32_be(self.max_uses)?;
        write.write_i32_be(self.xp)?;
        write.write_i32_be(self.special_price)?;
        write.write_f32_be(self.price_multiplier)?;
        write.write_i32_be(self.demand)?;
        Ok(())
    }
}

#[java_packet(MERCHANT_OFFERS)]
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
        if *version >= JavaMinecraftVersion::V_1_19 {
            write.write_var_int(&VarInt(self.offers.len() as i32))?;
        } else {
            write.write_u8(self.offers.len() as u8)?;
        }
        for offer in &self.offers {
            offer.write(&mut write, version)?;
        }
        write.write_var_int(&self.villager_level)?;
        write.write_var_int(&self.experience)?;
        write.write_bool(self.is_regular_villager)?;
        write.write_bool(self.can_restock)?;
        Ok(())
    }
}

impl<'a> crate::ServerPacket<'a> for CMerchantOffers {
    fn read(
        bytebuf: &mut &'a [u8],
        version: &JavaMinecraftVersion,
    ) -> Result<Self, crate::ser::ReadingError> {
        use crate::ser::NetworkReadExt;
        let window_id = bytebuf.get_var_int()?;
        let offers_count = if *version >= JavaMinecraftVersion::V_1_19 {
            bytebuf.get_var_int()?.0 as usize
        } else {
            bytebuf.get_u8()? as usize
        };

        let mut offers = Vec::with_capacity(offers_count);
        for _ in 0..offers_count {
            let (base_cost_a, output, cost_b) = if *version >= JavaMinecraftVersion::V_1_20_5 {
                let item_id = bytebuf.get_var_int()?.0 as u16;
                let count = bytebuf.get_var_int()?.0 as u8;
                let comp_count = bytebuf.get_var_int()?.0 as usize;
                for _ in 0..comp_count {
                    let _comp_id = bytebuf.get_var_int()?;
                }
                let item = pumpkin_data::item::Item::from_id(item_id)
                    .unwrap_or(&pumpkin_data::item::Item::AIR);
                let base_cost_a = ItemStackSerializer(std::borrow::Cow::Owned(
                    pumpkin_data::item_stack::ItemStack::new(count, item),
                ));
                let output = ItemStackSerializer::read_with_version(bytebuf, version)?;
                let has_cost_b = bytebuf.get_bool()?;
                let cost_b = if has_cost_b {
                    let item_id_b = bytebuf.get_var_int()?.0 as u16;
                    let count_b = bytebuf.get_var_int()?.0 as u8;
                    let comp_count_b = bytebuf.get_var_int()?.0 as usize;
                    for _ in 0..comp_count_b {
                        let _comp_id = bytebuf.get_var_int()?;
                    }
                    let item_b = pumpkin_data::item::Item::from_id(item_id_b)
                        .unwrap_or(&pumpkin_data::item::Item::AIR);
                    Some(ItemStackSerializer(std::borrow::Cow::Owned(
                        pumpkin_data::item_stack::ItemStack::new(count_b, item_b),
                    )))
                } else {
                    None
                };
                (base_cost_a, output, cost_b)
            } else {
                let base_cost_a = ItemStackSerializer::read_with_version(bytebuf, version)?;
                let output = ItemStackSerializer::read_with_version(bytebuf, version)?;
                let cost_b_stack = ItemStackSerializer::read_with_version(bytebuf, version)?;
                let cost_b = if cost_b_stack.0.is_empty() {
                    None
                } else {
                    Some(cost_b_stack)
                };
                (base_cost_a, output, cost_b)
            };

            let reward_exp = bytebuf.get_bool()?;
            let uses = bytebuf.get_i32_be()?;
            let max_uses = bytebuf.get_i32_be()?;
            let xp = bytebuf.get_i32_be()?;
            let special_price = bytebuf.get_i32_be()?;
            let price_multiplier = bytebuf.get_f32_be()?;
            let demand = bytebuf.get_i32_be()?;

            offers.push(MerchantOffer {
                base_cost_a,
                output,
                cost_b,
                reward_exp,
                uses,
                max_uses,
                xp,
                special_price,
                price_multiplier,
                demand,
            });
        }

        let villager_level = bytebuf.get_var_int()?;
        let experience = bytebuf.get_var_int()?;
        let is_regular_villager = bytebuf.get_bool()?;
        let can_restock = bytebuf.get_bool()?;

        Ok(Self {
            window_id,
            offers,
            villager_level,
            experience,
            is_regular_villager,
            can_restock,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, io::Cursor};

    use pumpkin_data::{
        data_component::DataComponent,
        data_component_impl::{
            DataComponentImpl, DyedColorImpl, ItemNameImpl, MapIdImpl, SuspiciousStewEffect,
            SuspiciousStewEffectsImpl,
        },
        item::Item,
        item_id_remap::remap_item_id_for_version,
        item_stack::ItemStack,
    };
    use pumpkin_util::version::JavaMinecraftVersion;

    use crate::ser::NetworkReadExt;

    use super::*;

    fn offer() -> MerchantOffer {
        MerchantOffer {
            base_cost_a: ItemStackSerializer(Cow::Owned(ItemStack::new(12, &Item::EMERALD))),
            output: ItemStackSerializer(Cow::Owned(ItemStack::new(1, &Item::BOOK))),
            cost_b: None,
            reward_exp: true,
            uses: 0,
            max_uses: 12,
            xp: 1,
            special_price: 0,
            price_multiplier: 0.05,
            demand: 0,
        }
    }

    #[test]
    fn merchant_inputs_use_item_cost_encoding() {
        let version = JavaMinecraftVersion::V_26_2;
        let packet =
            CMerchantOffers::new(VarInt(1), vec![offer()], VarInt(1), VarInt(0), true, true);
        let mut bytes = Vec::new();
        packet.write_packet_data(&mut bytes, &version).unwrap();
        let mut cursor = Cursor::new(&bytes);

        assert_eq!(cursor.get_var_int().unwrap(), VarInt(1));
        assert_eq!(cursor.get_var_int().unwrap(), VarInt(1));
        assert_eq!(
            cursor.get_var_int().unwrap(),
            VarInt::from(remap_item_id_for_version(Item::EMERALD.id, version))
        );
        assert_eq!(cursor.get_var_int().unwrap(), VarInt(12));
        assert_eq!(cursor.get_var_int().unwrap(), VarInt(0));
    }

    #[test]
    fn merchant_offer_is_out_of_stock_at_max_uses() {
        let mut offer = offer();
        offer.uses = offer.max_uses - 1;
        assert!(!offer.is_out_of_stock());
        offer.uses += 1;
        assert!(offer.is_out_of_stock());
    }

    #[test]
    fn restock_updates_demand_before_resetting_uses() {
        let mut offer = offer();
        offer.uses = 8;

        assert!(offer.needs_restock());
        offer.update_demand();
        offer.reset_uses();

        assert_eq!(offer.demand, 4);
        assert_eq!(offer.uses, 0);
        assert!(!offer.needs_restock());
    }

    #[test]
    fn dynamic_villager_results_have_network_codecs() {
        let mut dyed = ItemStack::new(1, &Item::LEATHER_CHESTPLATE);
        dyed.patch.push((
            DataComponent::DyedColor,
            Some(DyedColorImpl { rgb: 0x12_34_56 }.to_dyn()),
        ));
        let mut stew = ItemStack::new(1, &Item::SUSPICIOUS_STEW);
        stew.patch.push((
            DataComponent::SuspiciousStewEffects,
            Some(
                SuspiciousStewEffectsImpl {
                    effects: Cow::Owned(vec![SuspiciousStewEffect {
                        effect: Cow::Borrowed("minecraft:night_vision"),
                        duration: 100,
                    }]),
                }
                .to_dyn(),
            ),
        ));
        let mut map = ItemStack::new(1, &Item::FILLED_MAP);
        map.patch
            .push((DataComponent::MapId, Some(MapIdImpl { id: 1 }.to_dyn())));
        map.patch.push((
            DataComponent::ItemName,
            Some(
                ItemNameImpl {
                    name: Cow::Borrowed("filled_map.mansion"),
                }
                .to_dyn(),
            ),
        ));

        for output in [dyed, stew, map] {
            let mut dynamic_offer = offer();
            dynamic_offer.output = ItemStackSerializer(Cow::Owned(output));
            let packet = CMerchantOffers::new(
                VarInt(1),
                vec![dynamic_offer],
                VarInt(1),
                VarInt(0),
                true,
                true,
            );
            packet
                .write_packet_data(&mut Vec::new(), &JavaMinecraftVersion::V_26_2)
                .unwrap();
        }
    }
}
