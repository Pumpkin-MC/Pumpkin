use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::CampfireLikeProperties;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;
use pumpkin_data::sound::SoundCategory;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;

use crate::entity::player::Player;
use crate::item::items::ignite::ignition::Ignition;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;

pub struct FlintAndSteelItem;

impl ItemMetadata for FlintAndSteelItem {
    fn ids() -> Box<[u16]> {
        [Item::FLINT_AND_STEEL.id].into()
    }
}

#[async_trait]
impl PumpkinItem for FlintAndSteelItem {
    async fn use_on_block(
        &self,
        item: &Item,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        block: &Block,
        server: &Server,
    ) {
        Ignition::ignite_block(
            |world: Arc<World>, pos: BlockPos, new_state_id: u16| async move {
                world
                    .set_block_state(&pos, new_state_id, BlockFlags::NOTIFY_ALL)
                    .await;
            },
            item,
            player,
            location,
            face,
            block,
            server,
        )
        .await;
        // TODO: check CandleBlock and CandleCakeBlock
        let world = player.world().await;
        let state = world.get_block_state(&location).await;
        if CampfireLikeProperties::handles_block_id(block.id)
            && !CampfireLikeProperties::from_state_id(state.id, block).lit
        {
            let mut props = CampfireLikeProperties::from_state_id(state.id, block);
            if !props.waterlogged && !props.lit {
                props.lit = true;
            }
            world
                .play_sound(
                    Sound::ItemFlintandsteelUse,
                    SoundCategory::Blocks,
                    &location.to_centered_f64(),
                )
                .await;
            world
                .set_block_state(&location, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
                .await;
        }
    }
}
