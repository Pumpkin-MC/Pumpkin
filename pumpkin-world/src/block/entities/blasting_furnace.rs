use crate::inventory::Inventory;
use pumpkin_data::block_properties::BlockProperties;

use std::{
    array::from_fn,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU16, Ordering},
    },
};

use pumpkin_data::recipes::CookingRecipeKind;
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;

use crate::{
    block::entities::furnace_like_block_entity::CookingBlockEntityBase,
    impl_block_entity_for_cooking, impl_clearable_for_cooking, impl_cooking_block_entity_base,
    impl_inventory_for_cooking, impl_property_delegate_for_cooking, item::ItemStack,
};

pub struct BlastingFurnaceBlockEntity {
    pub position: BlockPos,
    pub dirty: AtomicBool,

    pub cooking_time_spent: AtomicU16,
    pub cooking_total_time: AtomicU16,
    pub lit_time_remaining: AtomicU16,
    pub lit_total_time: AtomicU16,

    pub items: [Arc<Mutex<ItemStack>>; Self::INVENTORY_SIZE],
}

impl BlastingFurnaceBlockEntity {
    pub const INVENTORY_SIZE: usize = 3;
    pub const ID: &'static str = "minecraft:blast_furnace";

    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            dirty: AtomicBool::new(false),
            items: from_fn(|_| Arc::new(Mutex::new(ItemStack::EMPTY.clone()))),
            cooking_total_time: AtomicU16::new(0),
            cooking_time_spent: AtomicU16::new(0),
            lit_total_time: AtomicU16::new(0),
            lit_time_remaining: AtomicU16::new(0),
        }
    }
}

impl_cooking_block_entity_base!(BlastingFurnaceBlockEntity);
impl_block_entity_for_cooking!(BlastingFurnaceBlockEntity, CookingRecipeKind::Blasting);
impl_inventory_for_cooking!(BlastingFurnaceBlockEntity);
impl_clearable_for_cooking!(BlastingFurnaceBlockEntity);
impl_property_delegate_for_cooking!(BlastingFurnaceBlockEntity);
