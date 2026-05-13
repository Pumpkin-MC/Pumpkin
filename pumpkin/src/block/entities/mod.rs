use std::pin::Pin;
use std::{any::Any, sync::Arc};

use pumpkin_data::{Block, block_properties::BLOCK_ENTITY_TYPES};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use crate::world::World;
use pumpkin_world::BlockStateId;
use pumpkin_world::inventory::Inventory;

pub mod barrel;
pub mod beacon;
pub mod bed;
pub mod bell;
pub mod blasting_furnace;
pub mod brewing_stand;
pub mod chest;
pub mod chest_like_block_entity;
pub mod chiseled_bookshelf;
pub mod command_block;
pub mod comparator;
pub mod daylight_detector;
pub mod dropper;
pub mod end_portal;
pub mod ender_chest;
pub mod furnace;
pub mod furnace_like_block_entity;
pub mod hopper;
pub mod jukebox;
pub mod lectern;
pub mod mob_spawner;
pub mod piston;
pub mod shulker_box;
pub mod sign;
pub mod smoker;
pub mod trapped_chest;

pub use furnace_like_block_entity::ExperienceContainer;
pub use pumpkin_world::block::entities::PropertyDelegate;

//TODO: We need a mark_dirty for chests
pub trait BlockEntity: Any + Send + Sync {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized;
    fn tick<'a>(&'a self, _world: &'a Arc<World>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
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

    /// Obtain NBT data for sending to the client in [`ChunkData`](crate::chunk::ChunkData)
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
                // Assuming scatter_inventory is an async method on World
                world.scatter_inventory(&position, &inventory).await;
            }
        })
    }
    fn is_dirty(&self) -> bool {
        false
    }

    fn clear_dirty(&self) {
        // Default implementation does nothing
        // Override in implementations that have a dirty flag
    }

    fn as_any(&self) -> &dyn Any;
    fn to_property_delegate(self: Arc<Self>) -> Option<Arc<dyn PropertyDelegate>> {
        None
    }
    fn to_experience_container(self: Arc<Self>) -> Option<Arc<dyn ExperienceContainer>> {
        None
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
    let id = nbt.get_string("id")?;
    let x = nbt.get_int("x")?;
    let y = nbt.get_int("y")?;
    let z = nbt.get_int("z")?;
    let pos = BlockPos::new(x, y, z);
    match id {
        barrel::BarrelBlockEntity::ID => {
            Some(Arc::new(barrel::BarrelBlockEntity::from_nbt(nbt, pos)))
        }
        chest::ChestBlockEntity::ID => Some(Arc::new(chest::ChestBlockEntity::from_nbt(nbt, pos))),
        trapped_chest::TrappedChestBlockEntity::ID => Some(Arc::new(
            trapped_chest::TrappedChestBlockEntity::from_nbt(nbt, pos),
        )),
        ender_chest::EnderChestBlockEntity::ID => Some(Arc::new(
            ender_chest::EnderChestBlockEntity::from_nbt(nbt, pos),
        )),
        furnace::FurnaceBlockEntity::ID => {
            Some(Arc::new(furnace::FurnaceBlockEntity::from_nbt(nbt, pos)))
        }
        blasting_furnace::BlastingFurnaceBlockEntity::ID => Some(Arc::new(
            blasting_furnace::BlastingFurnaceBlockEntity::from_nbt(nbt, pos),
        )),
        smoker::SmokerBlockEntity::ID => {
            Some(Arc::new(smoker::SmokerBlockEntity::from_nbt(nbt, pos)))
        }
        brewing_stand::BrewingStandBlockEntity::ID => Some(Arc::new(
            brewing_stand::BrewingStandBlockEntity::from_nbt(nbt, pos),
        )),
        hopper::HopperBlockEntity::ID => {
            Some(Arc::new(hopper::HopperBlockEntity::from_nbt(nbt, pos)))
        }
        jukebox::JukeboxBlockEntity::ID => {
            Some(Arc::new(jukebox::JukeboxBlockEntity::from_nbt(nbt, pos)))
        }
        mob_spawner::MobSpawnerBlockEntity::ID => Some(Arc::new(
            mob_spawner::MobSpawnerBlockEntity::from_nbt(nbt, pos),
        )),
        sign::SignBlockEntity::ID => Some(Arc::new(sign::SignBlockEntity::from_nbt(nbt, pos))),
        piston::PistonBlockEntity::ID => {
            Some(Arc::new(piston::PistonBlockEntity::from_nbt(nbt, pos)))
        }
        chiseled_bookshelf::ChiseledBookshelfBlockEntity::ID => Some(Arc::new(
            chiseled_bookshelf::ChiseledBookshelfBlockEntity::from_nbt(nbt, pos),
        )),
        dropper::DropperBlockEntity::ID => {
            Some(Arc::new(dropper::DropperBlockEntity::from_nbt(nbt, pos)))
        }
        command_block::CommandBlockEntity::ID => Some(Arc::new(
            command_block::CommandBlockEntity::from_nbt(nbt, pos),
        )),
        comparator::ComparatorBlockEntity::ID => Some(Arc::new(
            comparator::ComparatorBlockEntity::from_nbt(nbt, pos),
        )),
        daylight_detector::DaylightDetectorBlockEntity::ID => Some(Arc::new(
            daylight_detector::DaylightDetectorBlockEntity::from_nbt(nbt, pos),
        )),
        end_portal::EndPortalBlockEntity::ID => Some(Arc::new(
            end_portal::EndPortalBlockEntity::from_nbt(nbt, pos),
        )),
        beacon::BeaconBlockEntity::ID => {
            Some(Arc::new(beacon::BeaconBlockEntity::from_nbt(nbt, pos)))
        }
        bed::BedBlockEntity::ID => Some(Arc::new(bed::BedBlockEntity::from_nbt(nbt, pos))),
        bell::BellBlockEntity::ID => Some(Arc::new(bell::BellBlockEntity::from_nbt(nbt, pos))),
        shulker_box::ShulkerBoxBlockEntity::ID => Some(Arc::new(
            shulker_box::ShulkerBoxBlockEntity::from_nbt(nbt, pos),
        )),
        lectern::LecternBlockEntity::ID => {
            Some(Arc::new(lectern::LecternBlockEntity::from_nbt(nbt, pos)))
        }
        _ => None,
    }
}

#[must_use]
pub fn has_block_block_entity(block: &Block) -> bool {
    BLOCK_ENTITY_TYPES.contains(&block.name)
}
