use async_trait::async_trait;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::SlabType;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_world::BlockStateId;

use crate::block::BlockIsReplacing;
use crate::block::pumpkin_block::CanUpdateAtArgs;
use crate::block::pumpkin_block::OnPlaceArgs;
use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};

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
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if let BlockIsReplacing::Itself(state_id) = args.replacing {
            let mut slab_props = SlabProperties::from_state_id(state_id, args.block);
            slab_props.r#type = SlabType::Double;
            slab_props.waterlogged = false;
            return slab_props.to_state_id(args.block);
        }

        let mut slab_props = SlabProperties::default(args.block);
        slab_props.waterlogged = args.replacing.water_source();
        slab_props.r#type = match args.direction {
            BlockDirection::Up => SlabType::Top,
            BlockDirection::Down => SlabType::Bottom,
            _ => match args.use_item_on.cursor_pos.y {
                0.0...0.5 => SlabType::Bottom,
                _ => SlabType::Top,
            },
        };

        slab_props.to_state_id(args.block)
    }

    async fn can_update_at(&self, args: CanUpdateAtArgs<'_>) -> bool {
        let slab_props = SlabProperties::from_state_id(args.state_id, args.block);

        slab_props.r#type
            == match args.direction {
                BlockDirection::Up => SlabType::Bottom,
                BlockDirection::Down => SlabType::Top,
                _ => match args.use_item_on.cursor_pos.y {
                    0.0...0.5 => SlabType::Top,
                    _ => SlabType::Bottom,
                },
            }
    }
}
