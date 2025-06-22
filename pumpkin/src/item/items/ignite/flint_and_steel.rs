use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::{
    BlockProperties, CampfireLikeProperties, CandleLikeProperties, RedstoneOreLikeProperties,
};
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::{Rng, rng};
use std::sync::Arc;

use crate::item::items::ignite::ignition::Ignition;

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
        ignite_with_special_cases(
            item,
            player,
            location,
            face,
            block,
            server,
            Sound::ItemFlintandsteelUse,
        )
        .await;
    }
}

pub(crate) async fn ignite_with_special_cases(
    item: &Item,
    player: &Player,
    location: BlockPos,
    face: BlockDirection,
    block: &Block,
    server: &Server,
    ignition_sound: Sound,
) {
    let world = player.world().await;
    let state = world.get_block_state(&location).await;

    match block.id {
        id if (Block::CANDLE.id..=Block::BLACK_CANDLE.id).contains(&id) => {
            let mut properties = CandleLikeProperties::from_state_id(state.id, block);
            if !properties.lit && !properties.waterlogged {
                properties.lit = true;

                world
                    .set_block_state(
                        &location,
                        properties.to_state_id(block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
                play_use_sound(player, &location, ignition_sound).await;
                return;
            }
        }
        id if (Block::CANDLE_CAKE.id..=Block::BLACK_CANDLE_CAKE.id).contains(&id) => {
            let mut properties = RedstoneOreLikeProperties::from_state_id(state.id, block);
            if !properties.lit {
                properties.lit = true;
                world
                    .set_block_state(
                        &location,
                        properties.to_state_id(block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;

                play_use_sound(player, &location, ignition_sound).await;
                return;
            }
        }
        id if id == Block::CAMPFIRE.id || id == Block::SOUL_CAMPFIRE.id => {
            let mut properties = CampfireLikeProperties::from_state_id(state.id, block);
            if !properties.lit && !properties.waterlogged {
                properties.lit = true;

                world
                    .set_block_state(
                        &location,
                        properties.to_state_id(block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;

                play_use_sound(player, &location, ignition_sound).await;
                return;
            }
        }
        _ => {}
    }
    Ignition::ignite_block(
        |world: Arc<World>, pos: BlockPos, new_state_id: u16| async move {
            world
                .set_block_state(&pos, new_state_id, BlockFlags::NOTIFY_ALL)
                .await;

            Ignition::run_fire_spread(world, &pos);
            // TODO
        },
        item,
        player,
        location,
        face,
        block,
        server,
    )
    .await;
}

async fn play_use_sound(player: &Player, pos: &BlockPos, sound: Sound) {
    let seed = rng().random::<f64>();
    let pitch = rng().random_range(0.8f32..1.2f32);
    player
        .play_sound(
            sound as u16,
            SoundCategory::Ambient,
            &pos.to_f64(),
            1.0,
            pitch,
            seed,
        )
        .await;
}
