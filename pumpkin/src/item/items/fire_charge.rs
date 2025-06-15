use crate::block::blocks::fire::FireBlockBase;
use crate::entity::player::Player;
use crate::item::pumpkin_item::{ItemMetadata, PumpkinItem};
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::block_properties::{
    BlockProperties, CampfireLikeProperties, CandleLikeProperties
};
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockFlags, SimpleWorld};
use rand::{Rng, rng};
use std::sync::Arc;

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
        _item: &Item,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        _block: &Block,
        _server: &Server,
    ) {
        let world = player.world().await;
        let (block, state) = world.get_block_and_block_state(&location).await;

        match block.id {
            id if (id >= Block::CANDLE.id && id <= Block::BLACK_CANDLE.id)
                || (id >= Block::CANDLE_CAKE.id && id <= Block::BLACK_CANDLE_CAKE.id) =>
            {
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
                        play_fire_charge_use_sound(&player, &pos).await;
                    }
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

                    play_fire_charge_use_sound(&player, &location).await;
                } else {
                    let pos = location.offset(face.to_offset());

                    if FireBlockBase::can_place_at(world.as_ref(), &pos).await {
                        place_fire(&pos, &world).await;
                        play_fire_charge_use_sound(&player, &pos).await;
                    }
                }
            }
            _ => {
                let pos = location.offset(face.to_offset());
                
                if FireBlockBase::can_place_at(world.as_ref(), &pos).await {
                    place_fire(&pos, &world).await;
                    play_fire_charge_use_sound(&player, &pos).await;
                }
            }
        }
    }
}

pub(crate) async fn place_fire(pos: &BlockPos, world: &Arc<World>) {
    let fire_block = FireBlockBase::get_fire_type(&world, &pos).await;

    world
        .set_block_state(&pos, fire_block.default_state_id, BlockFlags::NOTIFY_ALL)
        .await;
}

async fn play_fire_charge_use_sound(player: &Player, pos: &BlockPos) {
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
