use std::{
    hash::{DefaultHasher, Hash, Hasher},
    sync::Arc,
};

use async_trait::async_trait;
use pumpkin_world::item::ItemStack;
use tokio::sync::Mutex;

use crate::screen_handler::{DefaultScreenHandlerBehaviour, ScreenHandler};

pub struct SyncHandler {}
#[async_trait]
pub trait TrackedSlot: Send + Sync {
    async fn set_recived_hash(&self, hash: u64) {}
    async fn set_recived_stack(&self, stack: ItemStack) {}
    async fn is_in_sync(&self, actual_stack: ItemStack) -> bool {
        true
    }
}

pub struct NormalTrackedSlot {
    recived_hash: Mutex<Option<u64>>,
    recived_stack: Mutex<Option<Arc<Mutex<ItemStack>>>>,
}

impl SyncHandler {
    pub fn update_state(
        &self,
        screen_handler: &DefaultScreenHandlerBehaviour,
        stacks: &Vec<ItemStack>,
        cursor_stack: &ItemStack,
        properties: Vec<i32>,
    ) {
    }

    pub fn update_slot(
        &self,
        screen_handler: &DefaultScreenHandlerBehaviour,
        slot: usize,
        stack: &ItemStack,
    ) {
    }

    pub fn update_cursor_stack(
        &self,
        screen_handler: &DefaultScreenHandlerBehaviour,
        stack: &ItemStack,
    ) {
    }

    pub fn update_property(
        &self,
        screen_handler: &DefaultScreenHandlerBehaviour,
        property: i32,
        value: i32,
    ) {
    }

    pub fn send_property_update(
        &self,
        screen_handler: &DefaultScreenHandlerBehaviour,
        property: i32,
        value: i32,
    ) {
    }

    pub fn create_tracked_slot(&self) -> Arc<dyn TrackedSlot> {
        Arc::new(NormalTrackedSlot::new())
    }
}

pub struct AlwaysInSyncTrackedSlot {}

impl AlwaysInSyncTrackedSlot {
    pub const ALWAYS_IN_SYNC_TRACKED_SLOT: AlwaysInSyncTrackedSlot = AlwaysInSyncTrackedSlot {};
}

#[async_trait]
impl TrackedSlot for AlwaysInSyncTrackedSlot {}

impl NormalTrackedSlot {
    pub fn new() -> Self {
        NormalTrackedSlot {
            recived_hash: Mutex::new(None),
            recived_stack: Mutex::new(None),
        }
    }
}

#[async_trait]
impl TrackedSlot for NormalTrackedSlot {
    async fn set_recived_hash(&self, hash: u64) {
        *self.recived_stack.lock().await = None;
        *self.recived_hash.lock().await = Some(hash);
    }

    async fn set_recived_stack(&self, stack: ItemStack) {
        *self.recived_hash.lock().await = None;
        *self.recived_stack.lock().await = Some(Arc::new(Mutex::new(stack)));
    }

    async fn is_in_sync(&self, actual_stack: ItemStack) -> bool {
        if let Some(received_stack) = self.recived_stack.lock().await.as_ref() {
            return received_stack
                .lock()
                .await
                .are_items_and_components_equal(&actual_stack);
        } else if let Some(received_hash) = self.recived_hash.lock().await.as_ref() {
            let hash_equal = calculate_hash(&actual_stack) == *received_hash;
            if hash_equal {
                *self.recived_stack.lock().await = Some(Arc::new(Mutex::new(actual_stack)));
            }
            return hash_equal;
        } else {
            return false;
        }
    }
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}
