use async_trait::async_trait;
use pumpkin_data::block_properties::{BlockProperties, HorizontalFacing, Integer1To4};
use pumpkin_world::BlockStateId;

use crate::block::pumpkin_block::{CanUpdateAtArgs, PumpkinBlock};
use crate::block::{BlockIsReplacing, pumpkin_block::OnPlaceArgs};

pub trait SegmentProperties {
    fn get_segment_amount(&self) -> Integer1To4;
    fn set_segment_amount(&mut self, amount: Integer1To4);
    fn get_facing(&self) -> HorizontalFacing;
    fn set_facing(&mut self, facing: HorizontalFacing);
}

macro_rules! impl_segment_properties {
    ($type:ty, $amount_field:ident) => {
        impl SegmentProperties for $type {
            fn get_segment_amount(&self) -> Integer1To4 {
                self.$amount_field
            }

            fn set_segment_amount(&mut self, amount: Integer1To4) {
                self.$amount_field = amount;
            }

            fn get_facing(&self) -> HorizontalFacing {
                self.facing
            }

            fn set_facing(&mut self, facing: HorizontalFacing) {
                self.facing = facing;
            }
        }
    };
}

impl_segment_properties!(
    pumpkin_data::block_properties::PinkPetalsLikeProperties,
    flower_amount
);
impl_segment_properties!(
    pumpkin_data::block_properties::LeafLitterLikeProperties,
    segment_amount
);

#[async_trait]
pub trait Segmented: PumpkinBlock {
    type Properties: BlockProperties + SegmentProperties;

    fn can_add_segment(&self, props: &Self::Properties) -> bool {
        let amount = props.get_segment_amount();
        matches!(amount, Integer1To4::L1 | Integer1To4::L2 | Integer1To4::L3)
    }

    fn get_next_segment_amount(&self, current: Integer1To4) -> Integer1To4 {
        match current {
            Integer1To4::L1 => Integer1To4::L2,
            Integer1To4::L2 => Integer1To4::L3,
            Integer1To4::L3 | Integer1To4::L4 => Integer1To4::L4,
        }
    }

    fn get_facing_for_segment(
        &self,
        player_facing: HorizontalFacing,
        segment_amount: Integer1To4,
    ) -> HorizontalFacing {
        let base_facing = match segment_amount {
            Integer1To4::L1 => HorizontalFacing::South,
            Integer1To4::L2 => HorizontalFacing::East,
            Integer1To4::L3 => HorizontalFacing::North,
            Integer1To4::L4 => HorizontalFacing::West,
        };

        match player_facing {
            HorizontalFacing::North => base_facing,
            HorizontalFacing::East => base_facing.rotate_clockwise(),
            HorizontalFacing::South => base_facing.rotate_clockwise().rotate_clockwise(),
            HorizontalFacing::West => base_facing.rotate_counter_clockwise(),
        }
    }

    async fn can_update_at(&self, ctx: CanUpdateAtArgs<'_>) -> bool {
        let current_props = Self::Properties::from_state_id(ctx.state_id, ctx.block);
        self.can_add_segment(&current_props)
    }

    async fn on_place(&self, ctx: OnPlaceArgs<'_>) -> BlockStateId {
        if let BlockIsReplacing::Itself(existing_state_id) = ctx.replacing {
            let mut props = Self::Properties::from_state_id(existing_state_id, ctx.block);

            if self.can_add_segment(&props) {
                let current_amount = props.get_segment_amount();
                let next_amount = self.get_next_segment_amount(current_amount);
                props.set_segment_amount(next_amount);
                props.to_state_id(ctx.block)
            } else {
                existing_state_id
            }
        } else {
            // Set first segment orientation based on player direction
            let player_facing = ctx.player.living_entity.entity.get_horizontal_facing();
            let mut props = Self::Properties::default(ctx.block);
            props.set_segment_amount(Integer1To4::L1);
            props.set_facing(self.get_facing_for_segment(player_facing, Integer1To4::L1));
            props.to_state_id(ctx.block)
        }
    }
}
