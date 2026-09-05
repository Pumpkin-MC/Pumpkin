use std::sync::Arc;

use crate::block::entities::daylight_detector::DaylightDetectorBlockEntity;
use pumpkin_data::game_event::GameEvent;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockActionResult, BlockBehaviour, BrokenArgs, EmitsRedstonePowerArgs, GetRedstonePowerArgs,
    NormalUseArgs, PlacedArgs,
};
use crate::world::World;

type DaylightDetectorProperties = pumpkin_data::block_properties::DaylightDetectorLikeProperties;

#[pumpkin_block("minecraft:daylight_detector")]
pub struct DaylightDetectorBlock;

impl BlockBehaviour for DaylightDetectorBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        args.world
            .add_block_entity(Arc::new(DaylightDetectorBlockEntity::new(*args.position)));
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        args.world.remove_block_entity(args.position);
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let player_abilities = args
            .player
            .abilities
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !player_abilities.allow_modify_world {
            return BlockActionResult::Pass;
        }

        let state = args.world.get_block_state(args.position);
        let mut props = DaylightDetectorProperties::from_state_id(state.id);
        props.inverted = !props.inverted;

        let new_state = props.to_state_id(args.block);
        args.world
            .set_block_state(args.position, new_state, BlockFlags::NOTIFY_LISTENERS);
        args.world.emit_game_event(
            GameEvent::BlockChange.name(),
            args.position.to_centered_f64(),
        );

        Self::update_signal_strength(args.world, args.position);

        BlockActionResult::Success
    }

    fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        let props = DaylightDetectorProperties::from_state_id(args.state.id);
        props.power
    }

    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }
}

impl DaylightDetectorBlock {
    #[must_use]
    pub fn calculate_signal_strength(
        effective_sky_brightness: i32,
        sun_angle_radians: f32,
        is_inverted: bool,
    ) -> u8 {
        let mut target = effective_sky_brightness;
        let mut sun_angle = sun_angle_radians;
        if is_inverted {
            target = 15 - target;
        } else if target > 0 {
            let offset = if sun_angle < std::f32::consts::PI {
                0.0
            } else {
                std::f32::consts::PI * 2.0
            };
            sun_angle += (offset - sun_angle) * 0.2;
            target = ((target as f32 * sun_angle.cos()) + 0.5).floor() as i32;
        }

        target.clamp(0, 15) as u8
    }

    pub fn update_signal_strength(world: &Arc<World>, block_pos: &BlockPos) {
        let (block, state) = world.get_block_and_state(block_pos);
        let mut props = DaylightDetectorProperties::from_state_id(state.id);

        let target = Self::calculate_signal_strength(
            world.get_effective_sky_brightness(block_pos),
            world.get_sun_angle(block_pos),
            props.inverted,
        );

        if props.power != target {
            props.power = target;
            let new_state = props.to_state_id(block);
            world.set_block_state(block_pos, new_state, BlockFlags::NOTIFY_ALL);
        }
    }
}
