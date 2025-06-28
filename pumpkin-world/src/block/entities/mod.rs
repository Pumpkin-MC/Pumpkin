use std::{any::Any, sync::Arc};

use async_trait::async_trait;
use barrel::BarrelBlockEntity;
use bed::BedBlockEntity;
use chest::ChestBlockEntity;
use comparator::ComparatorBlockEntity;
use end_portal::EndPortalBlockEntity;
use piston::PistonBlockEntity;
use pumpkin_data::{Block, block_properties::BLOCK_ENTITY_TYPES};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use sign::SignBlockEntity;

use crate::{inventory::Inventory, world::SimpleWorld};

pub mod barrel;
pub mod bed;
pub mod chest;
pub mod command_block;
pub mod comparator;
pub mod end_portal;
pub mod piston;
pub mod sign;

//TODO: We need a mark_dirty for chests
#[async_trait]
pub trait BlockEntity: Send + Sync {
    async fn write_nbt(&self, nbt: &mut NbtCompound);
    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized;
    async fn tick(&self, _world: &Arc<dyn SimpleWorld>) {}
    fn resource_location(&self) -> &'static str;
    fn get_position(&self) -> BlockPos;
    async fn write_internal(&self, nbt: &mut NbtCompound) {
        nbt.put_string("id", self.resource_location().to_string());
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
                *block_entity_name == self.resource_location().split(":").last().unwrap()
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
        PistonBlockEntity::ID => Some(Arc::new(block_entity_from_generic::<PistonBlockEntity>(
            nbt,
        ))),
        EndPortalBlockEntity::ID => Some(Arc::new(
            block_entity_from_generic::<EndPortalBlockEntity>(nbt),
        )),
        _ => None,
    }
}

pub fn has_block_block_entity(block: &Block) -> bool {
    BLOCK_ENTITY_TYPES.contains(&block.name)
}
