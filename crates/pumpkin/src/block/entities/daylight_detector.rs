use std::sync::Arc;

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use crate::block::registry::{Block, BlockActionResult, NormalUseArgs};
use crate::world::{BlockFlags, World};

use super::BlockEntity;

type DaylightDetectorProperties = pumpkin_data::block_properties::DaylightDetectorLikeProperties;

pub struct DaylightDetectorBlockEntity {
    pub position: BlockPos,
}

impl BlockEntity for DaylightDetectorBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(_nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        Self { position }
    }

    fn write_nbt(&self, _nbt: &mut NbtCompound) {}

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tick(&self, world: &Arc<World>) {
        if world.get_world_age() % 20 == 0 && world.dimension.has_skylight {
            Self::update_power(world, &self.position);
        }
    }
}

impl DaylightDetectorBlockEntity {
    pub const ID: &'static str = "minecraft:daylight_detector";

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self { position }
    }

    /// Clear-sky, unobstructed-sky daylight-detector power for `world_time`.
    #[must_use]
    pub fn daylight_detector_power_from_time(world_time: i64) -> u8 {
        use std::f32::consts::{PI, TAU};

        let day_fraction = world_time.rem_euclid(24_000) as f32 / 24_000.0;

        // Vanilla's smoothed celestial-angle calculation, expressed in radians.
        let linear_angle = (day_fraction - 0.25).rem_euclid(1.0);
        let curved_angle = 1.0 - ((linear_angle * PI).cos() + 1.0) * 0.5;
        let mut sun_angle = (linear_angle + (curved_angle - linear_angle) / 3.0) * TAU;

        // Vanilla clear-weather ambient darkness: 0 at noon, up to 11 at midnight.
        let ambient_darkness =
            ((1.0 - (sun_angle.cos() * 2.0 + 0.2)).clamp(0.0, 1.0) * 11.0) as u8;
        let sky_light = 15u8;
        let daylight = sky_light.saturating_sub(ambient_darkness);

        if daylight == 0 {
            return 0;
        }

        let transition_target = if sun_angle < PI { 0.0 } else { TAU };
        sun_angle += (transition_target - sun_angle) * 0.2;

        ((daylight as f32 * sun_angle.cos()).round()).clamp(0.0, 15.0) as u8
    }

    pub fn update_power(world: &Arc<World>, block_pos: &BlockPos) {
        let (block, state) = world.get_block_and_state(block_pos);
        let mut props = DaylightDetectorProperties::from_state_id(state.id);

        let daylight_power = Self::daylight_detector_power_from_time(world.get_time_of_day());

        let power = if props.inverted {
            15 - daylight_power
        } else {
            daylight_power
        };

        if power != props.power {
            props.power = power;
            world.set_block_state(
                *block_pos,
                props.to_state_id(block),
                BlockFlags::NOTIFY_ALL,
            );
        }
    }
}

// Binds player right-clicks directly into the DaylightDetector block logic
impl Block for DaylightDetectorBlockEntity {
    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let abilities = args
            .player
            .abilities
            .lock()
            .unwrap_or_else(|e| e.into_inner());

        if !abilities.allow_modify_world {
            return BlockActionResult::Pass;
        }

        let state = args.world.get_block_state(args.position);
        let mut props = DaylightDetectorProperties::from_state_id(state.id, args.block);

        props.inverted = !props.inverted;
        args.world.set_block_state(
            args.position,
            props.to_state_id(args.block),
            BlockFlags::NOTIFY_LISTENERS,
        );

        Self::update_power(args.world, &args.position);
        BlockActionResult::Success
    }
}
