use crate::block::blocks::fire::FireBlockBase;
use crate::entity::player::Player;
use crate::item::items::fire_charge::place_fire;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;
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
        _item: &Item,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        _block: &Block,
        _server: &Server,
    ) {
        // TODO: check CampfireBlock, CandleBlock and CandleCakeBlock
        let world = player.world().await;
        let (block, state) = world.get_block_and_block_state(&location).await;

        match block.id {
            id if (Block::CANDLE.id..=Block::BLACK_CANDLE.id).contains(&id) => {
                let mut properties = CandleLikeProperties::from_state_id(state.id, &block);
                if !properties.lit && !properties.waterlogged {
                    properties.lit = true;

                    world
                        .set_block_state(
                            &location,
                            properties.to_state_id(&block),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                } else {
                    let pos = location.offset(face.to_offset());

                    if FireBlockBase::can_place_at(world.as_ref(), &pos).await {
                        place_fire(&pos, &world).await;
                        play_flint_and_steel_use_sound(player, &pos).await;
                    }
                }
            }
            id if (Block::CANDLE_CAKE.id..=Block::BLACK_CANDLE_CAKE.id).contains(&id) => {
                let mut properties = RedstoneOreLikeProperties::from_state_id(state.id, &block);
                if properties.lit {
                    let pos = location.offset(face.to_offset());

                    if FireBlockBase::can_place_at(world.as_ref(), &pos).await {
                        place_fire(&pos, &world).await;
                        play_flint_and_steel_use_sound(player, &pos).await;
                    }
                } else {
                    properties.lit = true;
                    world
                        .set_block_state(
                            &location,
                            properties.to_state_id(&block),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                }
            }
            id if id == Block::CAMPFIRE.id || id == Block::SOUL_CAMPFIRE.id => {
                let mut properties = CampfireLikeProperties::from_state_id(state.id, &block);
                if !properties.lit && !properties.waterlogged {
                    properties.lit = true;

                    world
                        .set_block_state(
                            &location,
                            properties.to_state_id(&block),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;

                    play_flint_and_steel_use_sound(player, &location).await;
                } else {
                    let pos = location.offset(face.to_offset());

                    if FireBlockBase::can_place_at(world.as_ref(), &pos).await {
                        place_fire(&pos, &world).await;
                        play_flint_and_steel_use_sound(player, &pos).await;
                    }
                }
            }
            _ => {
                let pos = location.offset(face.to_offset());

                if FireBlockBase::can_place_at(world.as_ref(), &pos).await {
                    place_fire(&pos, &world).await;
                    play_flint_and_steel_use_sound(player, &pos).await;
                }
            }
        }
    }
}

async fn play_flint_and_steel_use_sound(player: &Player, pos: &BlockPos) {
    let seed = rng().random::<f64>();
    let pitch = rng().random_range(0.8f32..1.2f32);
    player
        .play_sound(
            Sound::ItemFirechargeUse as u16,
            SoundCategory::Ambient,
            &pos.to_f64(),
            1.0,
            pitch,
            seed,
        )
        .await;
}
