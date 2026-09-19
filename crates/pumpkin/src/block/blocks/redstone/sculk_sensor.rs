use std::sync::Arc;

use crate::block::entities::calibrated_sculk_sensor::CalibratedSculkSensorBlockEntity;
use crate::block::entities::sculk_sensor::SculkSensorBlockEntity;
use crate::block::{
    BlockBehaviour, BlockMetadata, EmitsRedstonePowerArgs, GetComparatorOutputArgs,
    GetRedstonePowerArgs, OnPlaceArgs, OnScheduledTickArgs, PathComputationType, PlacedArgs,
};
use crate::entity::player::Player;
use crate::world::World;
use pumpkin_data::block_properties::{
    CalibratedSculkSensorLikeProperties, HorizontalFacing, SculkSensorLikeProperties,
    SculkSensorPhase,
};
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::{Block, BlockDirection, BlockId, BlockState, BlockStateId};
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

/// Both sensor variants carry the same phase property under different types.
fn sculk_sensor_phase(block: &Block, state_id: BlockStateId) -> SculkSensorPhase {
    if block.id == BlockId::CALIBRATED_SCULK_SENSOR {
        CalibratedSculkSensorLikeProperties::from_state_id(state_id).sculk_sensor_phase
    } else {
        SculkSensorLikeProperties::from_state_id(state_id).sculk_sensor_phase
    }
}

/// How long a sensor stays active before it goes into cooldown. Vanilla's
/// `SculkSensorBlock.getActiveTicks`.
const ACTIVE_TICKS: u8 = 30;

impl SculkSensorBlock {
    /// Emits what vanilla's `SculkSensorBlock.activate` emits as the sensor lights up.
    ///
    /// This is the only event a sculk shrieker listens for, so it is what chains a shrieker
    /// to a sensor, rather than the shrieker hearing the original source itself.
    fn emit_tendrils_clicking(world: &Arc<World>, pos: &BlockPos, source_player: Option<&Player>) {
        world.emit_game_event(
            GameEvent::SculkSensorTendrilsClicking.name(),
            pos.to_centered_f64(),
            source_player,
        );
    }

    /// Activates the sensor at `pos`.
    ///
    /// `frequency` identifies the game event that caused the vibration and is what a
    /// comparator reads back, while `power` is the redstone strength the sensor emits and
    /// is derived from how far the vibration travelled. The two are unrelated in vanilla.
    ///
    /// `source_player` is carried over to the `sculk_sensor_tendrils_clicking` event the
    /// sensor emits when it activates; that event is what a shrieker in range listens for.
    ///
    /// Players tick in parallel, so two footsteps can reach the same sensor within one
    /// tick. The sensor's own block entity lock is what claims the inactive -> active
    /// transition, so only one of them activates it and runs the side effects; vanilla
    /// needs no such claim because it ticks on a single thread. The lock is released
    /// before those side effects, since the neighbour update comes back through
    /// `get_comparator_output` and would take it again.
    pub fn trigger(
        world: &Arc<World>,
        pos: &BlockPos,
        block: &Block,
        frequency: u8,
        power: u8,
        source_player: Option<&Player>,
    ) {
        let new_state_id = if block.id == BlockId::SCULK_SENSOR {
            let Some(sensor_be) = world.get_block_entity(pos) else {
                return;
            };
            let Some(sensor_be) = sensor_be.as_any().downcast_ref::<SculkSensorBlockEntity>()
            else {
                return;
            };
            let mut last_frequency = sensor_be
                .last_vibration_frequency
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            let state = world.get_block_state(pos);
            let mut props = SculkSensorLikeProperties::from_state_id(state.id);
            if props.sculk_sensor_phase != SculkSensorPhase::Inactive {
                return;
            }
            *last_frequency = frequency.into();
            props.sculk_sensor_phase = SculkSensorPhase::Active;
            props.power = power;
            let new_state_id = props.to_state_id(block);
            // Only the listeners here: `NOTIFY_NEIGHBORS` would notify the neighbours
            // synchronously while this lock is held, and a comparator next to the sensor
            // reads it back through `get_comparator_output`, which takes the same lock.
            // The neighbours are notified below instead, once it has been released.
            world.set_block_state(pos, new_state_id, BlockFlags::NOTIFY_LISTENERS);
            new_state_id
        } else if block.id == BlockId::CALIBRATED_SCULK_SENSOR {
            let state = world.get_block_state(pos);
            let props = CalibratedSculkSensorLikeProperties::from_state_id(state.id);
            if props.sculk_sensor_phase != SculkSensorPhase::Inactive {
                return;
            }

            // Read the calibrating signal before taking the lock below: the block behind
            // may be a comparator reading this very sensor back, which would come through
            // `get_comparator_output` and take the same lock.
            let back_dir = horizontal_facing_to_dir(props.facing).opposite();
            let back_pos = pos.offset(back_dir.to_offset());
            let back_state = world.get_block_state(&back_pos);
            let back_block = Block::from_state_id(back_state.id);
            let calibrated_freq = world
                .block_registry
                .get_weak_redstone_power(back_block, world, &back_pos, back_state, back_dir);
            if calibrated_freq > 0 && calibrated_freq != frequency {
                return;
            }

            let Some(cal_be) = world.get_block_entity(pos) else {
                return;
            };
            let Some(cal_be) = cal_be
                .as_any()
                .downcast_ref::<CalibratedSculkSensorBlockEntity>()
            else {
                return;
            };
            let mut last_frequency = cal_be
                .last_vibration_frequency
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            // Re-read under the lock; the phase may have been claimed since the check above.
            let state = world.get_block_state(pos);
            let mut props = CalibratedSculkSensorLikeProperties::from_state_id(state.id);
            if props.sculk_sensor_phase != SculkSensorPhase::Inactive {
                return;
            }
            *last_frequency = frequency.into();
            props.sculk_sensor_phase = SculkSensorPhase::Active;
            props.power = power;
            let new_state_id = props.to_state_id(block);
            world.set_block_state(pos, new_state_id, BlockFlags::NOTIFY_LISTENERS);
            new_state_id
        } else {
            return;
        };

        // Same order as vanilla's `SculkSensorBlock.activate`: schedule the deactivation
        // tick, then update the neighbours, then emit the event. The analog-output update is
        // the part `NOTIFY_NEIGHBORS` would have done during the state write above; it is
        // what lets a comparator reading the sensor pick the new frequency up.
        world.schedule_block_tick(block, *pos, ACTIVE_TICKS, TickPriority::Normal);
        world.update_neighbors(pos, None);
        if new_state_id.has_analog_output_signal() {
            world.update_neighbour_for_output_signal(pos, block);
        }
        Self::emit_tendrils_clicking(world, pos, source_player);
    }
}

impl BlockBehaviour for SculkSensorBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        if args.block.id == BlockId::CALIBRATED_SCULK_SENSOR {
            let mut props = CalibratedSculkSensorLikeProperties::default(args.block);
            props.facing = args.player.living_entity.entity.get_horizontal_facing();
            props.to_state_id(args.block)
        } else {
            let props = SculkSensorLikeProperties::default(args.block);
            props.to_state_id(args.block)
        }
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        if args.block.id == BlockId::CALIBRATED_SCULK_SENSOR {
            let entity = CalibratedSculkSensorBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        } else if args.block.id == BlockId::SCULK_SENSOR {
            let entity = SculkSensorBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(entity));
        }
    }

    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }

    fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        if args.block.id == BlockId::SCULK_SENSOR {
            let props = SculkSensorLikeProperties::from_state_id(args.state.id);
            if props.sculk_sensor_phase == SculkSensorPhase::Active {
                props.power
            } else {
                0
            }
        } else if args.block.id == BlockId::CALIBRATED_SCULK_SENSOR {
            let props = CalibratedSculkSensorLikeProperties::from_state_id(args.state.id);
            if props.sculk_sensor_phase == SculkSensorPhase::Active {
                props.power
            } else {
                0
            }
        } else {
            0
        }
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        // Vanilla reads the frequency only while the sensor is active.
        if sculk_sensor_phase(args.block, args.state.id) != SculkSensorPhase::Active {
            return Some(0);
        }

        let be = args.world.get_block_entity(args.position)?;
        if let Some(sensor_be) = be.as_any().downcast_ref::<SculkSensorBlockEntity>() {
            return Some(
                *sensor_be
                    .last_vibration_frequency
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) as u8,
            );
        }
        if let Some(cal_be) = be
            .as_any()
            .downcast_ref::<CalibratedSculkSensorBlockEntity>()
        {
            return Some(
                *cal_be
                    .last_vibration_frequency
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) as u8,
            );
        }
        None
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state(args.position);
        if args.block.id == BlockId::SCULK_SENSOR {
            let mut props = SculkSensorLikeProperties::from_state_id(state.id);
            match props.sculk_sensor_phase {
                SculkSensorPhase::Active => {
                    props.sculk_sensor_phase = SculkSensorPhase::Cooldown;
                    props.power = 0;
                    args.world.set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    );
                    args.world.schedule_block_tick(
                        args.block,
                        *args.position,
                        10,
                        TickPriority::Normal,
                    );
                }
                SculkSensorPhase::Cooldown => {
                    props.sculk_sensor_phase = SculkSensorPhase::Inactive;
                    props.power = 0;
                    args.world.set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    );
                }
                SculkSensorPhase::Inactive => {}
            }
        } else if args.block.id == BlockId::CALIBRATED_SCULK_SENSOR {
            let mut props = CalibratedSculkSensorLikeProperties::from_state_id(state.id);
            match props.sculk_sensor_phase {
                SculkSensorPhase::Active => {
                    props.sculk_sensor_phase = SculkSensorPhase::Cooldown;
                    props.power = 0;
                    args.world.set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    );
                    args.world.schedule_block_tick(
                        args.block,
                        *args.position,
                        10,
                        TickPriority::Normal,
                    );
                }
                SculkSensorPhase::Cooldown => {
                    props.sculk_sensor_phase = SculkSensorPhase::Inactive;
                    props.power = 0;
                    args.world.set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    );
                }
                SculkSensorPhase::Inactive => {}
            }
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
