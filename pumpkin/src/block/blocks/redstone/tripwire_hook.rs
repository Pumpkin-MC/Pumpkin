use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::{
    Block, BlockDirection,
    block_properties::BlockProperties,
    sound::{Sound, SoundCategory},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::{
    BlockStateId,
    chunk::TickPriority,
    world::{BlockAccessor, BlockFlags},
};
use rand::{Rng, rng};

use crate::{
    block::pumpkin_block::{
        CanPlaceAtArgs, EmitsRedstonePowerArgs, GetRedstonePowerArgs,
        GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, OnStateReplacedArgs,
        PlayerPlacedArgs, PumpkinBlock,
    },
    world::World,
};

type TripwireProperties = pumpkin_data::block_properties::TripwireLikeProperties;
type TripwireHookProperties = pumpkin_data::block_properties::TripwireHookLikeProperties;

#[pumpkin_block("minecraft:tripwire_hook")]
pub struct TripwireHookBlock;

#[async_trait]
impl PumpkinBlock for TripwireHookBlock {
    async fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = TripwireHookProperties::default(args.block);
        props.powered = false;
        props.attached = false;
        if Self::can_place_at(args.world, args.location, args.direction).await {
            props.facing = args.direction.opposite().to_cardinal_direction();
            return props.to_state_id(args.block);
        }
        args.block.default_state.id
    }

    async fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        Self::can_place_at(args.block_accessor, args.location, args.direction).await
    }

    async fn player_placed(&self, args: PlayerPlacedArgs<'_>) {
        Self::update(
            args.world,
            *args.location,
            args.state_id,
            false,
            false,
            -1,
            None,
        )
        .await;
    }

    async fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction.to_horizontal_facing().is_some_and(|facing| {
            let props = TripwireHookProperties::from_state_id(args.state_id, args.block);
            facing.opposite() == props.facing
        }) && !Self::can_place_at(args.world, args.location, args.direction).await
        {
            Block::AIR.default_state.id
        } else {
            args.state_id
        }
    }

    async fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state_id = args.world.get_block_state_id(args.location).await;
        Self::update(args.world, *args.location, state_id, false, true, -1, None).await;
    }

    async fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        if args.moved
            || Block::from_state_id(args.old_state_id)
                .is_some_and(|old_block| old_block == args.block)
        {
            return;
        }
        let props = TripwireHookProperties::from_state_id(args.old_state_id, args.block);
        if props.powered || props.attached {
            Self::update(
                args.world,
                *args.location,
                args.old_state_id,
                true,
                false,
                -1,
                None,
            )
            .await;
        }
        if props.powered {
            args.world.update_neighbor(args.location, args.block).await;
            args.world
                .update_neighbor(
                    &args.location.offset(props.facing.opposite().to_offset()),
                    args.block,
                )
                .await;
        }
    }

    #[inline]
    async fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }

    async fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        let props = TripwireHookProperties::from_state_id(args.state.id, args.block);
        if props.powered { 15 } else { 0 }
    }

    async fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        let props = TripwireHookProperties::from_state_id(args.state.id, args.block);
        if props.powered
            && args
                .direction
                .to_horizontal_facing()
                .is_some_and(|facing| props.facing == facing)
        {
            15
        } else {
            0
        }
    }
}

impl TripwireHookBlock {
    pub async fn can_place_at(
        world: &dyn BlockAccessor,
        block_pos: &BlockPos,
        face: BlockDirection,
    ) -> bool {
        if !face.is_horizontal() {
            return false;
        }
        let place_block_pos = block_pos.offset(face.to_offset());
        let place_block_state = world.get_block_state(&place_block_pos).await;
        place_block_state.is_side_solid(face)
    }

    #[allow(clippy::too_many_lines)]
    pub async fn update(
        world: &Arc<World>,
        start_hook_pos: BlockPos,
        start_hook_state_id: BlockStateId,
        skip_state_update: bool,
        notify_neighbors: bool,
        raw_wire_index: i32,
        raw_wire_state: Option<BlockStateId>,
    ) {
        let start_hook_props =
            TripwireHookProperties::from_state_id(start_hook_state_id, &Block::TRIPWIRE_HOOK);
        let mut can_attach = !skip_state_update;
        let mut wire_attached = false;
        let mut j = 0;
        let mut wires_props: Vec<Option<TripwireProperties>> = vec![None; 42];

        for k in 1..42 {
            let current_pos = start_hook_pos.offset_dir(start_hook_props.facing.to_offset(), k);
            let current_block = world.get_block(&current_pos).await;
            if current_block == &Block::TRIPWIRE_HOOK {
                let current_hook_props = {
                    let state_id = world.get_block_state_id(&current_pos).await;
                    TripwireHookProperties::from_state_id(state_id, &Block::TRIPWIRE_HOOK)
                };
                if current_hook_props.facing == start_hook_props.facing.opposite() {
                    j = k;
                }
                break;
            }
            if current_block == &Block::TRIPWIRE || k == raw_wire_index {
                let current_wire_props = {
                    let ro_state_id = world.get_block_state_id(&current_pos).await;
                    let state_id = if k == raw_wire_index {
                        raw_wire_state.unwrap_or(ro_state_id)
                    } else {
                        ro_state_id
                    };
                    TripwireProperties::from_state_id(state_id, &Block::TRIPWIRE)
                };
                wire_attached |= (!current_wire_props.disarmed) && current_wire_props.powered;
                wires_props[k as usize] = Some(current_wire_props);
                if k == raw_wire_index {
                    world
                        .schedule_block_tick(
                            &Block::TRIPWIRE_HOOK,
                            start_hook_pos,
                            10,
                            TickPriority::Normal,
                        )
                        .await;
                    can_attach &= !current_wire_props.disarmed;
                }
            } else {
                wires_props[k as usize] = None;
                can_attach = false;
            }
        }

        let future_attached = can_attach & (j > 1);
        let future_powered = wire_attached & future_attached;
        let mut future_hook_state = TripwireHookProperties::default(&Block::TRIPWIRE_HOOK);
        future_hook_state.attached = future_attached;
        future_hook_state.powered = future_powered;

        if j > 0 {
            let end_hook_pos = start_hook_pos.offset_dir(start_hook_props.facing.to_offset(), j);
            let future_hook_facing = start_hook_props.facing.opposite();
            let mut future_end_hook_state = future_hook_state;
            future_end_hook_state.facing = future_hook_facing;
            world
                .set_block_state(
                    &end_hook_pos,
                    future_end_hook_state.to_state_id(&Block::TRIPWIRE_HOOK),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
            Self::update_neighbors_on_axis(
                &Block::TRIPWIRE_HOOK,
                world,
                end_hook_pos,
                BlockDirection::from_cardinal_direction(future_hook_facing),
            )
            .await;
            Self::play_sound(
                world,
                &end_hook_pos,
                future_attached,
                future_powered,
                start_hook_props.attached,
                start_hook_props.powered,
            )
            .await;
        }

        Self::play_sound(
            world,
            &start_hook_pos,
            future_attached,
            future_powered,
            start_hook_props.attached,
            start_hook_props.powered,
        )
        .await;

        if !skip_state_update {
            let mut future_start_hook_state = future_hook_state;
            future_start_hook_state.facing = start_hook_props.facing;
            world
                .set_block_state(
                    &start_hook_pos,
                    future_start_hook_state.to_state_id(&Block::TRIPWIRE_HOOK),
                    BlockFlags::NOTIFY_ALL,
                )
                .await;
            if notify_neighbors {
                Self::update_neighbors_on_axis(
                    &Block::TRIPWIRE_HOOK,
                    world,
                    start_hook_pos,
                    BlockDirection::from_cardinal_direction(start_hook_props.facing),
                )
                .await;
            }
        }

        if start_hook_props.attached != future_attached {
            for l in 1..j {
                let current_wrie_pos =
                    start_hook_pos.offset_dir(start_hook_props.facing.to_offset(), l);
                if let Some(mut lv8) = wires_props[l as usize] {
                    lv8.attached = future_attached;
                    world
                        .set_block_state(
                            &current_wrie_pos,
                            lv8.to_state_id(&Block::TRIPWIRE),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                    // if world.get_block(&lv7).await != Block::AIR {}
                }
            }
        }
    }

    #[allow(clippy::fn_params_excessive_bools)]
    async fn play_sound(
        world: &Arc<World>,
        block_pos: &BlockPos,
        attached: bool,
        on: bool,
        detached: bool,
        off: bool,
    ) {
        let cat = SoundCategory::Blocks;
        let pos = block_pos.to_f64();
        if on && !off {
            world
                .play_sound_raw(Sound::BlockTripwireClickOn as u16, cat, &pos, 0.4, 0.6)
                .await;
            // TODO world.emitGameEvent((Entity)null, GameEvent.BLOCK_ACTIVATE, pos);
        } else if !on && off {
            world
                .play_sound_raw(Sound::BlockTripwireClickOff as u16, cat, &pos, 0.4, 0.5)
                .await;
            // TODO world.emitGameEvent((Entity)null, GameEvent.BLOCK_DEACTIVATE, pos);
        } else if attached && !detached {
            world
                .play_sound_raw(Sound::BlockTripwireAttach as u16, cat, &pos, 0.4, 0.7)
                .await;
            // TODO world.emitGameEvent((Entity)null, GameEvent.BLOCK_ATTACH, pos);
        } else if !attached && detached {
            let pitch = 1.2 / (rng().random::<f32>() * 0.2 + 0.9);
            world
                .play_sound_raw(Sound::BlockTripwireDetach as u16, cat, &pos, 0.4, pitch)
                .await;
            // TODO world.emitGameEvent((Entity)null, GameEvent.BLOCK_DETACH, pos);
        }
    }

    pub async fn update_neighbors_on_axis(
        block: &Block,
        world: &Arc<World>,
        block_pos: BlockPos,
        direction: BlockDirection,
    ) {
        world.update_neighbor(&block_pos, block).await;
        world
            .update_neighbors(
                &block_pos.offset(direction.opposite().to_offset()),
                Some(direction),
            )
            .await;
    }
}
