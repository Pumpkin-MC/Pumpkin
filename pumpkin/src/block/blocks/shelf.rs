use crate::block::blocks::redstone::{block_receives_redstone_power, get_redstone_power};
use crate::block::{BlockBehaviour, GetStateForNeighborUpdateArgs, NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs, UseWithItemArgs};
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::Block;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::SideChain;
use pumpkin_data::block_properties::{AcaciaShelfLikeProperties, HorizontalFacing};
use pumpkin_data::tag::RegistryKey;
use pumpkin_data::tag::get_tag_values;
use pumpkin_macros::pumpkin_block_from_tag;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::world::BlockFlags;
use crate::block::registry::BlockActionResult;

#[pumpkin_block_from_tag("minecraft:wooden_shelves")]
pub struct Shelf;

#[async_trait]
impl BlockBehaviour for Shelf {
    async fn normal_use(&self, _args: NormalUseArgs<'_>) -> BlockActionResult {
        log::warn!("Shelf normal_use() called");
        BlockActionResult::Pass
    }

    async fn use_with_item(&self, _args: UseWithItemArgs<'_>) -> BlockActionResult {
        // TODO: Here switch the items in the hotbar
        log::warn!("use_with_item: {}", _args.item_stack.lock().await.item.registry_key);
        for item in  &_args.player.inventory.main_inventory{
            log::warn!("use_with_item: items {}",item.lock().await.item.registry_key);
        }
        BlockActionResult::Pass
    }


    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = AcaciaShelfLikeProperties::default(args.block);
        props.waterlogged = args.replacing.water_source();
        props.powered = block_receives_redstone_power(args.world, args.position).await;
        props.side_chain = SideChain::Unconnected;
        props.facing = args
            .player
            .living_entity
            .entity
            .get_horizontal_facing()
            .opposite();
        props.to_state_id(args.block)
    }

    async fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let mut own_state = AcaciaShelfLikeProperties::from_state_id(
            args.world.get_block_state(args.position).await.id,
            args.block,
        );
        own_state.powered = block_receives_redstone_power(args.world, args.position).await;
        args.world
            .set_block_state(
                args.position,
                own_state.to_state_id(args.block),
                BlockFlags::NOTIFY_LISTENERS,
            )
            .await;
        let left = is_left_shelf(args.position, args.block, args.world, own_state.facing).await;
        let right = is_right_shelf(args.position, args.block, args.world, own_state.facing).await;
        if left == None && right == None {
            own_state.side_chain = SideChain::Unconnected;
            args.world
                .set_block_state(
                    args.position,
                    own_state.to_state_id(args.block),
                    BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
        } else if left != None && right == None {
            match left.unwrap() {
                SideChain::Left | SideChain::Unconnected | SideChain::Center => {
                    own_state.side_chain = SideChain::Right;
                    args.world
                        .set_block_state(
                            args.position,
                            own_state.to_state_id(args.block),
                            BlockFlags::NOTIFY_LISTENERS,
                        )
                        .await;
                }
                SideChain::Right => {
                    own_state.side_chain = SideChain::Unconnected;
                    args.world
                        .set_block_state(
                            args.position,
                            own_state.to_state_id(args.block),
                            BlockFlags::NOTIFY_LISTENERS,
                        )
                        .await;
                }
            }
        } else if left == None && right != None {
            match right.unwrap() {
                SideChain::Left => {
                    own_state.side_chain = SideChain::Unconnected;
                    args.world
                        .set_block_state(
                            args.position,
                            own_state.to_state_id(args.block),
                            BlockFlags::NOTIFY_LISTENERS,
                        )
                        .await;
                }
                SideChain::Right | SideChain::Unconnected | SideChain::Center => {
                    own_state.side_chain = SideChain::Left;
                    args.world
                        .set_block_state(
                            args.position,
                            own_state.to_state_id(args.block),
                            BlockFlags::NOTIFY_LISTENERS,
                        )
                        .await;
                }
            }
        } else if left != None && right != None {
            match left.unwrap() {
                SideChain::Unconnected => match right.unwrap() {
                    SideChain::Unconnected => {
                        own_state.side_chain = SideChain::Center;
                        args.world
                            .set_block_state(
                                args.position,
                                own_state.to_state_id(args.block),
                                BlockFlags::NOTIFY_LISTENERS,
                            )
                            .await;
                    }
                    SideChain::Center => {
                        own_state.side_chain = SideChain::Right;
                        args.world
                            .set_block_state(
                                args.position,
                                own_state.to_state_id(args.block),
                                BlockFlags::NOTIFY_LISTENERS,
                            )
                            .await;
                    }
                    _ => {
                    }
                },
                SideChain::Left => match right.unwrap() {
                    SideChain::Right | SideChain::Unconnected => {
                        own_state.side_chain = SideChain::Center;
                        args.world
                            .set_block_state(
                                args.position,
                                own_state.to_state_id(args.block),
                                BlockFlags::NOTIFY_LISTENERS,
                            )
                            .await;
                    }
                    SideChain::Center => {
                        own_state.side_chain = SideChain::Right;
                        args.world
                            .set_block_state(
                                args.position,
                                own_state.to_state_id(args.block),
                                BlockFlags::NOTIFY_LISTENERS,
                            )
                            .await;
                    }
                    _ => {
                    }
                },
                SideChain::Right => {
                    own_state.side_chain = SideChain::Left;
                    args.world
                        .set_block_state(
                            args.position,
                            own_state.to_state_id(args.block),
                            BlockFlags::NOTIFY_LISTENERS,
                        )
                        .await;
                }
                SideChain::Center => {
                    own_state.side_chain = SideChain::Right;
                    args.world
                        .set_block_state(
                            args.position,
                            own_state.to_state_id(args.block),
                            BlockFlags::NOTIFY_LISTENERS,
                        )
                        .await;
                }
            }
        }

    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        log::warn!(
            "Notify with block id: {} for neighbor {}",
            args.block.id,
            Block::from_state_id(args.neighbor_state_id).id
        );
        let state = args.world.get_block_state(args.position).await;
        let mut props = AcaciaShelfLikeProperties::from_state_id(state.id, args.block);
        props.powered = block_receives_redstone_power(args.world, args.position).await;
        props.to_state_id(args.block)
    }
}
async fn is_left_shelf(
    cur_block_pos: &BlockPos,
    cur_block: &Block,
    world: &World,
    facing: HorizontalFacing,
) -> Option<SideChain> {
    match facing {
        HorizontalFacing::South => {
            let new_pos = &cur_block_pos.west();
            let block = world.get_block(new_pos).await;
            if block.id == cur_block.id {
                let state = AcaciaShelfLikeProperties::from_state_id(
                    world.get_block_state(new_pos).await.id,
                    cur_block,
                );
                if block.id == cur_block.id && state.facing == facing && state.powered {
                    return Some(state.side_chain);
                }
            }
            None
        }
        _ => None,
    }
}

async fn is_right_shelf(
    cur_block_pos: &BlockPos,
    cur_block: &Block,
    world: &World,
    facing: HorizontalFacing,
) -> Option<SideChain> {
    match facing {
        HorizontalFacing::South => {
            let new_pos = &cur_block_pos.east();
            let block = world.get_block(new_pos).await;
            if block.id == cur_block.id {
                let state = AcaciaShelfLikeProperties::from_state_id(
                    world.get_block_state(new_pos).await.id,
                    block,
                );
                if block.id == cur_block.id && state.facing == facing && state.powered {
                    return Some(state.side_chain);
                }
            }
            None
        }
        _ => None,
    }
}
