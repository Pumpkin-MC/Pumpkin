use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, EmitsRedstonePowerArgs, GetComparatorOutputArgs, NormalUseArgs, OnPlaceArgs,
};
use async_trait::async_trait;
use pumpkin_data::block_properties::{
    BlockProperties, CopperGolemPose, CopperGolemStatueLikeProperties,
};
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::BlockStateId;

#[pumpkin_block_from_tag("minecraft:copper_golem_statues")]
pub struct GolemStatueBlock;

impl GolemStatueBlock {}

#[async_trait]
impl BlockBehaviour for GolemStatueBlock {
    async fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        todo!()
    }

    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut golem_props = CopperGolemStatueLikeProperties::default(args.block);
        if let Some(facing) = args.direction.to_horizontal_facing() {
            golem_props.facing = facing;
        }
        golem_props.waterlogged = args.replacing.water_source();
        golem_props.copper_golem_pose = CopperGolemPose::Standing;
        golem_props.to_state_id(args.block)
    }

    async fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }

    async fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        todo!()
    }
}
