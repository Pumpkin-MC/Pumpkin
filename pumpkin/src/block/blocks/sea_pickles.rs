use crate::block::BlockIsReplacing;
use crate::block::pumpkin_block::PumpkinBlock;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, Integer1To4};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockAccessor;

type SeaPickleProperties = pumpkin_data::block_properties::SeaPickleLikeProperties;

#[pumpkin_block("minecraft:sea_pickle")]
pub struct SeaPickleBlock;

#[async_trait]
impl PumpkinBlock for SeaPickleBlock {
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
        //TODO place around when player is crouching
        if player.get_entity().pose.load() != pumpkin_data::entity::EntityPose::Crouching {
            if let BlockIsReplacing::Itself(state_id) = replacing {
                let mut sea_pickle_prop = SeaPickleProperties::from_state_id(state_id, block);

                sea_pickle_prop.pickles = match sea_pickle_prop.pickles {
                    Integer1To4::L1 => Integer1To4::L2,
                    Integer1To4::L2 => Integer1To4::L3,
                    _ => Integer1To4::L4,
                };
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

    async fn can_update_at(
        &self,
        _world: &World,
        block: &Block,
        state_id: BlockStateId,
        _block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: &SUseItemOn,
    ) -> bool {
        let sea_pickle_prop = SeaPickleProperties::from_state_id(state_id, block);
        !sea_pickle_prop.pickles.eq(&Integer1To4::L4)
    }
}
