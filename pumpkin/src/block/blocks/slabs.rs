use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::SlabType;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::BlockStateId;

use crate::block::BlockBehaviour;
use crate::block::BlockFuture;
use crate::block::BlockIsReplacing;
use crate::block::CanUpdateAtArgs;
use crate::block::OnPlaceArgs;

type SlabProperties = pumpkin_data::block_properties::ResinBrickSlabLikeProperties;

#[pumpkin_block_from_tag("minecraft:slabs")]
pub struct SlabBlock;

impl BlockBehaviour for SlabBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
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
                    0.0..0.5 => SlabType::Bottom,
                    _ => SlabType::Top,
                },
            };

            slab_props.to_state_id(args.block)
        })
    }

    fn can_update_at<'a>(&'a self, args: CanUpdateAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let slab_props = SlabProperties::from_state_id(args.state_id, args.block);

            slab_props.r#type
                == match args.direction {
                    BlockDirection::Up => SlabType::Bottom,
                    BlockDirection::Down => SlabType::Top,
                    _ => match args.use_item_on.cursor_pos.y {
                        0.0..0.5 => SlabType::Top,
                        _ => SlabType::Bottom,
                    },
                }
        })
    }
}
