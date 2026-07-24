//! Vanilla 26.2 `TargetBlock` (CFR).
//!
//! - Emits weak redstone = `POWER` state (0–15)
//! - Projectile hit sets power from hit accuracy; arrow duration 20 ticks, other 8
//! - Scheduled tick clears power to 0

use std::sync::Arc;

use crate::block::{
    BlockBehaviour, BlockFuture, EmitsRedstonePowerArgs, GetRedstonePowerArgs, OnPlaceArgs,
    OnScheduledTickArgs,
};
use crate::world::World;
use pumpkin_data::block_properties::{BlockProperties, LightWeightedPressurePlateLikeProperties};
use pumpkin_data::{Block, BlockDirection, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

type TargetProps = LightWeightedPressurePlateLikeProperties;

const ACTIVATION_TICKS_ARROW: u8 = 20;
const ACTIVATION_TICKS_OTHER: u8 = 8;

#[pumpkin_block("minecraft:target")]
pub struct TargetBlock;

impl TargetBlock {
    /// Vanilla `TargetBlock.updateRedstoneOutput` / `setOutputPower`.
    pub async fn on_projectile_hit(
        world: &Arc<World>,
        pos: &BlockPos,
        face: BlockDirection,
        hit_location: Vector3<f64>,
        is_arrow: bool,
    ) -> u8 {
        let (block, state_id) = world.get_block_and_state_id(pos);
        if *block != Block::TARGET {
            return 0;
        }
        // Don't refresh while already counting down (vanilla hasScheduledTick check).
        if world.is_block_tick_scheduled(pos, block) {
            let props = TargetProps::from_state_id(state_id, block);
            return props.power;
        }

        let strength = Self::redstone_strength(face, hit_location);
        let duration = if is_arrow {
            ACTIVATION_TICKS_ARROW
        } else {
            ACTIVATION_TICKS_OTHER
        };

        let mut props = TargetProps::from_state_id(state_id, block);
        props.power = strength;
        world
            .set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
            .await;
        world.schedule_block_tick(block, *pos, duration, TickPriority::Normal);
        strength
    }

    /// Vanilla `getRedstoneStrength`: closer to face center → higher power (1–15).
    fn redstone_strength(face: BlockDirection, hit: Vector3<f64>) -> u8 {
        let frac = |v: f64| (v - v.floor()).abs();
        let dist_x = (frac(hit.x) - 0.5).abs();
        let dist_y = (frac(hit.y) - 0.5).abs();
        let dist_z = (frac(hit.z) - 0.5).abs();
        let distance = match face {
            BlockDirection::Up | BlockDirection::Down => dist_x.max(dist_z),
            BlockDirection::North | BlockDirection::South => dist_x.max(dist_y),
            BlockDirection::East | BlockDirection::West => dist_y.max(dist_z),
        };
        let t = ((0.5 - distance) / 0.5).clamp(0.0, 1.0);
        ((15.0 * t).ceil() as u8).max(1)
    }
}

impl BlockBehaviour for TargetBlock {
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
            let props = TargetProps::from_state_id(args.state.id, args.block);
            props.power
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            // Vanilla onPlace: if power>0 and no tick scheduled, force power 0.
            let mut props = TargetProps::default(args.block);
            props.power = 0;
            props.to_state_id(args.block)
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            let mut props = TargetProps::from_state_id(state.id, args.block);
            if props.power != 0 {
                props.power = 0;
                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;
            }
        })
    }
}
