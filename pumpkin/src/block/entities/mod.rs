use std::any::Any;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use barrel::BarrelBlockEntity;
use bed::BedBlockEntity;
use chest::ChestBlockEntity;
use chiseled_bookshelf::ChiseledBookshelfBlockEntity;
use comparator::ComparatorBlockEntity;
use dropper::DropperBlockEntity;
use end_portal::EndPortalBlockEntity;
use ender_chest::EnderChestBlockEntity;
use furnace::FurnaceBlockEntity;
use hopper::HopperBlockEntity;
use mob_spawner::MobSpawnerBlockEntity;
use piston::PistonBlockEntity;
use shulker_box::ShulkerBoxBlockEntity;
use sign::SignBlockEntity;

use pumpkin_data::{Block, block_properties::BLOCK_ENTITY_TYPES};
use pumpkin_inventory::screen_handler::PropertyDelegate;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entity::{BlockEntityCollection, BoxedFuture};
use pumpkin_world::inventory::Inventory;

use crate::world::World;

pub mod barrel;
pub mod bed;
pub mod chest;
pub mod chiseled_bookshelf;
pub mod command_block;
pub mod comparator;
pub mod dropper;
pub mod end_portal;
pub mod ender_chest;
pub mod furnace;
pub mod hopper;
pub mod mob_spawner;
pub mod piston;
pub mod shulker_box;
pub mod sign;

//TODO: We need a mark_dirty for chests
pub trait BlockEntity: Send + Sync {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized;
    fn tick<'a>(&'a self, _world: Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
    }
    fn resource_location(&self) -> &'static str;
    fn get_position(&self) -> BlockPos;
    fn write_internal<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            nbt.put_string("id", self.resource_location().to_string());
            let position = self.get_position();
            nbt.put_int("x", position.0.x);
            nbt.put_int("y", position.0.y);
            nbt.put_int("z", position.0.z);
            self.write_nbt(nbt).await;
        })
    }
    fn get_id(&self) -> u32 {
        pumpkin_data::block_properties::BLOCK_ENTITY_TYPES
            .iter()
            .position(|block_entity_name| {
                *block_entity_name == self.resource_location().split(':').next_back().unwrap()
            })
            .unwrap() as u32
    }
    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        None
    }
    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        None
    }
    fn set_block_state(&mut self, _block_state: BlockStateId) {}
    fn on_block_replaced<'a>(
        self: Arc<Self>,
        world: Arc<World>,
        position: BlockPos,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>
    where
        Self: 'a,
    {
        Box::pin(async move {
            if let Some(inventory) = self.get_inventory() {
                // Assuming scatter_inventory is an async method on SimpleWorld
                world.scatter_inventory(&position, &inventory).await;
            }
        })
    }
    fn is_dirty(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any;
    fn to_property_delegate(self: Arc<Self>) -> Option<Arc<dyn PropertyDelegate>> {
        None
    }
}

#[derive(Default)]
pub struct BlockEntityStorage {
    entities: HashMap<BlockPos, Arc<dyn BlockEntity>>,
}

impl BlockEntityStorage {
    pub async fn tick(&self, world: Arc<World>) {
        for block_entity in self.entities.values() {
            block_entity.tick(world.clone()).await;
        }
    }

    #[must_use]
    pub fn get(&self, block_pos: &BlockPos) -> Option<Arc<dyn BlockEntity>> {
        self.entities.get(block_pos).cloned()
    }

    pub fn insert(
        &mut self,
        block_pos: BlockPos,
        block_entity: Arc<dyn BlockEntity>,
    ) -> Option<Arc<dyn BlockEntity>> {
        self.entities.insert(block_pos, block_entity)
    }

    pub fn remove(&mut self, block_pos: &BlockPos) -> Option<Arc<dyn BlockEntity>> {
        self.entities.remove(block_pos)
    }
}

pub struct BlockEntityData(Arc<dyn BlockEntity>);

impl pumpkin_world::block::entity::BlockEntityData for BlockEntityData {
    fn get_position(&self) -> BlockPos {
        self.0.get_position()
    }
    fn get_id(&self) -> u32 {
        self.0.get_id()
    }
    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        self.0.chunk_data_nbt()
    }
}

impl BlockEntityCollection for BlockEntityStorage {
    type BlockEntity = BlockEntityData;
    fn from_nbt_entries(nbt_entries: &[NbtCompound]) -> Self {
        let mut entities = HashMap::new();
        for nbt in nbt_entries {
            let block_entity = block_entity_from_nbt(nbt);
            if let Some(block_entity) = block_entity {
                entities.insert(block_entity.get_position(), block_entity);
            }
        }
        Self { entities }
    }
    fn to_nbt_entries(&self) -> BoxedFuture<'_, Vec<NbtCompound>> {
        Box::pin(async move {
            futures::future::join_all(self.entities.values().map(|block_entity| async move {
                let mut nbt = NbtCompound::new();
                block_entity.write_internal(&mut nbt).await;
                nbt
            }))
            .await
        })
    }
    fn len(&self) -> usize {
        self.entities.len()
    }
    fn get_all(&self) -> Vec<Self::BlockEntity> {
        self.entities
            .values()
            .map(|be| BlockEntityData(be.clone()))
            .collect()
    }
}

#[must_use]
pub fn block_entity_from_generic<T: BlockEntity>(nbt: &NbtCompound) -> T {
    let x = nbt.get_int("x").unwrap();
    let y = nbt.get_int("y").unwrap();
    let z = nbt.get_int("z").unwrap();
    T::from_nbt(nbt, BlockPos::new(x, y, z))
}

#[must_use]
pub fn block_entity_from_nbt(nbt: &NbtCompound) -> Option<Arc<dyn BlockEntity>> {
    Some(match nbt.get_string("id").unwrap() {
        ChestBlockEntity::ID => Arc::new(block_entity_from_generic::<ChestBlockEntity>(nbt)),
        EnderChestBlockEntity::ID => {
            Arc::new(block_entity_from_generic::<EnderChestBlockEntity>(nbt))
        }
        SignBlockEntity::ID => Arc::new(block_entity_from_generic::<SignBlockEntity>(nbt)),
        BedBlockEntity::ID => Arc::new(block_entity_from_generic::<BedBlockEntity>(nbt)),
        ComparatorBlockEntity::ID => {
            Arc::new(block_entity_from_generic::<ComparatorBlockEntity>(nbt))
        }
        BarrelBlockEntity::ID => Arc::new(block_entity_from_generic::<BarrelBlockEntity>(nbt)),
        HopperBlockEntity::ID => Arc::new(block_entity_from_generic::<HopperBlockEntity>(nbt)),
        MobSpawnerBlockEntity::ID => {
            Arc::new(block_entity_from_generic::<MobSpawnerBlockEntity>(nbt))
        }
        DropperBlockEntity::ID => Arc::new(block_entity_from_generic::<DropperBlockEntity>(nbt)),
        ShulkerBoxBlockEntity::ID => {
            Arc::new(block_entity_from_generic::<ShulkerBoxBlockEntity>(nbt))
        }
        PistonBlockEntity::ID => Arc::new(block_entity_from_generic::<PistonBlockEntity>(nbt)),
        EndPortalBlockEntity::ID => {
            Arc::new(block_entity_from_generic::<EndPortalBlockEntity>(nbt))
        }
        ChiseledBookshelfBlockEntity::ID => Arc::new(block_entity_from_generic::<
            ChiseledBookshelfBlockEntity,
        >(nbt)),
        FurnaceBlockEntity::ID => Arc::new(block_entity_from_generic::<FurnaceBlockEntity>(nbt)),
        _ => return None,
    })
}

#[must_use]
pub fn has_block_block_entity(block: &Block) -> bool {
    BLOCK_ENTITY_TYPES.contains(&block.name)
}
