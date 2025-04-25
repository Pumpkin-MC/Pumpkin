use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockHalf;
use pumpkin_data::block::BlockProperties;
use pumpkin_data::block::StairShape;
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::Tagable;
use pumpkin_data::tag::get_tag_values;
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::BlockDirection;
use pumpkin_world::block::HorizontalFacingExt;
use std::sync::Arc;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::world::BlockFlags;
use crate::world::World;
use crate::{entity::player::Player, server::Server};

type StairsProperties = pumpkin_data::block::OakStairsLikeProperties;

pub struct StairBlock;

impl BlockMetadata for StairBlock {
    fn namespace(&self) -> &'static str {
        "minecraft"
    }

    fn ids(&self) -> &'static [&'static str] {
        get_tag_values(RegistryKey::Block, "minecraft:stairs").unwrap()
    }
}

#[async_trait]
impl PumpkinBlock for StairBlock {
    async fn on_place(
        &self,
        _server: &Server,
        world: &World,
        block: &Block,
        face: &BlockDirection,
        block_pos: &BlockPos,
        use_item_on: &SUseItemOn,
        player: &Player,
        _other: bool,
    ) -> BlockStateId {
        let mut stair_props = StairsProperties::default(block);

        stair_props.facing = player.living_entity.entity.get_horizontal_facing();
        stair_props.half = match face {
            BlockDirection::Up => BlockHalf::Top,
            BlockDirection::Down => BlockHalf::Bottom,
            _ => match use_item_on.cursor_pos.y {
                0.0...0.5 => BlockHalf::Bottom,
                0.5...1.0 => BlockHalf::Top,
                _ => BlockHalf::Top,
            },
        };

        let (other_block, other_block_state) = world
            .get_block_and_block_state(&block_pos.offset(stair_props.facing.to_offset()))
            .await
            .unwrap();
        if other_block.is_tagged_with("#minecraft:stairs").unwrap() {
            let other_stair_props =
                StairsProperties::from_state_id(other_block_state.id, &other_block);

            if stair_props.half == other_stair_props.half {
                if stair_props.facing.rotate_clockwise() == other_stair_props.facing {
                    stair_props.shape = StairShape::OuterRight;
                    return stair_props.to_state_id(block);
                }

                if stair_props.facing.rotate_counter_clockwise() == other_stair_props.facing {
                    stair_props.shape = StairShape::OuterLeft;
                    return stair_props.to_state_id(block);
                }
            }
        }

        let (other_block, other_block_state) = world
            .get_block_and_block_state(&block_pos.offset(stair_props.facing.opposite().to_offset()))
            .await
            .unwrap();
        if other_block.is_tagged_with("#minecraft:stairs").unwrap() {
            let other_stair_props =
                StairsProperties::from_state_id(other_block_state.id, &other_block);

            if stair_props.half == other_stair_props.half {
                if stair_props.facing.rotate_clockwise() == other_stair_props.facing {
                    stair_props.shape = StairShape::InnerRight;
                    return stair_props.to_state_id(block);
                }

                if stair_props.facing.rotate_counter_clockwise() == other_stair_props.facing {
                    stair_props.shape = StairShape::InnerLeft;
                    return stair_props.to_state_id(block);
                }
            }
        }

        stair_props.to_state_id(block)
    }

    async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        block_pos: &BlockPos,
        _source_block: &Block,
        _notify: bool,
    ) {
        let state_id = world.get_block_state_id(block_pos).await.unwrap();
        let stair_props = StairsProperties::from_state_id(state_id, block);

        let (other_block, other_block_state) = world
            .get_block_and_block_state(
                &block_pos.offset(stair_props.facing.rotate_clockwise().to_offset()),
            )
            .await
            .unwrap();
        let right_locked = if other_block.is_tagged_with("#minecraft:stairs").unwrap() {
            let other_stair_props =
                StairsProperties::from_state_id(other_block_state.id, &other_block);

            stair_props.half == other_stair_props.half
                && stair_props.facing == other_stair_props.facing
        } else {
            false
        };

        let (other_block, other_block_state) = world
            .get_block_and_block_state(
                &block_pos.offset(stair_props.facing.rotate_counter_clockwise().to_offset()),
            )
            .await
            .unwrap();
        let left_locked = if other_block.is_tagged_with("#minecraft:stairs").unwrap() {
            let other_stair_props =
                StairsProperties::from_state_id(other_block_state.id, &other_block);

            stair_props.half == other_stair_props.half
                && stair_props.facing == other_stair_props.facing
        } else {
            false
        };

        if right_locked && left_locked {
            // Straight
        }

        let side = stair_props.facing.rotate_clockwise();
        let pos = block_pos.offset(side.to_offset());
        let (other_block, other_block_state) = world.get_block_and_block_state(&pos).await.unwrap();
        if other_block.is_tagged_with("#minecraft:stairs").unwrap() {
            let mut other_stair_props =
                StairsProperties::from_state_id(other_block_state.id, &other_block);

            if stair_props.half == other_stair_props.half {
                if side == other_stair_props.facing {
                    other_stair_props.shape = StairShape::InnerLeft;
                    world
                        .set_block_state(
                            &pos,
                            other_stair_props.to_state_id(&other_block),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                } else if side.opposite() == other_stair_props.facing {
                    other_stair_props.shape = StairShape::OuterRight;
                    world
                        .set_block_state(
                            &pos,
                            other_stair_props.to_state_id(&other_block),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                }
            }
        }

        let side = stair_props.facing.rotate_counter_clockwise();
        let pos = block_pos.offset(side.to_offset());
        let (other_block, other_block_state) = world.get_block_and_block_state(&pos).await.unwrap();
        if other_block.is_tagged_with("#minecraft:stairs").unwrap() {
            let mut other_stair_props =
                StairsProperties::from_state_id(other_block_state.id, &other_block);

            if stair_props.half == other_stair_props.half {
                if side == other_stair_props.facing {
                    other_stair_props.shape = StairShape::InnerRight;
                    world
                        .set_block_state(
                            &pos,
                            other_stair_props.to_state_id(&other_block),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                } else if side.opposite() == other_stair_props.facing {
                    other_stair_props.shape = StairShape::OuterLeft;
                    world
                        .set_block_state(
                            &pos,
                            other_stair_props.to_state_id(&other_block),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                }
            }
        }
    }
}
