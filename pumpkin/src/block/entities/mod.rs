use std::pin::Pin;
use std::{any::Any, sync::Arc};

use pumpkin_data::{Block, block_properties::BLOCK_ENTITY_TYPES};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use crate::world::World;
use pumpkin_data::BlockStateId;
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
pub mod jigsaw_block;
pub mod jukebox;
pub mod lectern;
pub mod mob_spawner;
pub mod piston;
pub mod shulker_box;
pub mod sign;
pub mod smoker;
pub mod trapped_chest;

pub mod banner;
pub mod beehive;
pub mod brushable_block;
pub mod calibrated_sculk_sensor;
pub mod campfire;
pub mod conduit;
pub mod copper_golem_statue;
pub mod crafter;
pub mod creaking_heart;
pub mod decorated_pot;
pub mod dispenser;
pub mod enchanting_table;
pub mod end_gateway;
pub mod hanging_sign;
pub mod potent_sulfur;
pub mod sculk_catalyst;
pub mod sculk_sensor;
pub mod sculk_shrieker;
pub mod shelf;
pub mod skull;
pub mod structure_block;
pub mod test_block;
pub mod test_instance_block;
pub mod trial_spawner;
pub mod vault;

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

    /// Atomically takes the pending loot-table key and seed from this block entity.
    ///
    /// Returns `Some((key, seed))` if a deferred loot table was set, clearing it in the
    /// process. Returns `None` for entities that do not support loot tables, or if the
    /// loot has already been generated.
    fn take_loot_table(&self) -> Option<(String, i64)> {
        None
    }

    /// Returns `true` if this block entity has a pending deferred loot table that has
    /// not yet been unpacked. Does not consume the loot table.
    fn has_loot_table(&self) -> bool {
        false
    }

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
                *block_entity_name
                    == self
                        .resource_location()
                        .split(':')
                        .next_back()
                        .expect("Resource location should have a name")
            })
            .expect("Block entity type should be registered") as u32
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
    let x = nbt.get_int("x").expect("NBT should have x coordinate");
    let y = nbt.get_int("y").expect("NBT should have y coordinate");
    let z = nbt.get_int("z").expect("NBT should have z coordinate");
    T::from_nbt(nbt, BlockPos::new(x, y, z))
}

#[must_use]
#[allow(clippy::too_many_lines)]
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
        jigsaw_block::JigsawBlockEntity::ID => Some(Arc::new(
            jigsaw_block::JigsawBlockEntity::from_nbt(nbt, pos),
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
        dispenser::DispenserBlockEntity::ID => Some(Arc::new(
            dispenser::DispenserBlockEntity::from_nbt(nbt, pos),
        )),
        hanging_sign::HangingSignBlockEntity::ID => Some(Arc::new(
            hanging_sign::HangingSignBlockEntity::from_nbt(nbt, pos),
        )),
        creaking_heart::CreakingHeartBlockEntity::ID => Some(Arc::new(
            creaking_heart::CreakingHeartBlockEntity::from_nbt(nbt, pos),
        )),
        enchanting_table::EnchantingTableBlockEntity::ID => Some(Arc::new(
            enchanting_table::EnchantingTableBlockEntity::from_nbt(nbt, pos),
        )),
        skull::SkullBlockEntity::ID => Some(Arc::new(skull::SkullBlockEntity::from_nbt(nbt, pos))),
        banner::BannerBlockEntity::ID => {
            Some(Arc::new(banner::BannerBlockEntity::from_nbt(nbt, pos)))
        }
        structure_block::StructureBlockBlockEntity::ID => Some(Arc::new(
            structure_block::StructureBlockBlockEntity::from_nbt(nbt, pos),
        )),
        end_gateway::EndGatewayBlockEntity::ID => Some(Arc::new(
            end_gateway::EndGatewayBlockEntity::from_nbt(nbt, pos),
        )),
        conduit::ConduitBlockEntity::ID => {
            Some(Arc::new(conduit::ConduitBlockEntity::from_nbt(nbt, pos)))
        }
        campfire::CampfireBlockEntity::ID => {
            Some(Arc::new(campfire::CampfireBlockEntity::from_nbt(nbt, pos)))
        }
        beehive::BeehiveBlockEntity::ID => {
            Some(Arc::new(beehive::BeehiveBlockEntity::from_nbt(nbt, pos)))
        }
        sculk_sensor::SculkSensorBlockEntity::ID => Some(Arc::new(
            sculk_sensor::SculkSensorBlockEntity::from_nbt(nbt, pos),
        )),
        calibrated_sculk_sensor::CalibratedSculkSensorBlockEntity::ID => Some(Arc::new(
            calibrated_sculk_sensor::CalibratedSculkSensorBlockEntity::from_nbt(nbt, pos),
        )),
        sculk_catalyst::SculkCatalystBlockEntity::ID => Some(Arc::new(
            sculk_catalyst::SculkCatalystBlockEntity::from_nbt(nbt, pos),
        )),
        sculk_shrieker::SculkShriekerBlockEntity::ID => Some(Arc::new(
            sculk_shrieker::SculkShriekerBlockEntity::from_nbt(nbt, pos),
        )),
        shelf::ShelfBlockEntity::ID => Some(Arc::new(shelf::ShelfBlockEntity::from_nbt(nbt, pos))),
        brushable_block::BrushableBlockBlockEntity::ID => Some(Arc::new(
            brushable_block::BrushableBlockBlockEntity::from_nbt(nbt, pos),
        )),
        decorated_pot::DecoratedPotBlockEntity::ID => Some(Arc::new(
            decorated_pot::DecoratedPotBlockEntity::from_nbt(nbt, pos),
        )),
        crafter::CrafterBlockEntity::ID => {
            Some(Arc::new(crafter::CrafterBlockEntity::from_nbt(nbt, pos)))
        }
        trial_spawner::TrialSpawnerBlockEntity::ID => Some(Arc::new(
            trial_spawner::TrialSpawnerBlockEntity::from_nbt(nbt, pos),
        )),
        vault::VaultBlockEntity::ID => Some(Arc::new(vault::VaultBlockEntity::from_nbt(nbt, pos))),
        test_block::TestBlockBlockEntity::ID => Some(Arc::new(
            test_block::TestBlockBlockEntity::from_nbt(nbt, pos),
        )),
        test_instance_block::TestInstanceBlockBlockEntity::ID => Some(Arc::new(
            test_instance_block::TestInstanceBlockBlockEntity::from_nbt(nbt, pos),
        )),
        copper_golem_statue::CopperGolemStatueBlockEntity::ID => Some(Arc::new(
            copper_golem_statue::CopperGolemStatueBlockEntity::from_nbt(nbt, pos),
        )),
        potent_sulfur::PotentSulfurBlockEntity::ID => Some(Arc::new(
            potent_sulfur::PotentSulfurBlockEntity::from_nbt(nbt, pos),
        )),
        _ => None,
    }
}

#[must_use]
pub fn has_block_block_entity(block: &Block) -> bool {
    BLOCK_ENTITY_TYPES.contains(&block.name)
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn create_block_entity(
    block_entity_type_id: u16,
    position: BlockPos,
) -> Option<Arc<dyn BlockEntity>> {
    use pumpkin_data::block_properties::FacingHopper;
    if block_entity_type_id == u16::MAX {
        return None;
    }
    let name = BLOCK_ENTITY_TYPES.get(block_entity_type_id as usize)?;
    match *name {
        "furnace" => Some(Arc::new(furnace::FurnaceBlockEntity::new(position))),
        "chest" => Some(Arc::new(chest::ChestBlockEntity::new(position))),
        "trapped_chest" => Some(Arc::new(trapped_chest::TrappedChestBlockEntity::new(
            position,
        ))),
        "ender_chest" => Some(Arc::new(ender_chest::EnderChestBlockEntity::new(position))),
        "jukebox" => Some(Arc::new(jukebox::JukeboxBlockEntity::new(position))),
        "dispenser" => Some(Arc::new(dispenser::DispenserBlockEntity::new(position))),
        "dropper" => Some(Arc::new(dropper::DropperBlockEntity::new(position))),
        "sign" => Some(Arc::new(sign::SignBlockEntity::empty(position))),
        "hanging_sign" => Some(Arc::new(hanging_sign::HangingSignBlockEntity::empty(
            position,
        ))),
        "mob_spawner" => Some(Arc::new(mob_spawner::MobSpawnerBlockEntity::new(
            position, None,
        ))),
        "creaking_heart" => Some(Arc::new(creaking_heart::CreakingHeartBlockEntity::new(
            position,
        ))),
        "brewing_stand" => Some(Arc::new(brewing_stand::BrewingStandBlockEntity::new(
            position,
        ))),
        "enchanting_table" => Some(Arc::new(enchanting_table::EnchantingTableBlockEntity::new(
            position,
        ))),
        "end_portal" => Some(Arc::new(end_portal::EndPortalBlockEntity::new(position))),
        "beacon" => Some(Arc::new(beacon::BeaconBlockEntity::new(position))),
        "skull" => Some(Arc::new(skull::SkullBlockEntity::new(position))),
        "daylight_detector" => Some(Arc::new(
            daylight_detector::DaylightDetectorBlockEntity::new(position),
        )),
        "hopper" => Some(Arc::new(hopper::HopperBlockEntity::new(
            position,
            FacingHopper::Down,
        ))),
        "comparator" => Some(Arc::new(comparator::ComparatorBlockEntity::new(position))),
        "banner" => Some(Arc::new(banner::BannerBlockEntity::new(position))),
        "structure_block" => Some(Arc::new(structure_block::StructureBlockBlockEntity::new(
            position,
        ))),
        "end_gateway" => Some(Arc::new(end_gateway::EndGatewayBlockEntity::new(position))),
        "command_block" => Some(Arc::new(command_block::CommandBlockEntity::new(
            position, true, false,
        ))),
        "shulker_box" => Some(Arc::new(shulker_box::ShulkerBoxBlockEntity::new(position))),
        "conduit" => Some(Arc::new(conduit::ConduitBlockEntity::new(position))),
        "barrel" => Some(Arc::new(barrel::BarrelBlockEntity::new(position))),
        "smoker" => Some(Arc::new(smoker::SmokerBlockEntity::new(position))),
        "blast_furnace" => Some(Arc::new(blasting_furnace::BlastingFurnaceBlockEntity::new(
            position,
        ))),
        "lectern" => Some(Arc::new(lectern::LecternBlockEntity::new(position))),
        "bell" => Some(Arc::new(bell::BellBlockEntity::new(position))),
        "jigsaw" => Some(Arc::new(jigsaw_block::JigsawBlockEntity::new(position))),
        "campfire" => Some(Arc::new(campfire::CampfireBlockEntity::new(position))),
        "beehive" => Some(Arc::new(beehive::BeehiveBlockEntity::new(position))),
        "sculk_sensor" => Some(Arc::new(sculk_sensor::SculkSensorBlockEntity::new(
            position,
        ))),
        "calibrated_sculk_sensor" => Some(Arc::new(
            calibrated_sculk_sensor::CalibratedSculkSensorBlockEntity::new(position),
        )),
        "sculk_catalyst" => Some(Arc::new(sculk_catalyst::SculkCatalystBlockEntity::new(
            position,
        ))),
        "sculk_shrieker" => Some(Arc::new(sculk_shrieker::SculkShriekerBlockEntity::new(
            position,
        ))),
        "chiseled_bookshelf" => Some(Arc::new(
            chiseled_bookshelf::ChiseledBookshelfBlockEntity::new(position),
        )),
        "shelf" => Some(Arc::new(shelf::ShelfBlockEntity::new(position))),
        "brushable_block" => Some(Arc::new(brushable_block::BrushableBlockBlockEntity::new(
            position,
        ))),
        "decorated_pot" => Some(Arc::new(decorated_pot::DecoratedPotBlockEntity::new(
            position,
        ))),
        "crafter" => Some(Arc::new(crafter::CrafterBlockEntity::new(position))),
        "trial_spawner" => Some(Arc::new(trial_spawner::TrialSpawnerBlockEntity::new(
            position,
        ))),
        "vault" => Some(Arc::new(vault::VaultBlockEntity::new(position))),
        "test_block" => Some(Arc::new(test_block::TestBlockBlockEntity::new(position))),
        "test_instance_block" => Some(Arc::new(
            test_instance_block::TestInstanceBlockBlockEntity::new(position),
        )),
        "copper_golem_statue" => Some(Arc::new(
            copper_golem_statue::CopperGolemStatueBlockEntity::new(position),
        )),
        "potent_sulfur" => Some(Arc::new(potent_sulfur::PotentSulfurBlockEntity::new(
            position,
        ))),
        _ => None,
    }
}
