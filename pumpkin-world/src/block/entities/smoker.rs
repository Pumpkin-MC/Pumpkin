use crate::inventory::Inventory;
use pumpkin_data::block_properties::BlockProperties;

use std::{
    array::from_fn,
    collections::HashMap,
    sync::{
        Arc, Mutex as StdMutex,
        atomic::{AtomicBool, AtomicU16, Ordering},
    },
};

use pumpkin_data::recipes::CookingRecipeKind;
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;

use crate::{
    block::entities::furnace_like_block_entity::CookingBlockEntityBase,
    impl_block_entity_for_cooking, impl_clearable_for_cooking, impl_cooking_block_entity_base,
    impl_experience_container_for_cooking, impl_inventory_for_cooking,
    impl_property_delegate_for_cooking, item::ItemStack,
};

pub struct SmokerBlockEntity {
    pub position: BlockPos,
    pub dirty: AtomicBool,

    pub cooking_time_spent: AtomicU16,
    pub cooking_total_time: AtomicU16,
    pub lit_time_remaining: AtomicU16,
    pub lit_total_time: AtomicU16,

    pub items: [Arc<Mutex<ItemStack>>; Self::INVENTORY_SIZE],

    /// Tracks recipes used for XP calculation (vanilla `RecipesUsed` NBT format)
    /// Maps result item ID -> craft count
    pub recipes_used: StdMutex<HashMap<String, u32>>,
}

impl SmokerBlockEntity {
    pub const INVENTORY_SIZE: usize = 3;
    pub const ID: &'static str = "minecraft:smoker";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            dirty: AtomicBool::new(false),
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            cooking_total_time: AtomicU16::new(0),
            cooking_time_spent: AtomicU16::new(0),
            lit_total_time: AtomicU16::new(0),
            lit_time_remaining: AtomicU16::new(0),
            recipes_used: StdMutex::new(HashMap::new()),
        }
    }
}

impl_cooking_block_entity_base!(SmokerBlockEntity);
impl_block_entity_for_cooking!(SmokerBlockEntity, CookingRecipeKind::Smoking);
impl_inventory_for_cooking!(SmokerBlockEntity);
impl_clearable_for_cooking!(SmokerBlockEntity);
impl_property_delegate_for_cooking!(SmokerBlockEntity);
impl_experience_container_for_cooking!(SmokerBlockEntity);
