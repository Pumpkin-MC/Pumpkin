use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;
use pumpkin_util::math::position::BlockPos;

use crate::entity::player::Player;
use crate::item::items::ignite::flint_and_steel::ignite_with_special_cases;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;

pub struct FireChargeItem;

impl ItemMetadata for FireChargeItem {
    fn ids() -> Box<[u16]> {
        [Item::FIRE_CHARGE.id].into()
    }
}

#[async_trait]
impl PumpkinItem for FireChargeItem {
    async fn use_on_block(
        &self,
        item: &Item,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        block: &Block,
        server: &Server,
    ) {
        ignite_with_special_cases(
            item,
            player,
            location,
            face,
            block,
            server,
            Sound::ItemFirechargeUse,
        )
        .await;
    }
}
