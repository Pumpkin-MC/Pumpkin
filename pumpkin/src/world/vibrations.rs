//! Minimal vanilla `VibrationSystem`: routes game-event vibrations to sculk
//! sensors so they emit redstone like vanilla.
//!
//! Covers activation power, frequencies, wool occlusion, the 1-block-per-tick
//! travel delay, and amethyst resonance; the sensor phase machine lives in the
//! sculk sensor block.

use std::sync::Arc;

use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{BlockId, HorizontalFacingExt};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::block::blocks::redstone::sculk_sensor::SculkSensorBlock;
use crate::block::entities::calibrated_sculk_sensor::CalibratedSculkSensorBlockEntity;
use crate::block::entities::sculk_sensor::SculkSensorBlockEntity;
use crate::world::World;

/// Vanilla `VibrationSystem.VIBRATION_FREQUENCY_FOR_EVENT` subset for the
/// events the server currently emits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vibration {
    Step,
    ProjectileLand,
    HitGround,
    Splash,
    ProjectileShoot,
    EntityInteract,
    EntityDamage,
    Eat,
    ContainerClose,
    BlockClose,
    BlockDeactivate,
    BlockDetach,
    ContainerOpen,
    BlockOpen,
    BlockActivate,
    BlockAttach,
    PrimeFuse,
    NoteBlockPlay,
    BlockChange,
    BlockDestroy,
    BlockPlace,
    EntityPlace,
    LightningStrike,
    Teleport,
    EntityDie,
    Explode,
    /// Amethyst resonance re-emission (`GameEvent.RESONATE_1..15`): carries
    /// the resonated frequency verbatim.
    Resonate(u8),
}

impl Vibration {
    #[must_use]
    pub const fn frequency(self) -> u8 {
        match self {
            Self::Step => 1,
            Self::ProjectileLand | Self::HitGround | Self::Splash => 2,
            Self::ProjectileShoot => 3,
            Self::EntityInteract => 6,
            Self::EntityDamage => 7,
            Self::Eat => 8,
            Self::ContainerClose | Self::BlockClose | Self::BlockDeactivate | Self::BlockDetach => {
                9
            }
            Self::ContainerOpen
            | Self::BlockOpen
            | Self::BlockActivate
            | Self::BlockAttach
            | Self::PrimeFuse
            | Self::NoteBlockPlay => 10,
            Self::BlockChange => 11,
            Self::BlockDestroy => 12,
            Self::BlockPlace => 13,
            Self::EntityPlace | Self::LightningStrike | Self::Teleport => 14,
            Self::EntityDie | Self::Explode => 15,
            Self::Resonate(frequency) => frequency,
        }
    }
}

/// A vibration in flight toward a sensor: vanilla `VibrationInfo` travel time
/// is one tick per block of distance.
pub struct PendingVibration {
    pub sensor_pos: BlockPos,
    pub power: u8,
    pub frequency: u8,
    pub remaining_ticks: u32,
}

/// Vanilla `SculkSensorBlock.RESONANCE_PITCH_BEND`: note-block pitches for the
/// tone map `[0,0,2,4,6,7,9,10,12,14,15,18,19,21,22,24]`.
fn resonance_pitch(frequency: u8) -> f32 {
    const TONE_MAP: [i32; 16] = [0, 0, 2, 4, 6, 7, 9, 10, 12, 14, 15, 18, 19, 21, 22, 24];
    let tone = TONE_MAP[usize::from(frequency.min(15))];
    ((tone as f32 - 12.0) / 12.0).exp2()
}

/// Vanilla `VibrationSystem.Listener.isOccluded`, simplified to one center ray:
/// wool anywhere on the straight path swallows the vibration.
fn is_occluded(world: &World, from: Vector3<f64>, to: Vector3<f64>) -> bool {
    let delta = Vector3::new(to.x - from.x, to.y - from.y, to.z - from.z);
    let distance = delta.length();
    if distance < f64::EPSILON {
        return false;
    }
    let steps = (distance / 0.3).ceil() as i32;
    let mut last = BlockPos::floored(from.x, from.y, from.z);
    for i in 1..steps {
        let t = f64::from(i) / f64::from(steps);
        let sample = BlockPos::floored(
            delta.x.mul_add(t, from.x),
            delta.y.mul_add(t, from.y),
            delta.z.mul_add(t, from.z),
        );
        if sample == last {
            continue;
        }
        last = sample;
        if world
            .get_block(&sample)
            .has_tag(&tag::Block::MINECRAFT_OCCLUDES_VIBRATION_SIGNALS)
        {
            return true;
        }
    }
    false
}

impl World {
    /// Vanilla `Level.gameEvent` reduced to vibration dispatch: wakes every
    /// sculk sensor in listening range of the event.
    pub async fn emit_vibration(self: &Arc<Self>, event: Vibration, source: Vector3<f64>) {
        // Worlds without a single sculk sensor skip the chunk scan entirely.
        if !self
            .has_sculk_sensors
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return;
        }
        let frequency = event.frequency();
        if frequency == 0 {
            return;
        }

        // Calibrated sensors listen up to 16 blocks; scan the covering chunks.
        const MAX_RADIUS: f64 = 16.0;
        let min_chunk_x = ((source.x - MAX_RADIUS) / 16.0).floor() as i32;
        let max_chunk_x = ((source.x + MAX_RADIUS) / 16.0).floor() as i32;
        let min_chunk_z = ((source.z - MAX_RADIUS) / 16.0).floor() as i32;
        let max_chunk_z = ((source.z + MAX_RADIUS) / 16.0).floor() as i32;

        let mut sensors: Vec<BlockPos> = Vec::new();
        for chunk_x in min_chunk_x..=max_chunk_x {
            for chunk_z in min_chunk_z..=max_chunk_z {
                let Some(chunk) = self
                    .block_entities
                    .get(&pumpkin_util::math::vector2::Vector2::new(chunk_x, chunk_z))
                else {
                    continue;
                };
                for (pos, block_entity) in chunk.iter() {
                    let id = block_entity.resource_location();
                    if id == SculkSensorBlockEntity::ID
                        || id == CalibratedSculkSensorBlockEntity::ID
                    {
                        sensors.push(*pos);
                    }
                }
            }
        }

        for sensor_pos in sensors {
            let block = self.get_block(&sensor_pos);
            let radius = match block.id {
                BlockId::SCULK_SENSOR => 8.0f64,
                BlockId::CALIBRATED_SCULK_SENSOR => {
                    // Vanilla CalibratedSculkSensorBlockEntity: a redstone
                    // signal into the amethyst side only accepts vibrations of
                    // exactly that frequency.
                    let state = self.get_block_state(&sensor_pos);
                    let props =
                        pumpkin_data::block_properties::CalibratedSculkSensorLikeProperties::from_state_id(
                            state.id, block,
                        );
                    let back = props.facing.to_block_direction().opposite();
                    let back_pos = sensor_pos.offset(back.to_offset());
                    let (back_block, back_state) = self.get_block_and_state(&back_pos);
                    let comparison = crate::block::blocks::redstone::get_redstone_power(
                        back_block, back_state, self, &back_pos, back,
                    )
                    .await;
                    if comparison != 0 && comparison != frequency {
                        continue;
                    }
                    16.0f64
                }
                _ => continue,
            };

            // Vanilla: sensors ignore place/destroy of their own block.
            let sensor_block_pos = BlockPos::floored(source.x, source.y, source.z);
            if sensor_block_pos == sensor_pos
                && matches!(event, Vibration::BlockPlace | Vibration::BlockDestroy)
            {
                continue;
            }

            let center = sensor_pos.to_centered_f64();
            let dx = center.x - source.x;
            let dy = center.y - source.y;
            let dz = center.z - source.z;
            let distance = dx.mul_add(dx, dy.mul_add(dy, dz * dz)).sqrt();
            if distance > radius {
                continue;
            }
            if is_occluded(self, source, center) {
                continue;
            }

            // Vanilla getRedstoneStrengthForDistance.
            let power = (15 - (15.0 / radius * distance).floor() as i32).max(1) as u8;
            // Vanilla vibrations travel one block per tick before arriving.
            self.pending_vibrations
                .lock()
                .unwrap()
                .push(PendingVibration {
                    sensor_pos,
                    power,
                    frequency,
                    remaining_ticks: distance.floor() as u32,
                });
        }
    }

    /// Advances in-flight vibrations by one game tick and delivers arrivals,
    /// including vanilla amethyst resonance from activating sensors.
    pub async fn tick_pending_vibrations(self: &Arc<Self>) {
        let due: Vec<PendingVibration> = {
            let mut pending = self.pending_vibrations.lock().unwrap();
            if pending.is_empty() {
                return;
            }
            let mut due = Vec::new();
            pending.retain_mut(|vibration| {
                if vibration.remaining_ticks == 0 {
                    due.push(PendingVibration {
                        sensor_pos: vibration.sensor_pos,
                        power: vibration.power,
                        frequency: vibration.frequency,
                        remaining_ticks: 0,
                    });
                    false
                } else {
                    vibration.remaining_ticks -= 1;
                    true
                }
            });
            due
        };

        for vibration in due {
            let block = self.get_block(&vibration.sensor_pos);
            if block.id != BlockId::SCULK_SENSOR && block.id != BlockId::CALIBRATED_SCULK_SENSOR {
                continue;
            }
            let was_inactive = SculkSensorBlock::vibrate(
                self,
                &vibration.sensor_pos,
                block,
                vibration.power,
                vibration.frequency,
            )
            .await;
            if !was_inactive {
                continue;
            }
            // Vanilla SculkSensorBlock.tryResonateVibration: adjacent blocks
            // tagged vibration_resonators re-emit the frequency as a resonance
            // event with the amethyst chime.
            for direction in pumpkin_data::BlockDirection::all() {
                let resonator_pos = vibration.sensor_pos.offset(direction.to_offset());
                if !self
                    .get_block(&resonator_pos)
                    .has_tag(&tag::Block::MINECRAFT_VIBRATION_RESONATORS)
                {
                    continue;
                }
                self.play_sound_fine(
                    pumpkin_data::sound::Sound::BlockAmethystBlockResonate,
                    pumpkin_data::sound::SoundCategory::Blocks,
                    &resonator_pos.to_centered_f64(),
                    1.0,
                    resonance_pitch(vibration.frequency),
                );
                self.emit_vibration(
                    Vibration::Resonate(vibration.frequency),
                    resonator_pos.to_centered_f64(),
                )
                .await;
            }
        }
    }
}
