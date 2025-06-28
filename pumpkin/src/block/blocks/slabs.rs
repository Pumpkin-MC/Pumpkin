use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::SlabType;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

use crate::block::BlockIsReplacing;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::world::World;
use crate::{entity::player::Player, server::Server};

type SlabProperties = pumpkin_data::block_properties::ResinBrickSlabLikeProperties;

pub struct SlabBlock;

impl BlockMetadata for SlabBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:slabs").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for SlabBlock {
    async fn on_place(
        &self,
        _server: &Server,
        _world: &World,
        _player: &Player,
        block: &Block,
        _block_pos: &BlockPos,
        face: BlockDirection,
        replacing: BlockIsReplacing,
        use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        if let BlockIsReplacing::Itself(state_id) = replacing {
            let mut slab_props = SlabProperties::from_state_id(state_id, block);
            slab_props.r#type = SlabType::Double;
            slab_props.waterlogged = false;
            return slab_props.to_state_id(block);
        }

        let mut slab_props = SlabProperties::default(block);
        slab_props.waterlogged = replacing.water_source();
        slab_props.r#type = match face {
            BlockDirection::Up => SlabType::Top,
            BlockDirection::Down => SlabType::Bottom,
            _ => match use_item_on.cursor_pos.y {
                0.0...0.5 => SlabType::Bottom,
                _ => SlabType::Top,
            },
        };

        slab_props.to_state_id(block)
    }

    #[allow(clippy::too_many_arguments)]
    async fn can_update_at(
        &self,
        _world: &World,
        block: &Block,
        state_id: BlockStateId,
        _block_pos: &BlockPos,
        face: BlockDirection,
        use_item_on: &SUseItemOn,
        _player: &Player,
    ) -> bool {
        let slab_props = SlabProperties::from_state_id(state_id, block);

        slab_props.r#type
            == match face {
                BlockDirection::Up => SlabType::Bottom,
                BlockDirection::Down => SlabType::Top,
                _ => match use_item_on.cursor_pos.y {
                    0.0...0.5 => SlabType::Top,
                    _ => SlabType::Bottom,
                },
            }
    }
}
