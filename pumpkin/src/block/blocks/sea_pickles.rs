use crate::block::BlockIsReplacing;
use crate::block::pumpkin_block::PumpkinBlock;
use crate::block::registry::BlockActionResult;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, Integer1To4};
use pumpkin_data::entity::EntityPose;
use pumpkin_data::item::Item;
use pumpkin_data::tag::Tagable;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::Rng;
use std::sync::Arc;

type SeaPickleProperties = pumpkin_data::block_properties::SeaPickleLikeProperties;

#[pumpkin_block("minecraft:sea_pickle")]
pub struct SeaPickleBlock;

#[async_trait]
impl PumpkinBlock for SeaPickleBlock {
    #[allow(clippy::many_single_char_names)]
    async fn use_with_item(
        &self,
        block: &Block,
        _player: &Player,
        location: BlockPos,
        item: &Item,
        _server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        if item != &Item::BONE_MEAL
            || !world
                .get_block(&location.down())
                .await
                .is_tagged_with("minecraft:coral_blocks")
                .unwrap()
            || !SeaPickleProperties::from_state_id(world.get_block_state_id(&location).await, block)
                .waterlogged
        {
            return BlockActionResult::Continue;
        }

        //1:1 vanilla algorithm
        //TODO use pumpkin random

        //let mut j = 1;
        let mut count = 0;
        let base_x = location.0.x - 2;
        let mut removed_z = 0;
        for added_x in 0..5 {
            for added_z in 0..1 {
                let temp_y = 2 + location.0.y - 1;
                for y in (temp_y - 2)..temp_y {
                    //let mut lv2: BlockState;
                    let lv = BlockPos::new(base_x + added_x, y, location.0.z - removed_z + added_z);
                    if lv == location
                        || rand::rng().random_range(0..6) != 0
                        || !world.get_block(&lv).await.eq(&Block::WATER)
                        || !world
                            .get_block(&lv.down())
                            .await
                            .is_tagged_with("minecraft:coral_blocks")
                            .unwrap()
                    {
                        continue;
                    }
                    let mut sea_pickle_prop = SeaPickleProperties::default(block);

                    sea_pickle_prop.pickles = match rand::rng().random_range(0..4) + 1 {
                        1 => Integer1To4::L1,
                        2 => Integer1To4::L2,
                        3 => Integer1To4::L3,
                        _ => Integer1To4::L4,
                    };
                    world
                        .set_block_state(
                            &lv,
                            sea_pickle_prop.to_state_id(block),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                }
            }
            if count < 2 {
                //j += 2;
                removed_z += 1;
            } else {
                //j -= 2;
                removed_z -= 1;
            }
            count += 1;
        }
        let mut sea_pickle_prop = SeaPickleProperties::default(block);
        sea_pickle_prop.pickles = Integer1To4::L4;
        world
            .set_block_state(
                &location,
                sea_pickle_prop.to_state_id(block),
                BlockFlags::NOTIFY_LISTENERS,
            )
            .await;

        BlockActionResult::Consume
    }

    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        player: &Player,
        block: &Block,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        replacing: BlockIsReplacing,
        _use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        if player.get_entity().pose.load() != EntityPose::Crouching {
            if let BlockIsReplacing::Itself(state_id) = replacing {
                let mut sea_pickle_prop = SeaPickleProperties::from_state_id(state_id, block);
                if sea_pickle_prop.pickles != Integer1To4::L4 {
                    sea_pickle_prop.pickles = match sea_pickle_prop.pickles {
                        Integer1To4::L1 => Integer1To4::L2,
                        Integer1To4::L2 => Integer1To4::L3,
                        _ => Integer1To4::L4,
                    };
                }
                return sea_pickle_prop.to_state_id(block);
            }
        }

        let mut sea_pickle_prop = SeaPickleProperties::default(block);
        sea_pickle_prop.waterlogged = replacing.water_source();
        sea_pickle_prop.to_state_id(block)
    }

    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        _world: Option<&World>,
        block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        let support_block = block_accessor.get_block_state(&block_pos.down()).await;
        support_block.is_center_solid(BlockDirection::Up)
    }

    #[allow(clippy::too_many_arguments)]
    async fn can_update_at(
        &self,
        _world: &World,
        block: &Block,
        state_id: BlockStateId,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: &SUseItemOn,
        player: &Player,
    ) -> bool {
        player.get_entity().pose.load() != EntityPose::Crouching
            && SeaPickleProperties::from_state_id(state_id, block).pickles != Integer1To4::L4
    }
}
