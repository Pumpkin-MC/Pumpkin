use std::sync::Arc;

use pumpkin_data::block_properties::{
    BlockProperties, CopperGolemPose, CopperGolemStatueLikeProperties,
};
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockId, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

use crate::block::blocks::copper_weathering;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockFuture, BlockIsReplacing, BlockMetadata, GetComparatorOutputArgs,
    NormalUseArgs, OnPlaceArgs, RandomTickArgs, UseWithItemArgs,
};
use crate::entity::EntityBase;
use crate::world::World;
use crate::world::game_event::{GameEventContext, emit_game_event};

/// `net.minecraft.world.level.block.CopperGolemStatueBlock` and its weathering subclass
/// `WeatheringCopperGolemStatueBlock`.
pub struct CopperGolemStatueBlock;

/// The four oxidation stages, mirroring `WeatheringCopper.NEXT_BY_BLOCK`. The waxed
/// variants share every behaviour except random-tick weathering.
const OXIDATION_FAMILY: [&Block; 4] = [
    &Block::COPPER_GOLEM_STATUE,
    &Block::EXPOSED_COPPER_GOLEM_STATUE,
    &Block::WEATHERED_COPPER_GOLEM_STATUE,
    &Block::OXIDIZED_COPPER_GOLEM_STATUE,
];

impl BlockMetadata for CopperGolemStatueBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::COPPER_GOLEM_STATUE,
            BlockId::EXPOSED_COPPER_GOLEM_STATUE,
            BlockId::WEATHERED_COPPER_GOLEM_STATUE,
            BlockId::OXIDIZED_COPPER_GOLEM_STATUE,
            BlockId::WAXED_COPPER_GOLEM_STATUE,
            BlockId::WAXED_EXPOSED_COPPER_GOLEM_STATUE,
            BlockId::WAXED_WEATHERED_COPPER_GOLEM_STATUE,
            BlockId::WAXED_OXIDIZED_COPPER_GOLEM_STATUE,
        ]
        .into()
    }
}

impl CopperGolemStatueBlock {
    /// `CopperGolemStatueBlock.updatePose`: cycles the pose, plays the become-statue
    /// sound and emits a `block_change` game event attributed to the player.
    async fn update_pose(
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        position: &BlockPos,
        player: Option<&Arc<crate::entity::player::Player>>,
    ) {
        world.play_block_sound(
            Sound::EntityCopperGolemBecomeStatue,
            SoundCategory::Blocks,
            *position,
        );

        let mut props = CopperGolemStatueLikeProperties::from_state_id(state_id, block);
        props.copper_golem_pose = next_pose(props.copper_golem_pose);
        world
            .set_block_state(position, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
            .await;

        let context = player.map_or_else(GameEventContext::none, |player| {
            GameEventContext::of_entity(player.clone() as Arc<dyn EntityBase>)
        });
        emit_game_event(
            world,
            GameEvent::BlockChange,
            Vector3::new(
                f64::from(position.0.x) + 0.5,
                f64::from(position.0.y) + 0.5,
                f64::from(position.0.z) + 0.5,
            ),
            context,
        )
        .await;
    }
}

/// `CopperGolemStatueBlock.Pose.getNextPose`: `BY_ID` is built with
/// `OutOfBoundsStrategy.ZERO`, so STAR wraps back to STANDING.
const fn next_pose(pose: CopperGolemPose) -> CopperGolemPose {
    match pose {
        CopperGolemPose::Standing => CopperGolemPose::Sitting,
        CopperGolemPose::Sitting => CopperGolemPose::Running,
        CopperGolemPose::Running => CopperGolemPose::Star,
        CopperGolemPose::Star => CopperGolemPose::Standing,
    }
}

impl BlockBehaviour for CopperGolemStatueBlock {
    /// `getStateForPlacement`: faces away from the placing player, waterlogged when it
    /// replaced a water source. POSE always starts at STANDING (the default state).
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = CopperGolemStatueLikeProperties::default(args.block);
            props.facing = args.player.get_entity().get_horizontal_facing();
            props.waterlogged = matches!(args.replacing, BlockIsReplacing::Water(_));
            props.to_state_id(args.block)
        })
    }

    /// `useItemOn` with an empty hand: cycles the pose.
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            Self::update_pose(
                args.world,
                args.block,
                state.id,
                args.position,
                Some(args.player),
            )
            .await;
            BlockActionResult::Success
        })
    }

    /// `WeatheringCopperGolemStatueBlock.useItemOn`: an axe is passed through (it either
    /// de-waxes, scrapes oxidation, or -- on an UNAFFECTED statue -- releases the golem,
    /// all handled by the axe item), honeycomb is passed through to the waxing path, and
    /// anything else cycles the pose.
    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let item: &Item = args.item_stack.lock().await.item;
            if item.has_tag(&tag::Item::MINECRAFT_AXES) || item.id == Item::HONEYCOMB.id {
                return BlockActionResult::PassToDefaultBlockAction;
            }

            let state = args.world.get_block_state(args.position);
            Self::update_pose(
                args.world,
                args.block,
                state.id,
                args.position,
                Some(args.player),
            )
            .await;
            BlockActionResult::Success
        })
    }

    /// `getAnalogOutputSignal`: `POSE.ordinal() + 1`, i.e. 1..=4.
    fn get_comparator_output<'a>(
        &'a self,
        args: GetComparatorOutputArgs<'a>,
    ) -> BlockFuture<'a, Option<u8>> {
        Box::pin(async move {
            let props = CopperGolemStatueLikeProperties::from_state_id(args.state.id, args.block);
            let ordinal = match props.copper_golem_pose {
                CopperGolemPose::Standing => 0,
                CopperGolemPose::Sitting => 1,
                CopperGolemPose::Running => 2,
                CopperGolemPose::Star => 3,
            };
            Some(ordinal + 1)
        })
    }

    /// `WeatheringCopperGolemStatueBlock.randomTick`: only the unwaxed variants weather,
    /// and FACING/POSE/WATERLOGGED carry over to the next oxidation stage.
    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            let props = CopperGolemStatueLikeProperties::from_state_id(state.id, args.block);
            copper_weathering::try_oxidize_copper(
                args.world,
                args.position,
                args.block,
                &OXIDATION_FAMILY,
                |next_block| props.to_state_id(next_block),
            )
            .await;
        })
    }
}
