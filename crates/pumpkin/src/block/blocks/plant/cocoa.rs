use crate::block::{BlockBehaviour, BlockFuture, CanPlaceAtArgs, OnPlaceArgs, RandomTickArgs};
use pumpkin_data::block_properties::{BlockProperties, CocoaLikeProperties};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, HorizontalFacingExt};
use pumpkin_macros::pumpkin_block;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

const MAX_AGE: i32 = 2;

type CocoaBlockProperties = CocoaLikeProperties;

#[pumpkin_block("minecraft:cocoa")]
pub struct CocoaBlock;

impl BlockBehaviour for CocoaBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs) -> bool {
        let props = CocoaBlockProperties::from_state_id(args.state.id, args.block);

        let facing = match args.direction {
            Some(direction) => {
                let Some(facing) = direction.to_horizontal_facing() else {
                    return false;
                };
                facing
            }
            None => props.facing,
        };

        let offset = facing.to_offset();

        let block = args.block_accessor.get_block(&args.position.offset(offset));
        block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_COCOA)
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: crate::block::GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, pumpkin_data::BlockStateId> {
        Box::pin(async move {
            let props = CocoaBlockProperties::from_state_id(args.state_id, args.block);

            if props.facing.to_block_direction() != args.direction {
                return args.state_id;
            }

            let offset = props.facing.to_offset();

            let position = args.position.offset(offset);
            let support_block = args.world.get_block(&position);
            if !support_block.has_tag(&tag::Block::MINECRAFT_SUPPORTS_COCOA) {
                return Block::AIR.default_state.id;
            }

            args.state_id
        })
    }

    fn on_place<'a>(
        &'a self,
        args: OnPlaceArgs<'a>,
    ) -> BlockFuture<'a, pumpkin_data::BlockStateId> {
        Box::pin(async move {
            let mut props = CocoaBlockProperties::default(args.block);

            let Some(facing) = args.direction.to_horizontal_facing() else {
                return Block::AIR.default_state.id;
            };

            props.facing = facing;
            props.to_state_id(args.block)
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if rand::rng().random_range(0..5) == 0 {
                let state = args.world.get_block_state(args.position);
                let mut props = CocoaBlockProperties::from_state_id(state.id, args.block);
                let age = i32::from(props.age);
                if age < MAX_AGE {
                    props.age += 1;
                    let state = props.to_state_id(args.block);
                    args.world
                        .set_block_state(args.position, state, BlockFlags::NOTIFY_NEIGHBORS)
                        .await;
                }
            }
        })
    }
}
