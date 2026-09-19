use std::any::Any;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use pumpkin_data::data_component_impl::JukeboxPlayableImpl;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::jukebox_song::JukeboxSong;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::position::BlockPos;

use crate::block::entities::BlockEntity;
use crate::world::World;
use pumpkin_inventory::{Clearable, Inventory};

/// Matches vanilla's `JukeboxBlockEntity`
pub struct JukeboxBlockEntity {
    position: BlockPos,
    /// The record item stored in the jukebox (`RecordItem` in NBT)
    record_stack: Arc<Mutex<ItemStack>>,
    /// Ticks since the current song started playing
    ticks_since_song_started: AtomicU64,
    /// Length of the current song in ticks (0 if not playing)
    song_length_ticks: AtomicU64,
    dirty: AtomicBool,
    comparator_dirty: AtomicBool,
}

const RECORD_ITEM_NBT_KEY: &str = "RecordItem";
const TICKS_SINCE_SONG_STARTED_NBT_KEY: &str = "ticks_since_song_started";

impl BlockEntity for JukeboxBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let record_stack = nbt
            .get_compound(RECORD_ITEM_NBT_KEY)
            .and_then(ItemStack::read_item_stack)
            .unwrap_or_else(|| ItemStack::EMPTY.clone());

        let ticks_since_song_started = nbt
            .get_long(TICKS_SINCE_SONG_STARTED_NBT_KEY)
            .and_then(|ticks| u64::try_from(ticks).ok());
        let song_length_ticks = ticks_since_song_started
            .and_then(|ticks| {
                Self::song_length(&record_stack)
                    .filter(|length| ticks < *length)
                    .map(|length| (ticks, length))
            })
            .unwrap_or((0, 0));

        Self {
            position,
            record_stack: Arc::new(Mutex::new(record_stack)),
            ticks_since_song_started: AtomicU64::new(song_length_ticks.0),
            song_length_ticks: AtomicU64::new(song_length_ticks.1),
            dirty: AtomicBool::new(false),
            comparator_dirty: AtomicBool::new(false),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        let record = self
            .record_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !record.is_empty() {
            let mut record_nbt = NbtCompound::new();
            record.write_item_stack(&mut record_nbt);
            nbt.put(RECORD_ITEM_NBT_KEY, record_nbt);
        }

        let ticks = self.ticks_since_song_started.load(Ordering::Relaxed);
        if self.song_length_ticks.load(Ordering::Relaxed) > 0 {
            nbt.put_long(TICKS_SINCE_SONG_STARTED_NBT_KEY, ticks as i64);
        }
    }

    fn tick(&self, _world: &Arc<World>) {
        // Increment ticks if we're playing
        let song_length = self.song_length_ticks.load(Ordering::Relaxed);
        if song_length > 0 {
            let ticks = self
                .ticks_since_song_started
                .fetch_add(1, Ordering::Relaxed);
            // Check if song has finished
            if ticks >= song_length {
                self.stop_playing();
                // TODO: Update block state to has_record = false? Or just stop redstone?
                // In vanilla, the disc stays but music stops and redstone turns off
            }
        }
    }

    fn is_comparator_dirty(&self) -> bool {
        self.comparator_dirty.load(Ordering::Relaxed)
    }

    fn clear_comparator_dirty(&self) {
        self.comparator_dirty.store(false, Ordering::Relaxed);
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(record) = self.record_stack.try_lock()
            && !record.is_empty()
        {
            let mut record_nbt = NbtCompound::new();
            record.write_item_stack(&mut record_nbt);
            nbt.put("RecordItem", NbtTag::Compound(record_nbt));
        }
        if self.song_length_ticks.load(Ordering::Relaxed) > 0 {
            nbt.put_long(
                TICKS_SINCE_SONG_STARTED_NBT_KEY,
                self.ticks_since_song_started.load(Ordering::Relaxed) as i64,
            );
        }
        Some(nbt)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        Some(self)
    }
}

impl JukeboxBlockEntity {
    pub const ID: &'static str = "minecraft:jukebox";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            record_stack: Arc::new(Mutex::new(ItemStack::EMPTY.clone())),
            ticks_since_song_started: AtomicU64::new(0),
            song_length_ticks: AtomicU64::new(0),
            dirty: AtomicBool::new(false),
            comparator_dirty: AtomicBool::new(false),
        }
    }

    fn song_length(record: &ItemStack) -> Option<u64> {
        let playable = record.get_data_component::<JukeboxPlayableImpl>()?;
        let song_name = playable.song.rsplit(':').next()?;
        JukeboxSong::from_name(song_name).map(|song| song.length_in_ticks())
    }

    /// Get the current record stack
    pub fn get_record(&self) -> ItemStack {
        self.record_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Set the record stack - matches vanilla's `setStack()`
    /// Note: The caller is responsible for updating block state and playing music
    pub fn set_record(&self, stack: ItemStack) {
        *self
            .record_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = stack;
        self.mark_dirty();
    }

    /// Clear the stack and return what was there - used for dropping
    pub fn clear_record(&self) -> ItemStack {
        self.stop_playing();
        let mut record = self
            .record_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let taken = record.clone();
        *record = ItemStack::EMPTY.clone();
        self.mark_dirty();
        taken
    }

    /// Start playing a song with the given length in ticks
    pub fn start_playing(&self, length_in_ticks: u64) {
        self.ticks_since_song_started.store(0, Ordering::Relaxed);
        self.song_length_ticks
            .store(length_in_ticks, Ordering::Relaxed);
        self.mark_dirty();
    }

    /// Stop playing the current song
    pub fn stop_playing(&self) {
        self.ticks_since_song_started.store(0, Ordering::Relaxed);
        self.song_length_ticks.store(0, Ordering::Relaxed);
        self.mark_dirty();
    }

    /// Check if a song is currently playing
    pub fn is_playing(&self) -> bool {
        let song_length = self.song_length_ticks.load(Ordering::Relaxed);
        if song_length == 0 {
            return false;
        }
        let ticks = self.ticks_since_song_started.load(Ordering::Relaxed);
        ticks < song_length
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
        self.comparator_dirty.store(true, Ordering::Relaxed);
    }
}

/// Implements single-slot inventory for jukebox (matches vanilla's `SingleStackInventory`)
impl Inventory for JukeboxBlockEntity {
    fn size(&self) -> usize {
        1
    }

    fn is_empty(&self) -> bool {
        self.record_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty()
    }

    fn get_stack(&self, _slot: usize) -> ItemStack {
        self.record_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    fn remove_stack(&self, _slot: usize) -> ItemStack {
        self.stop_playing();
        let mut record = self
            .record_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let taken = record.clone();
        *record = ItemStack::EMPTY.clone();
        self.mark_dirty();
        taken
    }

    fn remove_stack_specific(&self, _slot: usize, _amount: u8) -> ItemStack {
        // Jukebox only holds one item, so remove the whole stack
        self.remove_stack(0)
    }

    fn set_stack(&self, _slot: usize, stack: ItemStack) {
        *self
            .record_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = stack;
        self.mark_dirty();
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
        self.comparator_dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for JukeboxBlockEntity {
    fn clear(&self) {
        self.stop_playing();
        *self
            .record_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = ItemStack::EMPTY.clone();
        self.mark_dirty();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::item::Item;

    fn jukebox_nbt(ticks: Option<i64>) -> NbtCompound {
        let mut record = NbtCompound::new();
        ItemStack::new(1, &Item::MUSIC_DISC_PIGSTEP).write_item_stack(&mut record);

        let mut nbt = NbtCompound::new();
        nbt.put_compound(RECORD_ITEM_NBT_KEY, record);
        if let Some(ticks) = ticks {
            nbt.put_long(TICKS_SINCE_SONG_STARTED_NBT_KEY, ticks);
        }
        nbt
    }

    #[test]
    fn restores_active_song_from_nbt() {
        let entity = JukeboxBlockEntity::from_nbt(&jukebox_nbt(Some(20)), BlockPos::new(0, 64, 0));

        assert!(entity.is_playing());
        assert_eq!(entity.ticks_since_song_started.load(Ordering::Relaxed), 20);
        assert_eq!(
            entity.song_length_ticks.load(Ordering::Relaxed),
            JukeboxSong::Pigstep.length_in_ticks()
        );
    }

    #[test]
    fn does_not_start_song_without_playback_ticks() {
        let entity = JukeboxBlockEntity::from_nbt(&jukebox_nbt(None), BlockPos::new(0, 64, 0));

        assert!(!entity.is_playing());
    }

    #[test]
    fn preserves_song_started_at_tick_zero() {
        let entity = JukeboxBlockEntity::new(BlockPos::new(0, 64, 0));
        entity.set_record(ItemStack::new(1, &Item::MUSIC_DISC_PIGSTEP));
        entity.start_playing(JukeboxSong::Pigstep.length_in_ticks());

        let mut nbt = NbtCompound::new();
        entity.write_nbt(&mut nbt);
        let restored = JukeboxBlockEntity::from_nbt(&nbt, BlockPos::new(0, 64, 0));

        assert!(restored.is_playing());
        assert_eq!(restored.ticks_since_song_started.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn does_not_restore_finished_song() {
        let length = JukeboxSong::Pigstep.length_in_ticks();
        let entity = JukeboxBlockEntity::from_nbt(
            &jukebox_nbt(Some(length as i64)),
            BlockPos::new(0, 64, 0),
        );

        assert!(!entity.is_playing());
        assert_eq!(entity.song_length_ticks.load(Ordering::Relaxed), 0);
    }
}
