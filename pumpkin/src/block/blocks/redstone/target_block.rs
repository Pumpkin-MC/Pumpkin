use std::sync::Arc;

use pumpkin_data::BlockDirection;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{
    Axis, BlockProperties, LightWeightedPressurePlateLikeProperties,
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockBehaviour, BlockFuture, BlockIsReplacing, EmitsRedstonePowerArgs, GetRedstonePowerArgs,
    OnPlaceArgs, OnScheduledTickArgs,
};
use crate::world::World;

/// The `target` block shares its state layout (a single `power` property) with the
/// weighted pressure plates, so the generated properties type is named after them.
type TargetProperties = LightWeightedPressurePlateLikeProperties;

#[pumpkin_block("minecraft:target")]
pub struct TargetBlock;

impl TargetBlock {
    /// Signal strength of a dead centre hit.
    pub const MAX_POWER: u8 = 15;
    /// Ticks a target stays powered after being hit by an arrow or a trident.
    pub const PERSISTENT_PROJECTILE_DELAY: u8 = 20;
    /// Ticks a target stays powered after being hit by any other projectile.
    pub const PROJECTILE_DELAY: u8 = 8;

    /// Applies the redstone power of a projectile hit and schedules the reset back to zero.
    ///
    /// Returns the signal strength the hit is worth. As in vanilla, the power is only
    /// applied when no reset is queued for this position yet, so a target that is already
    /// lit up keeps its original signal until the reset runs.
    pub async fn trigger(
        world: &Arc<World>,
        position: &BlockPos,
        face: BlockDirection,
        hit_pos: Vector3<f64>,
        delay: u8,
    ) -> u8 {
        let power = Self::calculate_power(face, hit_pos);
        let (block, state_id) = world.get_block_and_state_id(position);
        if world.is_block_tick_scheduled(position, block) {
            return power;
        }

        let mut props = TargetProperties::from_state_id(state_id, block);
        props.power = power;
        world
            .set_block_state(position, props.to_state_id(block), BlockFlags::NOTIFY_ALL)
            .await;
        world.schedule_block_tick(block, *position, delay, TickPriority::Normal);
        power
    }

    /// The closer the hit is to the centre of the hit face, the stronger the signal.
    /// `offset` is the largest of the two in face distances from that centre, so the
    /// signal only reaches 15 on a dead centre hit.
    fn calculate_power(face: BlockDirection, hit_pos: Vector3<f64>) -> u8 {
        let x = (hit_pos.x - hit_pos.x.floor() - 0.5).abs();
        let y = (hit_pos.y - hit_pos.y.floor() - 0.5).abs();
        let z = (hit_pos.z - hit_pos.z.floor() - 0.5).abs();
        let offset = match face.to_axis() {
            Axis::X => y.max(z),
            Axis::Y => x.max(z),
            Axis::Z => x.max(y),
        };
        let power =
            (f64::from(Self::MAX_POWER) * ((0.5 - offset) / 0.5).clamp(0.0, 1.0)).ceil() as u8;
        power.max(1)
    }
}

impl BlockBehaviour for TargetBlock {
    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            // Vanilla resets a carried-over nonzero OUTPUT_POWER when the block
            // identity at this position changes (TargetBlock.onPlace).
            let mut props = if let BlockIsReplacing::Itself(old_state_id) = args.replacing {
                TargetProperties::from_state_id(old_state_id, args.block)
            } else {
                TargetProperties::default(args.block)
            };
            props.power = 0;
            props.to_state_id(args.block)
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
        Box::pin(async move { TargetProperties::from_state_id(args.state.id, args.block).power })
    }

    // Vanilla only overrides `getWeakRedstonePower` for the target block, so strong power
    // stays at the default of 0 and a target never powers the block it is next to.

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position);
            let mut props = TargetProperties::from_state_id(state.id, args.block);
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
