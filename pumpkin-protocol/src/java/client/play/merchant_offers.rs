use pumpkin_data::packet::clientbound::PLAY_MERCHANT_OFFERS;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;

#[derive(Serialize, Deserialize, Clone)]
pub struct MerchantOffer {
    pub base_cost_a: ItemStackSerializer<'static>,
    pub output: ItemStackSerializer<'static>,
    pub cost_b: Option<ItemStackSerializer<'static>>,
    pub is_disabled: bool,
    pub uses: i32,
    pub max_uses: i32,
    pub xp: i32,
    pub special_price: i32,
    pub price_multiplier: f32,
    pub demand: i32,
}

#[derive(Serialize)]
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
