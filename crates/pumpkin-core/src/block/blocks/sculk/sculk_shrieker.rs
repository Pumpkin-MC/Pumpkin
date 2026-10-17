use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::block::entities::sculk_shrieker::SculkShriekerBlockEntity;
use crate::block::{
    BlockBehaviour, BlockMetadata, OnEntityStepArgs, OnPlaceArgs, OnScheduledTickArgs,
};
use crate::entity::Entity;
use crate::entity::mob::warden::WardenEntity;
use crate::entity::mob::warden_spawn_tracker::{MAX_WARNING_LEVEL, WardenSpawnTracker};
use crate::entity::player::Player;
use crate::entity::spawn_util::{SpawnStrategy, try_spawn_mob};
use crate::world::World;
use pumpkin_data::{
    BlockId, BlockStateId,
    block_properties::SculkShriekerLikeProperties,
    entity::EntityType,
    game_event::GameEvent,
    sound::{Sound, SoundCategory},
    world::WorldEvent,
};
use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

const WARNING_SOUND_RADIUS: i32 = 10;
const WARDEN_SPAWN_ATTEMPTS: i32 = 20;
const WARDEN_SPAWN_RANGE_XZ: i32 = 5;
const WARDEN_SPAWN_RANGE_Y: i32 = 6;
const DARKNESS_RADIUS: f64 = 40.0;
const SHRIEKING_TICKS: u8 = 90;

pub struct SculkShriekerBlock;

impl BlockMetadata for SculkShriekerBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::SCULK_SHRIEKER].into()
    }
}

impl SculkShriekerBlock {
    fn try_get_player(world: &World, entity: &Entity) -> Option<Arc<Player>> {
        if let Some(player) = world.get_player_by_id(entity.entity_id) {
            return Some(player);
        }
        let passengers = entity
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let rider = passengers.first()?;
        world.get_player_by_id(rider.get_entity().entity_id)
    }

    fn with_shrieker<R>(
        world: &World,
        pos: &BlockPos,
        f: impl FnOnce(&SculkShriekerBlockEntity) -> R,
    ) -> Option<R> {
        let entity = world.get_block_entity(pos)?;
        entity
            .as_any()
            .downcast_ref::<SculkShriekerBlockEntity>()
            .map(f)
    }

    fn can_respond(world: &World, can_summon: bool) -> bool {
        let level_info = world.level_info.load();
        can_summon
            && level_info.difficulty != Difficulty::Peaceful
            && level_info.game_rules.spawn_wardens
    }

    pub fn try_shriek(world: &Arc<World>, pos: &BlockPos, player: &Arc<Player>) {
        let block = world.get_block(pos);
        if block.id != BlockId::SCULK_SHRIEKER {
            return;
        }
        let state = world.get_block_state(pos);
        let mut props = SculkShriekerLikeProperties::from_state_id(state.id);
        if props.shrieking {
            return;
        }

        let can_respond = Self::can_respond(world, props.can_summon);
        let warning_level = if can_respond {
            WardenSpawnTracker::try_warn(world, pos, player)
        } else {
            Some(0)
        };
        Self::with_shrieker(world, pos, |shrieker| {
            *shrieker
                .warning_level
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = warning_level.unwrap_or(0);
            shrieker.shrieking_can_summon.store(
                warning_level.is_some() && props.can_summon,
                Ordering::Relaxed,
            );
        });
        if warning_level.is_none() {
            return;
        }

        props.shrieking = true;
        world.set_block_state(pos, props.to_state_id(block), BlockFlags::NOTIFY_LISTENERS);
        world.schedule_block_tick(block, *pos, SHRIEKING_TICKS, TickPriority::Normal);
        world.sync_world_event(WorldEvent::ParticlesSculkShriek, *pos, 0);
        world.emit_game_event(GameEvent::Shriek.name(), pos.to_centered_f64());
    }

    fn try_respond(world: &Arc<World>, pos: &BlockPos, can_summon: bool) {
        let warning_level = Self::with_shrieker(world, pos, |shrieker| {
            shrieker
                .shrieking_can_summon
                .store(false, Ordering::Relaxed);
            *shrieker
                .warning_level
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        });
        if let Some(warning_level) = warning_level
            && can_summon
        {
            Self::respond(world, pos, warning_level);
        }
    }

    pub fn respond(world: &Arc<World>, pos: &BlockPos, warning_level: i32) {
        if !Self::can_respond(world, true) || warning_level <= 0 {
            return;
        }

        if !Self::try_summon_warden(world, pos, warning_level) {
            Self::play_warden_reply_sound(world, pos, warning_level);
        }
        WardenEntity::apply_darkness_around(world, pos.to_centered_f64(), DARKNESS_RADIUS);
    }

    fn try_summon_warden(world: &Arc<World>, pos: &BlockPos, warning_level: i32) -> bool {
        if warning_level < MAX_WARNING_LEVEL {
            return false;
        }
        let Some(warden) = try_spawn_mob(
            &EntityType::WARDEN,
            WardenEntity::new,
            world,
            pos,
            WARDEN_SPAWN_ATTEMPTS,
            WARDEN_SPAWN_RANGE_XZ,
            WARDEN_SPAWN_RANGE_Y,
            SpawnStrategy::OnTopOfCollider,
            false,
        ) else {
            return false;
        };
        warden.emerge();
        true
    }

    fn play_warden_reply_sound(world: &World, pos: &BlockPos, warning_level: i32) {
        let sound = match warning_level {
            1 => Sound::EntityWardenNearbyClose,
            2 => Sound::EntityWardenNearbyCloser,
            3 => Sound::EntityWardenNearbyClosest,
            4 => Sound::EntityWardenListeningAngry,
            _ => return,
        };
        let radius = WARNING_SOUND_RADIUS;
        let sound_pos = {
            let mut random = rand::rng();
            pos.add(
                random.random_range(-radius..=radius),
                random.random_range(-radius..=radius),
                random.random_range(-radius..=radius),
            )
        };
        world.play_sound_fine(sound, SoundCategory::Hostile, &sound_pos.to_f64(), 5.0, 1.0);
    }
}

impl BlockBehaviour for SculkShriekerBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = SculkShriekerLikeProperties::default(args.block);
        props.shrieking = false;
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }

    fn on_entity_step(&self, args: OnEntityStepArgs<'_>) {
        if let Some(player) = Self::try_get_player(args.world, args.entity.get_entity()) {
            Self::try_shriek(args.world, args.position, &player);
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state(args.position);
        let mut props = SculkShriekerLikeProperties::from_state_id(state.id);
        if props.shrieking {
            props.shrieking = false;
            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );
            Self::try_respond(args.world, args.position, props.can_summon);
        }
    }
}
