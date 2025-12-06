use crate::block::blocks::plant::PlantBlockBase;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, CanPlaceAtArgs, CanUpdateAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    UseWithItemArgs,
};
use crate::block::{BlockFuture, BlockIsReplacing};
use crate::entity::EntityBase;
use pumpkin_data::block_properties::{BlockProperties, Integer1To4};
use pumpkin_data::entity::EntityPose;
use pumpkin_data::item::Item;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;
use rand::Rng;

type SeaPickleProperties = pumpkin_data::block_properties::SeaPickleLikeProperties;

#[pumpkin_block("minecraft:sea_pickle")]
pub struct SeaPickleBlock;

impl BlockBehaviour for SeaPickleBlock {
    #[allow(clippy::many_single_char_names)]
    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            if args.item_stack.lock().await.item != &Item::BONE_MEAL
                || !args
                    .world
                    .get_block(&args.position.down())
                    .await
                    .has_tag(&tag::Block::MINECRAFT_CORAL_BLOCKS)
                || !SeaPickleProperties::from_state_id(
                    args.world.get_block_state_id(args.position).await,
                    args.block,
                )
                .waterlogged
            {
                return BlockActionResult::Pass;
            }

            //1:1 vanilla algorithm
            //TODO use pumpkin random

            //let mut j = 1;
            let mut count = 0;
            let base_x = args.position.0.x - 2;
            let mut removed_z = 0;
            for added_x in 0..5 {
                for added_z in 0..1 {
                    let temp_y = 2 + args.position.0.y - 1;
                    for y in (temp_y - 2)..temp_y {
                        //let mut lv2: BlockState;
                        let lv = BlockPos::new(
                            base_x + added_x,
                            y,
                            args.position.0.z - removed_z + added_z,
                        );
                        if &lv == args.position
                            || rand::rng().random_range(0..6) != 0
                            || !args.world.get_block(&lv).await.eq(&Block::WATER)
                            || !args
                                .world
                                .get_block(&lv.down())
                                .await
                                .has_tag(&tag::Block::MINECRAFT_CORAL_BLOCKS)
                        {
                            continue;
                        }
                        let mut sea_pickle_prop = SeaPickleProperties::default(args.block);

                        sea_pickle_prop.pickles = match rand::rng().random_range(0..4) + 1 {
                            1 => Integer1To4::L1,
                            2 => Integer1To4::L2,
                            3 => Integer1To4::L3,
                            _ => Integer1To4::L4,
                        };
                        args.world
                            .set_block_state(
                                &lv,
                                sea_pickle_prop.to_state_id(args.block),
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
            let mut sea_pickle_prop = SeaPickleProperties::default(args.block);
            sea_pickle_prop.pickles = Integer1To4::L4;
            args.world
                .set_block_state(
                    args.position,
                    sea_pickle_prop.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;

            BlockActionResult::Consume
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if args.player.get_entity().pose.load() != EntityPose::Crouching
                && let BlockIsReplacing::Itself(state_id) = args.replacing
            {
                let mut sea_pickle_prop = SeaPickleProperties::from_state_id(state_id, args.block);
                if sea_pickle_prop.pickles != Integer1To4::L4 {
                    sea_pickle_prop.pickles = match sea_pickle_prop.pickles {
                        Integer1To4::L1 => Integer1To4::L2,
                        Integer1To4::L2 => Integer1To4::L3,
                        _ => Integer1To4::L4,
                    };
                }
                return sea_pickle_prop.to_state_id(args.block);
            }

            let mut sea_pickle_prop = SeaPickleProperties::default(args.block);
            sea_pickle_prop.waterlogged = args.replacing.water_source();
            sea_pickle_prop.to_state_id(args.block)
        })
    }

    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            let support_block = args
                .block_accessor
                .get_block_state(&args.position.down())
                .await;
            support_block.is_center_solid(BlockDirection::Up)
        })
    }

    fn can_update_at<'a>(&'a self, args: CanUpdateAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            args.player.get_entity().pose.load() != EntityPose::Crouching
                && SeaPickleProperties::from_state_id(args.state_id, args.block).pickles
                    != Integer1To4::L4
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            <Self as PlantBlockBase>::get_state_for_neighbor_update(
                self,
                args.world,
                args.position,
                args.state_id,
            )
            .await
        })
    }
}

impl PlantBlockBase for SeaPickleBlock {}
