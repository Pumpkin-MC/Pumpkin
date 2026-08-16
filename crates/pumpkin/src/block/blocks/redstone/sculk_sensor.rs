use std::sync::Arc;

use crate::block::entities::calibrated_sculk_sensor::CalibratedSculkSensorBlockEntity;
use crate::block::entities::sculk_sensor::SculkSensorBlockEntity;
use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, EmitsRedstonePowerArgs, GetComparatorOutputArgs,
    GetRedstonePowerArgs, OnPlaceArgs, OnScheduledTickArgs, PlacedArgs,
};
use crate::world::World;
use pumpkin_data::block_properties::{
    BlockProperties, CalibratedSculkSensorLikeProperties, HorizontalFacing,
    SculkSensorLikeProperties, SculkSensorPhase,
};
use pumpkin_data::{Block, BlockDirection, BlockId, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

pub struct SculkSensorBlock;

impl BlockMetadata for SculkSensorBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::SCULK_SENSOR, BlockId::CALIBRATED_SCULK_SENSOR].into()
    }
}

const fn horizontal_facing_to_dir(facing: HorizontalFacing) -> BlockDirection {
    match facing {
        HorizontalFacing::North => BlockDirection::North,
        HorizontalFacing::South => BlockDirection::South,
        HorizontalFacing::West => BlockDirection::West,
        HorizontalFacing::East => BlockDirection::East,
    }
}

impl SculkSensorBlock {
    pub async fn trigger(world: &Arc<World>, pos: &BlockPos, block: &Block, power: u8) {
        if block.id == BlockId::SCULK_SENSOR {
            let state = world.get_block_state(pos);
            let mut props = SculkSensorLikeProperties::from_state_id(state.id, block);
            if props.sculk_sensor_phase == SculkSensorPhase::Inactive {
                if let Some(be) = world.get_block_entity(pos)
                    && let Some(sensor_be) = be.as_any().downcast_ref::<SculkSensorBlockEntity>()
                {
                    *sensor_be.last_vibration_frequency.lock().await = power as i32;
                }

                props.sculk_sensor_phase = SculkSensorPhase::Active;
                props.power = power;
                world
                    .set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
                    .await;
                world.update_neighbors(pos, None).await;
                world.schedule_block_tick(block, *pos, 30, TickPriority::Normal);
            }
        } else if block.id == BlockId::CALIBRATED_SCULK_SENSOR {
            let state = world.get_block_state(pos);
            let mut props = CalibratedSculkSensorLikeProperties::from_state_id(state.id, block);
            if props.sculk_sensor_phase == SculkSensorPhase::Inactive {
                let back_dir = horizontal_facing_to_dir(props.facing).opposite();
                let back_pos = pos.offset(back_dir.to_offset());
                let back_state = world.get_block_state(&back_pos);
                let back_block = Block::from_state_id(back_state.id);

                let calibrated_freq = world
                    .block_registry
                    .get_weak_redstone_power(back_block, world, &back_pos, back_state, back_dir)
                    .await;

                if calibrated_freq > 0 && calibrated_freq != power {
                    return;
                }

                if let Some(be) = world.get_block_entity(pos)
                    && let Some(cal_be) = be
                        .as_any()
                        .downcast_ref::<CalibratedSculkSensorBlockEntity>()
                {
                    *cal_be.last_vibration_frequency.lock().await = power as i32;
                }

                props.sculk_sensor_phase = SculkSensorPhase::Active;
                props.power = power;
                world
                    .set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
                    .await;
                world.update_neighbors(pos, None).await;
                world.schedule_block_tick(block, *pos, 30, TickPriority::Normal);
            }
        }
    }
}

impl BlockBehaviour for SculkSensorBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            if args.block.id == BlockId::CALIBRATED_SCULK_SENSOR {
                let mut props = CalibratedSculkSensorLikeProperties::default(args.block);
                props.facing = args.player.living_entity.entity.get_horizontal_facing();
                props.to_state_id(args.block)
            } else {
                let props = SculkSensorLikeProperties::default(args.block);
                props.to_state_id(args.block)
            }
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if args.block.id == BlockId::CALIBRATED_SCULK_SENSOR {
                let entity = CalibratedSculkSensorBlockEntity::new(*args.position);
                args.world.add_block_entity(Arc::new(entity));
            } else if args.block.id == BlockId::SCULK_SENSOR {
                let entity = SculkSensorBlockEntity::new(*args.position);
                args.world.add_block_entity(Arc::new(entity));
            }
        })
    }

    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }

    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            if args.block.id == BlockId::SCULK_SENSOR {
                let props = SculkSensorLikeProperties::from_state_id(args.state.id, args.block);
                if props.sculk_sensor_phase == SculkSensorPhase::Active {
                    props.power
                } else {
                    0
                }
            } else if args.block.id == BlockId::CALIBRATED_SCULK_SENSOR {
                let props =
                    CalibratedSculkSensorLikeProperties::from_state_id(args.state.id, args.block);
                if props.sculk_sensor_phase == SculkSensorPhase::Active {
                    props.power
                } else {
                    0
                }
            } else {
                0
            }
        })
    }

    fn get_comparator_output<'a>(
        &'a self,
        args: GetComparatorOutputArgs<'a>,
    ) -> BlockFuture<'a, Option<u8>> {
        Box::pin(async move {
            let be = args.world.get_block_entity(args.position)?;
            if let Some(sensor_be) = be.as_any().downcast_ref::<SculkSensorBlockEntity>() {
                return Some(*sensor_be.last_vibration_frequency.lock().await as u8);
            }
            if let Some(cal_be) = be
                .as_any()
                .downcast_ref::<CalibratedSculkSensorBlockEntity>()
            {
                return Some(*cal_be.last_vibration_frequency.lock().await as u8);
            }
            None
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            if args.block.id == BlockId::SCULK_SENSOR {
                let mut props = SculkSensorLikeProperties::from_state_id(state.id, args.block);
                match props.sculk_sensor_phase {
                    SculkSensorPhase::Active => {
                        props.sculk_sensor_phase = SculkSensorPhase::Cooldown;
                        props.power = 0;
                        args.world
                            .set_block_state(
                                args.position,
                                props.to_state_id(args.block),
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await;
                        args.world.schedule_block_tick(
                            args.block,
                            *args.position,
                            10,
                            TickPriority::Normal,
                        );
                        args.world.update_neighbors(args.position, None).await;
                    }
                    SculkSensorPhase::Cooldown => {
                        props.sculk_sensor_phase = SculkSensorPhase::Inactive;
                        props.power = 0;
                        args.world
                            .set_block_state(
                                args.position,
                                props.to_state_id(args.block),
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await;
                        args.world.update_neighbors(args.position, None).await;
                    }
                    SculkSensorPhase::Inactive => {}
                }
            } else if args.block.id == BlockId::CALIBRATED_SCULK_SENSOR {
                let mut props =
                    CalibratedSculkSensorLikeProperties::from_state_id(state.id, args.block);
                match props.sculk_sensor_phase {
                    SculkSensorPhase::Active => {
                        props.sculk_sensor_phase = SculkSensorPhase::Cooldown;
                        props.power = 0;
                        args.world
                            .set_block_state(
                                args.position,
                                props.to_state_id(args.block),
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await;
                        args.world.schedule_block_tick(
                            args.block,
                            *args.position,
                            10,
                            TickPriority::Normal,
                        );
                        args.world.update_neighbors(args.position, None).await;
                    }
                    SculkSensorPhase::Cooldown => {
                        props.sculk_sensor_phase = SculkSensorPhase::Inactive;
                        props.power = 0;
                        args.world
                            .set_block_state(
                                args.position,
                                props.to_state_id(args.block),
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await;
                        args.world.update_neighbors(args.position, None).await;
                    }
                    SculkSensorPhase::Inactive => {}
                }
            }
        })
    }
}
