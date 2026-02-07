use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;

use crate::inventory::{Clearable, Inventory, InventoryFuture};
use crate::world::SimpleWorld;
use crate::{block::entities::BlockEntity, item::ItemStack};

/// Matches vanilla's JukeboxBlockEntity
pub struct JukeboxBlockEntity {
    position: BlockPos,
    /// The record item stored in the jukebox (RecordItem in NBT)
    record_stack: Arc<Mutex<ItemStack>>,
    /// Ticks since the current song started playing
    ticks_since_song_started: AtomicU64,
    /// Length of the current song in ticks (0 if not playing)
    song_length_ticks: AtomicU64,
    dirty: AtomicBool,
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

        let ticks_since_song_started =
            nbt.get_long(TICKS_SINCE_SONG_STARTED_NBT_KEY).unwrap_or(0) as u64;

        Self {
            position,
            record_stack: Arc::new(Mutex::new(record_stack)),
            ticks_since_song_started: AtomicU64::new(ticks_since_song_started),
            song_length_ticks: AtomicU64::new(0), // Will be set when playing starts
            dirty: AtomicBool::new(false),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            let record = self.record_stack.lock().await;
            if !record.is_empty() {
                let mut record_nbt = NbtCompound::new();
                record.write_item_stack(&mut record_nbt);
                nbt.put(RECORD_ITEM_NBT_KEY, record_nbt);
            }

            let ticks = self.ticks_since_song_started.load(Ordering::Relaxed);
            if ticks > 0 {
                nbt.put_long(TICKS_SINCE_SONG_STARTED_NBT_KEY, ticks as i64);
            }
        })
    }

    fn tick<'a>(
        &'a self,
        _world: &'a Arc<dyn SimpleWorld>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
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
        })
    }

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
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
        }
    }

    /// Get the current record stack
    pub async fn get_record(&self) -> ItemStack {
        self.record_stack.lock().await.clone()
    }

    /// Set the record stack - matches vanilla's setStack()
    /// Note: The caller is responsible for updating block state and playing music
    pub async fn set_record(&self, stack: ItemStack) {
        *self.record_stack.lock().await = stack;
        self.mark_dirty();
    }

    /// Clear the stack and return what was there - used for dropping
    pub async fn clear_record(&self) -> ItemStack {
        self.stop_playing();
        let mut record = self.record_stack.lock().await;
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
    }
}

/// Implements single-slot inventory for jukebox (matches vanilla's SingleStackInventory)
impl Inventory for JukeboxBlockEntity {
    fn size(&self) -> usize {
        1
    }

    fn is_empty(&self) -> InventoryFuture<'_, bool> {
        Box::pin(async move { self.record_stack.lock().await.is_empty() })
    }

    fn get_stack(&self, _slot: usize) -> InventoryFuture<'_, Arc<Mutex<ItemStack>>> {
        Box::pin(async move { self.record_stack.clone() })
    }

    fn remove_stack(&self, _slot: usize) -> InventoryFuture<'_, ItemStack> {
        Box::pin(async move {
            self.stop_playing();
            let mut record = self.record_stack.lock().await;
            let taken = record.clone();
            *record = ItemStack::EMPTY.clone();
            self.mark_dirty();
            taken
        })
    }

    fn remove_stack_specific(&self, _slot: usize, _amount: u8) -> InventoryFuture<'_, ItemStack> {
        // Jukebox only holds one item, so remove the whole stack
        self.remove_stack(0)
    }

    fn set_stack(&self, _slot: usize, stack: ItemStack) -> InventoryFuture<'_, ()> {
        Box::pin(async move {
            *self.record_stack.lock().await = stack;
            self.mark_dirty();
        })
    }

    fn mark_dirty(&self) {
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clearable for JukeboxBlockEntity {
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.stop_playing();
            *self.record_stack.lock().await = ItemStack::EMPTY.clone();
            self.mark_dirty();
        })
    }
}
