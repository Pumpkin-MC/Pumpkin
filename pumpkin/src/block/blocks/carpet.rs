use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::tag::{RegistryKey, get_tag_values};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::pumpkin_block;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockAccessor;

pub struct CarpetBlock;

impl BlockMetadata for CarpetBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:wool_carpets").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for CarpetBlock {
    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        world: Option<&World>,
        _block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        !world
            .unwrap()
            .get_block(&block_pos.down())
            .await
            .eq(&Block::AIR)
    }
}

#[pumpkin_block("minecraft:moss_carpet")]
pub struct MossCarpetBlock;

#[async_trait]
impl PumpkinBlock for MossCarpetBlock {
    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        world: Option<&World>,
        _block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        !world
            .unwrap()
            .get_block(&block_pos.down())
            .await
            .eq(&Block::AIR)
    }
}

#[pumpkin_block("minecraft:pale_moss_carpet")]
pub struct PaleMossCarpetBlock;

#[async_trait]
impl PumpkinBlock for PaleMossCarpetBlock {
    async fn can_place_at(
        &self,
        _server: Option<&Server>,
        world: Option<&World>,
        _block_accessor: &dyn BlockAccessor,
        _player: Option<&Player>,
        _block: &Block,
        block_pos: &BlockPos,
        _face: BlockDirection,
        _use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        !world
            .unwrap()
            .get_block(&block_pos.down())
            .await
            .eq(&Block::AIR)
    }
}
