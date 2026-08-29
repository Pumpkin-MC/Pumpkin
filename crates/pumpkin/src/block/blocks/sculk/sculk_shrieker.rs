use std::sync::Arc;

use crate::block::entities::sculk_shrieker::SculkShriekerBlockEntity;
use crate::block::{BlockBehaviour, BlockMetadata, OnPlaceArgs, OnScheduledTickArgs};
use crate::world::World;
use pumpkin_data::potion::Effect;
use pumpkin_data::{
    BlockId, BlockStateId,
    block_properties::{BlockProperties, SculkShriekerLikeProperties},
    effect::StatusEffect,
    sound::{Sound, SoundCategory},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

const SHRIEK_TICKS: u8 = 90;
const DARKNESS_RADIUS: f64 = 40.0;
const DARKNESS_DURATION: i32 = 260;

pub struct SculkShriekerBlock;

impl BlockMetadata for SculkShriekerBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::SCULK_SHRIEKER].into()
    }
}

impl SculkShriekerBlock {
    pub fn try_activate(world: &Arc<World>, pos: &BlockPos) -> bool {
        let block = world.get_block(pos);
        if block.id != BlockId::SCULK_SHRIEKER {
            return false;
        }
        let state = world.get_block_state(pos);
        let mut props = SculkShriekerLikeProperties::from_state_id(state.id, block);

        if props.shrieking {
            return false;
        }

        props.shrieking = true;
        world.set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_ALL);

        world.play_sound(
            Sound::BlockSculkShriekerShriek,
            SoundCategory::Blocks,
            &pos.to_f64(),
        );

        world.schedule_block_tick(block, *pos, SHRIEK_TICKS, TickPriority::Normal);

        let center = pos.to_centered_f64();
        for player in world.get_nearby_players(center, DARKNESS_RADIUS) {
            let darkness = Effect {
                effect_type: &StatusEffect::DARKNESS,
                duration: DARKNESS_DURATION,
                amplifier: 0,
                ambient: false,
                show_particles: false,
                show_icon: true,
                blend: true,
            };
            player.send_effect(&darkness);
            player.living_entity.add_effect(darkness);
        }

        if let Some(entity) = world.get_block_entity(pos)
            && let Some(shrieker) = entity.as_any().downcast_ref::<SculkShriekerBlockEntity>()
        {
            let mut level = shrieker
                .warning_level
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *level = (*level + 1).min(4);
            if props.can_summon && *level >= 4 {
                // TODO: spawn Warden near pos
            }
        }

        true
    }
}

impl BlockBehaviour for SculkShriekerBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = SculkShriekerLikeProperties::default(args.block);
        props.shrieking = false;
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state(args.position);
        let mut props = SculkShriekerLikeProperties::from_state_id(state.id, args.block);
        if props.shrieking {
            props.shrieking = false;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
        }
    }
}
