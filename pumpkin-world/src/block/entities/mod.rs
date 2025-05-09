use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use barrel::BarrelBlockEntity;
use bed::BedBlockEntity;
use chest::ChestBlockEntity;
use comparator::ComparatorBlockEntity;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use sign::SignBlockEntity;

use crate::inventory::Inventory;

pub mod barrel;
pub mod bed;
pub mod chest;
pub mod comparator;
pub mod sign;

//TODO: We need a mark_dirty for chests
#[async_trait]
pub trait BlockEntity: Send + Sync {
    async fn write_nbt(&self, nbt: &mut NbtCompound);
    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized;
    fn identifier(&self) -> &'static str;
    fn get_position(&self) -> BlockPos;
    async fn write_internal(&self, nbt: &mut NbtCompound) {
        nbt.put_string("id", self.identifier().to_string());
        let position = self.get_position();
        nbt.put_int("x", position.0.x);
        nbt.put_int("y", position.0.y);
        nbt.put_int("z", position.0.z);
        self.write_nbt(nbt).await;
    }
    fn get_id(&self) -> u32 {
        pumpkin_data::block_properties::BLOCK_ENTITY_TYPES
            .iter()
            .position(|block_entity_name| {
                *block_entity_name == self.identifier().split(":").last().unwrap()
            })
            .unwrap() as u32
    }
    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        None
    }
    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        None
    }
    fn is_dirty(&self) -> bool {
        false
    }
    fn as_any(&self) -> &dyn Any;
}

pub fn block_entity_from_generic<T: BlockEntity>(nbt: &NbtCompound) -> T {
    let x = nbt.get_int("x").unwrap();
    let y = nbt.get_int("y").unwrap();
    let z = nbt.get_int("z").unwrap();
    T::from_nbt(nbt, BlockPos::new(x, y, z))
}

pub fn block_entity_from_nbt(nbt: &NbtCompound) -> Option<Arc<dyn BlockEntity>> {
    let id = nbt.get_string("id").unwrap();
    match id.as_str() {
        ChestBlockEntity::ID => Some(Arc::new(block_entity_from_generic::<ChestBlockEntity>(nbt))),
        SignBlockEntity::ID => Some(Arc::new(block_entity_from_generic::<SignBlockEntity>(nbt))),
        BedBlockEntity::ID => Some(Arc::new(block_entity_from_generic::<BedBlockEntity>(nbt))),
        ComparatorBlockEntity::ID => Some(Arc::new(block_entity_from_generic::<
            ComparatorBlockEntity,
        >(nbt))),
        BarrelBlockEntity::ID => Some(Arc::new(block_entity_from_generic::<BarrelBlockEntity>(
            nbt,
        ))),
        _ => None,
    }
}
