use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::block::Block;
use pumpkin_data::block::BlockState;
use pumpkin_data::block::HorizontalFacing;
use pumpkin_data::block::{BlockProperties, Boolean};
use pumpkin_protocol::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::BlockDirection;
use pumpkin_world::block::HorizontalFacingExt;
use pumpkin_world::chunk::TickPriority;

type RWallTorchProps = pumpkin_data::block::FurnaceLikeProperties;
type RTorchProps = pumpkin_data::block::RedstoneOreLikeProperties;

use crate::block::pumpkin_block::{BlockMetadata, PumpkinBlock};
use crate::block::registry::BlockRegistry;
use crate::server::Server;
use crate::world::BlockFlags;
use crate::world::World;

use super::get_redstone_power;

#[allow(clippy::too_many_lines)]
pub fn register_redstone_torch_blocks(manager: &mut BlockRegistry) {
    for block in ["redstone_torch", "redstone_wall_torch"] {
        pub struct TorchBlock {
            id: &'static str,
        }
        impl BlockMetadata for TorchBlock {
            fn namespace(&self) -> &'static str {
                "minecraft"
            }

            fn id(&self) -> &'static str {
                self.id
            }
        }

        #[async_trait]
        impl PumpkinBlock for TorchBlock {
            async fn on_place(
                &self,
                _server: &Server,
                world: &World,
                _block: &Block,
                face: &BlockDirection,
                block_pos: &BlockPos,
                _use_item_on: &SUseItemOn,
                _player_direction: &HorizontalFacing,
                _other: bool,
            ) -> BlockStateId {
                if face.is_horizontal() {
                    let mut torch_props = RWallTorchProps::default(&Block::REDSTONE_WALL_TORCH);
                    torch_props.facing = face.to_horizontal_facing().unwrap().opposite();
                    torch_props.lit =
                        Boolean::from_bool(should_be_lit(world, block_pos, face).await);
                    return torch_props.to_state_id(&Block::REDSTONE_WALL_TORCH);
                }
                let mut torch_props = RTorchProps::default(&Block::REDSTONE_TORCH);
                torch_props.lit = Boolean::from_bool(
                    should_be_lit(world, block_pos, &BlockDirection::Down).await,
                );
                return torch_props.to_state_id(&Block::REDSTONE_TORCH);
            }

            async fn on_neighbor_update(
                &self,
                world: &Arc<World>,
                block: &Block,
                block_pos: &BlockPos,
                _source_block: &Block,
                _notify: bool,
            ) {
                let state = world.get_block_state(block_pos).await.unwrap();

                if world.is_block_tick_scheduled(block_pos, block).await {
                    return;
                }

                if block == &Block::REDSTONE_WALL_TORCH {
                    let props = RWallTorchProps::from_state_id(state.id, block);
                    if props.lit.to_bool()
                        != should_be_lit(
                            world,
                            block_pos,
                            &props.facing.to_block_direction().opposite(),
                        )
                        .await
                    {
                        world
                            .schedule_block_tick(block, *block_pos, 2, TickPriority::Normal)
                            .await;
                    }
                } else if block == &Block::REDSTONE_TORCH {
                    let props = RTorchProps::from_state_id(state.id, block);
                    if props.lit.to_bool()
                        != should_be_lit(world, block_pos, &BlockDirection::Down).await
                    {
                        world
                            .schedule_block_tick(block, *block_pos, 2, TickPriority::Normal)
                            .await;
                    }
                }
            }

            async fn emits_redstone_power(
                &self,
                _block: &Block,
                _state: &BlockState,
                _direction: &BlockDirection,
            ) -> bool {
                true
            }

            async fn get_weak_redstone_power(
                &self,
                block: &Block,
                _world: &World,
                _block_pos: &BlockPos,
                state: &BlockState,
                direction: &BlockDirection,
            ) -> u8 {
                if block == &Block::REDSTONE_WALL_TORCH {
                    let props = RWallTorchProps::from_state_id(state.id, block);
                    if props.lit.to_bool() && direction != &props.facing.to_block_direction() {
                        return 15;
                    }
                } else if block == &Block::REDSTONE_TORCH {
                    let props = RTorchProps::from_state_id(state.id, block);
                    if props.lit.to_bool() && direction != &BlockDirection::Up {
                        return 15;
                    }
                }
                0
            }

            async fn get_strong_redstone_power(
                &self,
                block: &Block,
                _world: &World,
                _block_pos: &BlockPos,
                state: &BlockState,
                direction: &BlockDirection,
            ) -> u8 {
                if direction == &BlockDirection::Down {
                    if block == &Block::REDSTONE_WALL_TORCH {
                        let props = RWallTorchProps::from_state_id(state.id, block);
                        if props.lit.to_bool() {
                            return 15;
                        }
                    } else if block == &Block::REDSTONE_TORCH {
                        let props = RTorchProps::from_state_id(state.id, block);
                        if props.lit.to_bool() {
                            return 15;
                        }
                    }
                }
                0
            }

            async fn on_scheduled_tick(
                &self,
                world: &Arc<World>,
                block: &Block,
                block_pos: &BlockPos,
            ) {
                let state = world.get_block_state(block_pos).await.unwrap();
                if block == &Block::REDSTONE_WALL_TORCH {
                    let mut props = RWallTorchProps::from_state_id(state.id, block);
                    let should_be_lit_now = should_be_lit(
                        world,
                        block_pos,
                        &props.facing.to_block_direction().opposite(),
                    )
                    .await;
                    if props.lit.to_bool() != should_be_lit_now {
                        props.lit = Boolean::from_bool(should_be_lit_now);
                        world
                            .set_block_state(
                                block_pos,
                                props.to_state_id(block),
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await;
                        update_neighbors(world, block_pos).await;
                    }
                } else if block == &Block::REDSTONE_TORCH {
                    let mut props = RTorchProps::from_state_id(state.id, block);
                    let should_be_lit_now =
                        should_be_lit(world, block_pos, &BlockDirection::Down).await;
                    if props.lit.to_bool() != should_be_lit_now {
                        props.lit = Boolean::from_bool(should_be_lit_now);
                        world
                            .set_block_state(
                                block_pos,
                                props.to_state_id(block),
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await;
                        update_neighbors(world, block_pos).await;
                    }
                }
            }

            async fn placed(
                &self,
                world: &Arc<World>,
                _block: &Block,
                _state_id: BlockStateId,
                block_pos: &BlockPos,
                _old_state_id: BlockStateId,
                _notify: bool,
            ) {
                update_neighbors(world, block_pos).await;
            }

            async fn on_state_replaced(
                &self,
                world: &Arc<World>,
                _block: &Block,
                location: BlockPos,
                _old_state_id: BlockStateId,
                _moved: bool,
            ) {
                update_neighbors(world, &location).await;
            }
        }

        manager.register(TorchBlock { id: block });
    }
}

pub async fn should_be_lit(world: &World, pos: &BlockPos, face: &BlockDirection) -> bool {
    let other_pos = pos.offset(face.to_offset());
    let (block, state) = world.get_block_and_block_state(&other_pos).await.unwrap();
    get_redstone_power(&block, &state, world, other_pos, face).await == 0
}

pub async fn update_neighbors(world: &Arc<World>, pos: &BlockPos) {
    for dir in BlockDirection::all() {
        let other_pos = pos.offset(dir.to_offset());
        world.update_neighbors(&other_pos, None).await;
    }
}
