use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, EmitsRedstonePowerArgs, GetComparatorOutputArgs, NormalUseArgs, OnPlaceArgs,
};
use crate::entity::EntityBase;
use async_trait::async_trait;
use pumpkin_data::block_properties::{
    BlockProperties, CopperGolemPose, CopperGolemStatueLikeProperties,
};
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;

#[pumpkin_block_from_tag("minecraft:copper_golem_statues")]
pub struct GolemStatueBlock;

impl GolemStatueBlock {}

#[async_trait]
impl BlockBehaviour for GolemStatueBlock {
    async fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state = args.world.get_block_state(args.position).await;
        let mut golem_props = CopperGolemStatueLikeProperties::from_state_id(state.id, args.block);
        golem_props.copper_golem_pose = get_next_pose(golem_props.copper_golem_pose);
        args.world
            .set_block_state(
                args.position,
                golem_props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            )
            .await;
        args.world
            .play_block_sound(
                Sound::EntityCopperGolemBecomeStatue,
                SoundCategory::Blocks,
                *args.position,
            )
            .await;
        BlockActionResult::Pass
    }

    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut golem_props = CopperGolemStatueLikeProperties::default(args.block);
        golem_props.facing = args.player.get_entity().get_horizontal_facing().opposite();
        golem_props.waterlogged = args.replacing.water_source();
        golem_props.copper_golem_pose = CopperGolemPose::Standing;
        golem_props.to_state_id(args.block)
    }

    async fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }

    async fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        let state = args.world.get_block_state(args.position).await;
        let golem_props = CopperGolemStatueLikeProperties::from_state_id(state.id, args.block);
        match golem_props.copper_golem_pose {
            CopperGolemPose::Standing => Some(1),
            CopperGolemPose::Sitting => Some(2),
            CopperGolemPose::Running => Some(3),
            CopperGolemPose::Star => Some(4),
        }
    }
}

fn get_next_pose(pose: CopperGolemPose) -> CopperGolemPose {
    if CopperGolemPose::Standing == pose {
        return CopperGolemPose::Sitting;
    }
    if CopperGolemPose::Sitting == pose {
        return CopperGolemPose::Running;
    }
    if CopperGolemPose::Running == pose {
        return CopperGolemPose::Star;
    }
    if CopperGolemPose::Star == pose {
        return CopperGolemPose::Standing;
    }
    CopperGolemPose::Standing
}

fn is_waxed(name: &str) -> bool {
    name.contains("WAXED")
}
