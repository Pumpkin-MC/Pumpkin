use std::sync::Arc;

use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockFuture, BrokenArgs, EmitsRedstonePowerArgs, GetComparatorOutputArgs,
    GetRedstonePowerArgs, NormalUseArgs, OnStateReplacedArgs, PlacedArgs, UseWithItemArgs,
};
use crate::entity::Entity;
use crate::entity::item::ItemEntity;
use crate::world::World;
use pumpkin_data::data_component_impl::JukeboxPlayableImpl;
use pumpkin_data::entity::EntityType;
use pumpkin_data::jukebox_song::JukeboxSong;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{
    Block,
    block_properties::{BlockProperties, JukeboxLikeProperties},
};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::block::entities::jukebox::JukeboxBlockEntity;
use pumpkin_world::world::BlockFlags;
use rand::{RngExt, rng};

#[pumpkin_block("minecraft:jukebox")]
pub struct JukeboxBlock;

impl JukeboxBlock {
    fn has_record_state(block: &Block, state_id: u16) -> bool {
        JukeboxLikeProperties::from_state_id(state_id, block).has_record
    }

    async fn set_record_state(
        has_record: bool,
        block: &Block,
        position: &BlockPos,
        world: &Arc<World>,
    ) {
        let new_state = JukeboxLikeProperties { has_record };
        world
            .set_block_state(
                position,
                new_state.to_state_id(block),
                BlockFlags::NOTIFY_LISTENERS,
            )
            .await;
    }

    /// Drops the record from the jukebox - matches vanilla's `JukeboxBlockEntity.dropRecord()`
    /// Spawns item at (pos + 0.5, pos + 1.01, pos + 0.5) with horizontal random offset
    async fn drop_record(position: &BlockPos, world: &Arc<World>) {
        if let Some(block_entity) = world.get_block_entity(position).await
            && let Some(jukebox_entity) = block_entity.as_any().downcast_ref::<JukeboxBlockEntity>()
        {
            let record = jukebox_entity.clear_record().await;
            if !record.is_empty() {
                // Vanilla: Vec3d.add(pos, 0.5, 1.01, 0.5).addHorizontalRandom(random, 0.7F)
                // addHorizontalRandom adds random in range [-0.35, 0.35] to x and z
                let spawn_pos = Vector3::new(
                    f64::from(position.0.x) + 0.5 + rng().random_range(-0.35..0.35),
                    f64::from(position.0.y) + 1.01,
                    f64::from(position.0.z) + 0.5 + rng().random_range(-0.35..0.35),
                );

                let entity = Entity::new(world.clone(), spawn_pos, &EntityType::ITEM);
                // Vanilla: setToDefaultPickupDelay() = 10 ticks
                let item_entity = Arc::new(ItemEntity::new(entity, record).await);
                world.spawn_entity(item_entity).await;
            }
        }
    }

    /// Stops the music and updates block state
    async fn stop_playing(block: &Block, position: &BlockPos, world: &Arc<World>) {
        Self::set_record_state(false, block, position, world).await;
        world
            .sync_world_event(WorldEvent::JukeboxStopsPlaying, *position, 0)
            .await;
    }

    /// Starts playing music
    async fn start_playing(position: &BlockPos, world: &Arc<World>, song_id: u32) {
        world
            .sync_world_event(WorldEvent::JukeboxStartsPlaying, *position, song_id as i32)
            .await;
    }
}

impl BlockBehaviour for JukeboxBlock {
    /// Called when the jukebox is placed - creates the block entity
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let block_entity = JukeboxBlockEntity::new(*args.position);
            args.world.add_block_entity(Arc::new(block_entity)).await;
        })
    }

    /// Called when player right-clicks with empty hand or non-disc item
    /// Vanilla: `JukeboxBlock.onUse()` - drops record if present
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state_id = args.world.get_block_state(args.position).await.id;

            // Vanilla: if (state.get(HAS_RECORD) && world.getBlockEntity(pos) instanceof JukeboxBlockEntity lv)
            if Self::has_record_state(args.block, state_id) {
                // Drop the record
                Self::drop_record(args.position, args.world).await;
                // Stop the music and update block state
                Self::stop_playing(args.block, args.position, args.world).await;
                return BlockActionResult::Success;
            }

            BlockActionResult::Pass
        })
    }

    /// Called when player right-clicks with an item
    /// Vanilla: `JukeboxBlock.onUseWithItem()` -> `JukeboxPlayableComponent.tryPlayStack()`
    fn use_with_item<'a>(
        &'a self,
        args: UseWithItemArgs<'a>,
    ) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let world = args.world;
            let state_id = world.get_block_state(args.position).await.id;

            // Vanilla: if (state.get(HAS_RECORD)) return PASS_TO_DEFAULT_BLOCK_ACTION
            if Self::has_record_state(args.block, state_id) {
                return BlockActionResult::PassToDefaultBlockAction;
            }

            let mut item_stack = args.item_stack.lock().await;

            // Vanilla: JukeboxPlayableComponent lv = stack.get(DataComponentTypes.JUKEBOX_PLAYABLE)
            let jukebox_playable = item_stack
                .get_data_component::<JukeboxPlayableImpl>()
                .map(|i| i.song);

            // Vanilla: if (lv == null) return PASS_TO_DEFAULT_BLOCK_ACTION
            let Some(jukebox_playable) = jukebox_playable else {
                return BlockActionResult::PassToDefaultBlockAction;
            };

            let Some(song_name) = jukebox_playable.split(':').nth(1) else {
                return BlockActionResult::PassToDefaultBlockAction;
            };

            let Some(jukebox_song) = JukeboxSong::from_name(song_name) else {
                log::error!("Jukebox playable song not registered: {song_name}");
                return BlockActionResult::PassToDefaultBlockAction;
            };

            // Vanilla: ItemStack lv3 = stack.splitUnlessCreative(1, player)
            let record = item_stack.split_unless_creative(args.player.gamemode.load(), 1);

            // Vanilla: lv4.setStack(lv3)
            if let Some(block_entity) = world.get_block_entity(args.position).await
                && let Some(jukebox_entity) =
                    block_entity.as_any().downcast_ref::<JukeboxBlockEntity>()
            {
                jukebox_entity.set_record(record).await;
                // Start tracking playback with song duration
                jukebox_entity.start_playing(jukebox_song.length_in_ticks());
            }

            // Update block state to has_record = true
            Self::set_record_state(true, args.block, args.position, world).await;

            // Start playing the music (client-side audio)
            Self::start_playing(args.position, world, jukebox_song.get_id()).await;

            // TODO: world.emitGameEvent(GameEvent.BLOCK_CHANGE, pos, ...)
            // TODO: player.incrementStat(Stats.PLAY_RECORD)

            BlockActionResult::Success
        })
    }

    /// Called when the jukebox is broken
    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Drop the record if there is one
            Self::drop_record(args.position, args.world).await;
            // Stop the music
            args.world
                .sync_world_event(WorldEvent::JukeboxStopsPlaying, *args.position, 0)
                .await;
        })
    }

    /// Vanilla: `JukeboxBlock.onStateReplaced()` -> `ItemScatterer.onStateReplaced()`
    fn on_state_replaced<'a>(&'a self, _args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            // Vanilla calls ItemScatterer.onStateReplaced which updates comparators
            // TODO: world.updateComparators(pos, block) when implemented
        })
    }

    /// Vanilla: `JukeboxBlock.emitsRedstonePower()` returns true
    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }

    /// Vanilla: Returns 15 if playing, 0 otherwise
    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            // Vanilla: return world.getBlockEntity(pos) instanceof JukeboxBlockEntity lv && lv.getManager().isPlaying() ? 15 : 0
            if let Some(block_entity) = args.world.get_block_entity(args.position).await
                && let Some(jukebox_entity) =
                    block_entity.as_any().downcast_ref::<JukeboxBlockEntity>()
                && jukebox_entity.is_playing()
            {
                15
            } else {
                0
            }
        })
    }

    /// Vanilla: Returns the song's comparator output (0-15)
    fn get_comparator_output<'a>(
        &'a self,
        args: GetComparatorOutputArgs<'a>,
    ) -> BlockFuture<'a, Option<u8>> {
        Box::pin(async move {
            // Vanilla: return world.getBlockEntity(pos) instanceof JukeboxBlockEntity lv ? lv.getComparatorOutput() : 0
            if let Some(block_entity) = args.world.get_block_entity(args.position).await
                && let Some(jukebox_entity) =
                    block_entity.as_any().downcast_ref::<JukeboxBlockEntity>()
            {
                let record = jukebox_entity.get_record().await;
                // Get the song from the record's jukebox_playable component
                if let Some(playable) = record.get_data_component::<JukeboxPlayableImpl>()
                    && let Some(song_name) = playable.song.split(':').nth(1)
                    && let Some(song) = JukeboxSong::from_name(song_name)
                {
                    return Some(song.comparator_output());
                }
            }
            Some(0)
        })
    }
}
