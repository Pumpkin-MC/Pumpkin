use std::{any::Any, sync::Arc};

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::{Block, block_properties::BLOCK_ENTITY_TYPES};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use crate::world::World;
use pumpkin_data::BlockStateId;
use pumpkin_inventory::Inventory;

pub mod barrel;
pub mod beacon;
pub mod bed;
pub mod bedrock;
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
pub mod map;
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
pub use pumpkin_inventory::PropertyDelegate;

//TODO: We need a mark_dirty for chests
pub trait BlockEntity: Any + Send + Sync {
    fn write_nbt(&self, nbt: &mut NbtCompound);
    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized;
    fn tick(&self, _world: &Arc<World>) {}
    fn resource_location(&self) -> &'static str;
    fn get_position(&self) -> BlockPos;

    /// Clears the pending loot-table key and runs `generate` with it and the seed.
    /// Implementations reserve the one-time unpack under the loot lock, release both chest locks
    /// before generation, then place into still-empty slots.
    // TODO vanilla `RandomizableContainerBlockEntity`: barrels, shulker boxes, dispensers, droppers, hoppers,
    // crafters and decorated pots hold loot tables too. Only chests implement this.
    fn unpack_loot_table_with(&self, _generate: &mut dyn FnMut(String, i64) -> Vec<ItemStack>) {}

    /// Returns `true` if this block entity has a pending deferred loot table that has
    /// not yet been unpacked. Does not consume the loot table.
    fn has_loot_table(&self) -> bool {
        false
    }

    /// Vanilla `getLootTable`: the pending loot table key, without consuming it.
    fn loot_table_key(&self) -> Option<String> {
        None
    }

    fn write_internal(&self, nbt: &mut NbtCompound) {
        nbt.put_string("id", self.resource_location().to_string());
        let position = self.get_position();
        nbt.put_int("x", position.0.x);
        nbt.put_int("y", position.0.y);
        nbt.put_int("z", position.0.z);
        self.write_nbt(nbt);
    }
    fn get_id(&self) -> u32 {
        let name = self
            .resource_location()
            .split(':')
            .next_back()
            .unwrap_or("");
        pumpkin_data::block_properties::BLOCK_ENTITY_TYPES
            .iter()
            .position(|block_entity_name| *block_entity_name == name)
            .unwrap_or(0) as u32
    }

    /// Obtain the block entity's own NBT for persistence.
    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        None
    }

    /// Whether the saved tag is also the tag sent to clients.
    ///
    /// Container contents only reach the client when a window opens, so
    /// containers override this with `false`.
    #[must_use]
    fn sends_saved_update_tag() -> bool
    where
        Self: Sized,
    {
        true
    }

    /// Obtain block actor NBT for fields Bedrock does not include in its block state.
    fn bedrock_block_actor_data(&self, _state_id: BlockStateId) -> Option<NbtCompound> {
        None
    }

    fn get_inventory(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        None
    }

    /// Vanilla `Container`: the inventory hoppers, droppers and crafters move items through.
    ///
    /// Hoppers on parallel threads write through `Inventory::update_stack`, so the returned
    /// inventory must override it with one lock for read and write (`update_locked_stack`).
    fn get_container(self: Arc<Self>) -> Option<Arc<dyn Inventory>> {
        self.get_inventory()
    }

    /// Copies the block entity's state on the item stack dropped for it.
    fn collect_item_components(&self, _stack: &mut ItemStack) {}

    /// Copies all implicit components onto a Creative Ctrl+Pick Block item stack (`BlockEntity.collectComponents`).
    fn collect_pick_item_components(&self, stack: &mut ItemStack) {
        self.collect_item_components(stack);
    }

    /// Restores the state from the item stack the block was placed from.
    fn apply_item_components(&self, _stack: &ItemStack) {}

    fn get_lock(&self) -> Option<pumpkin_data::data_component_impl::LockImpl> {
        None
    }

    fn get_custom_name(&self) -> Option<pumpkin_util::text::TextComponent> {
        None
    }

    fn drops_for_creative_player(&self) -> bool {
        false
    }

    fn set_block_state(&mut self, _block_state: BlockStateId) {}

    fn set_world(&self, _world: std::sync::Weak<World>) {}

    /// Runs post-load side effects matching vanilla `BlockEntity.loadAdditional` when NBT is
    /// applied to a live block entity in a world.
    fn on_nbt_loaded(&self, _world: &Arc<World>, _previous: Option<&dyn BlockEntity>) {}

    fn on_block_replaced(self: Arc<Self>, world: &Arc<World>, position: &BlockPos) {
        if let Some(inventory) = self.get_inventory() {
            world.scatter_inventory(position, &inventory);
        }
    }

    fn on_player_destroyed(
        &self,
        _world: &Arc<World>,
        _state: BlockStateId,
        _player: &Arc<crate::entity::player::Player>,
        _tool: &ItemStack,
    ) {
    }
    fn is_dirty(&self) -> bool {
        false
    }

    fn clear_dirty(&self) {
        // Default implementation does nothing
        // Override in implementations that have a dirty flag
    }

    /// Same idea as [`Self::is_dirty`]/[`Self::clear_dirty`], tracked separately.
    fn is_comparator_dirty(&self) -> bool {
        false
    }

    fn clear_comparator_dirty(&self) {}

    fn as_any(&self) -> &dyn Any;
    fn to_property_delegate(self: Arc<Self>) -> Option<Arc<dyn PropertyDelegate>> {
        None
    }
    fn to_experience_container(self: Arc<Self>) -> Option<Arc<dyn ExperienceContainer>> {
        None
    }
}

/// Shared `BaseContainerBlockEntity` / `BeaconBlockEntity` `CustomName` + `lock` state (`26.3`).
#[derive(Default)]
pub struct ContainerMetadata {
    pub custom_name: std::sync::Mutex<Option<pumpkin_util::text::TextComponent>>,
    pub lock: std::sync::Mutex<Option<pumpkin_data::data_component_impl::LockImpl>>,
}

impl ContainerMetadata {
    #[must_use]
    pub fn from_nbt(nbt: &NbtCompound) -> Self {
        let custom_name = nbt
            .get("CustomName")
            .or_else(|| nbt.get("custom_name"))
            .and_then(pumpkin_data::data_component_impl::CustomNameImpl::read_data)
            .map(|c| c.name);
        let lock = nbt
            .get("lock")
            .or_else(|| nbt.get("Lock"))
            .and_then(pumpkin_data::data_component_impl::LockImpl::read_data);
        Self {
            custom_name: std::sync::Mutex::new(custom_name),
            lock: std::sync::Mutex::new(lock),
        }
    }

    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        use pumpkin_data::data_component_impl::{CustomNameImpl, DataComponentImpl};
        if let Some(name) = self
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
        {
            nbt.put("CustomName", CustomNameImpl { name }.write_data());
        }
        if let Some(lock) = self
            .lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
        {
            nbt.put("lock", lock.write_data());
        }
    }

    pub fn apply_item_components(&self, stack: &ItemStack) {
        if let Some(name) =
            stack.get_data_component::<pumpkin_data::data_component_impl::CustomNameImpl>()
        {
            *self
                .custom_name
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(name.name.clone());
        }
        if let Some(lock) =
            stack.get_data_component::<pumpkin_data::data_component_impl::LockImpl>()
        {
            *self
                .lock
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(lock.clone());
        }
    }

    pub fn collect_name_only(&self, stack: &mut ItemStack) {
        if let Some(name) = self
            .custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
        {
            stack.set_data_component(pumpkin_data::data_component_impl::CustomNameImpl { name });
        }
    }

    pub fn collect_item_components(&self, stack: &mut ItemStack) {
        self.collect_name_only(stack);
        if let Some(lock) = self
            .lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
        {
            stack.set_data_component(lock);
        }
    }

    #[must_use]
    pub fn get_lock(&self) -> Option<pumpkin_data::data_component_impl::LockImpl> {
        self.lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    #[must_use]
    pub fn get_custom_name(&self) -> Option<pumpkin_util::text::TextComponent> {
        self.custom_name
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

pub fn apply_container_items_to_slice(stack: &ItemStack, items: &mut [ItemStack]) -> bool {
    let Some(container) =
        stack.get_data_component::<pumpkin_data::data_component_impl::ContainerImpl>()
    else {
        return false;
    };
    for (slot, item) in &container.items {
        if let Some(target) = items.get_mut(*slot as usize) {
            *target = item.clone();
        }
    }
    true
}

pub fn collect_container_items_from_slice(items: &[ItemStack], stack: &mut ItemStack) {
    let contents: Vec<(u8, ItemStack)> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| !item.is_empty())
        .map(|(slot, item)| (slot as u8, item.clone()))
        .collect();
    if !contents.is_empty() {
        stack.set_data_component(pumpkin_data::data_component_impl::ContainerImpl {
            items: contents,
        });
    }
}

#[must_use]
pub fn block_entity_from_generic<T: BlockEntity>(nbt: &NbtCompound) -> T {
    let x = nbt.get_int("x").unwrap_or(0);
    let y = nbt.get_int("y").unwrap_or(0);
    let z = nbt.get_int("z").unwrap_or(0);
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
        map::MAP_BLOCK_ENTITY_ID => Some(Arc::new(map::MapBlockEntity::from_nbt(nbt, pos))),
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

macro_rules! block_entity_types {
    ($( $name:literal => $ty:ty => $factory:expr ),* $(,)?) => {
        #[must_use]
        #[allow(clippy::too_many_lines)]
        pub fn create_block_entity(
            block_entity_type_id: u16,
            position: BlockPos,
        ) -> Option<Arc<dyn BlockEntity>> {
            if block_entity_type_id == u16::MAX {
                return None;
            }
            let name = BLOCK_ENTITY_TYPES.get(usize::from(block_entity_type_id))?;
            match *name {
                $( $name => Some(Arc::new(($factory)(position))), )*
                _ => None,
            }
        }

        /// Returns whether a block entity type sends its saved tag to clients.
        ///
        /// The value comes from [`BlockEntity::sends_saved_update_tag`].
        #[must_use]
        pub fn block_entity_sends_saved_update_tag(block_entity_type_id: u16) -> bool {
            let Some(name) = BLOCK_ENTITY_TYPES.get(usize::from(block_entity_type_id)) else {
                return true;
            };
            match *name {
                $( $name => <$ty as BlockEntity>::sends_saved_update_tag(), )*
                _ => true,
            }
        }
    };
}

block_entity_types! {
    "furnace" => furnace::FurnaceBlockEntity => furnace::FurnaceBlockEntity::new,
    "chest" => chest::ChestBlockEntity => chest::ChestBlockEntity::new,
    "trapped_chest" => trapped_chest::TrappedChestBlockEntity => trapped_chest::TrappedChestBlockEntity::new,
    "ender_chest" => ender_chest::EnderChestBlockEntity => ender_chest::EnderChestBlockEntity::new,
    "jukebox" => jukebox::JukeboxBlockEntity => jukebox::JukeboxBlockEntity::new,
    "dispenser" => dispenser::DispenserBlockEntity => dispenser::DispenserBlockEntity::new,
    "dropper" => dropper::DropperBlockEntity => dropper::DropperBlockEntity::new,
    "sign" => sign::SignBlockEntity => sign::SignBlockEntity::empty,
    "hanging_sign" => hanging_sign::HangingSignBlockEntity => hanging_sign::HangingSignBlockEntity::empty,
    "mob_spawner" => mob_spawner::MobSpawnerBlockEntity => |position| mob_spawner::MobSpawnerBlockEntity::new(position, None),
    "creaking_heart" => creaking_heart::CreakingHeartBlockEntity => creaking_heart::CreakingHeartBlockEntity::new,
    "piston" => piston::PistonBlockEntity => |position| piston::PistonBlockEntity::from_nbt(&pumpkin_nbt::compound::NbtCompound::new(), position),
    "brewing_stand" => brewing_stand::BrewingStandBlockEntity => brewing_stand::BrewingStandBlockEntity::new,
    "enchanting_table" => enchanting_table::EnchantingTableBlockEntity => enchanting_table::EnchantingTableBlockEntity::new,
    "end_portal" => end_portal::EndPortalBlockEntity => end_portal::EndPortalBlockEntity::new,
    "beacon" => beacon::BeaconBlockEntity => beacon::BeaconBlockEntity::new,
    "skull" => skull::SkullBlockEntity => skull::SkullBlockEntity::new,
    "daylight_detector" => daylight_detector::DaylightDetectorBlockEntity => daylight_detector::DaylightDetectorBlockEntity::new,
    "hopper" => hopper::HopperBlockEntity => hopper::HopperBlockEntity::new,
    "comparator" => comparator::ComparatorBlockEntity => comparator::ComparatorBlockEntity::new,
    "banner" => banner::BannerBlockEntity => banner::BannerBlockEntity::new,
    "structure_block" => structure_block::StructureBlockBlockEntity => structure_block::StructureBlockBlockEntity::new,
    "end_gateway" => end_gateway::EndGatewayBlockEntity => end_gateway::EndGatewayBlockEntity::new,
    "command_block" => command_block::CommandBlockEntity => |position| command_block::CommandBlockEntity::new(position, true, false),
    "shulker_box" => shulker_box::ShulkerBoxBlockEntity => shulker_box::ShulkerBoxBlockEntity::new,
    "conduit" => conduit::ConduitBlockEntity => conduit::ConduitBlockEntity::new,
    "barrel" => barrel::BarrelBlockEntity => barrel::BarrelBlockEntity::new,
    "smoker" => smoker::SmokerBlockEntity => smoker::SmokerBlockEntity::new,
    "blast_furnace" => blasting_furnace::BlastingFurnaceBlockEntity => blasting_furnace::BlastingFurnaceBlockEntity::new,
    "lectern" => lectern::LecternBlockEntity => lectern::LecternBlockEntity::new,
    "bell" => bell::BellBlockEntity => bell::BellBlockEntity::new,
    "jigsaw" => jigsaw_block::JigsawBlockEntity => jigsaw_block::JigsawBlockEntity::new,
    "campfire" => campfire::CampfireBlockEntity => campfire::CampfireBlockEntity::new,
    "beehive" => beehive::BeehiveBlockEntity => beehive::BeehiveBlockEntity::new,
    "sculk_sensor" => sculk_sensor::SculkSensorBlockEntity => sculk_sensor::SculkSensorBlockEntity::new,
    "calibrated_sculk_sensor" => calibrated_sculk_sensor::CalibratedSculkSensorBlockEntity => calibrated_sculk_sensor::CalibratedSculkSensorBlockEntity::new,
    "sculk_catalyst" => sculk_catalyst::SculkCatalystBlockEntity => sculk_catalyst::SculkCatalystBlockEntity::new,
    "sculk_shrieker" => sculk_shrieker::SculkShriekerBlockEntity => sculk_shrieker::SculkShriekerBlockEntity::new,
    "chiseled_bookshelf" => chiseled_bookshelf::ChiseledBookshelfBlockEntity => chiseled_bookshelf::ChiseledBookshelfBlockEntity::new,
    "shelf" => shelf::ShelfBlockEntity => shelf::ShelfBlockEntity::new,
    "brushable_block" => brushable_block::BrushableBlockBlockEntity => brushable_block::BrushableBlockBlockEntity::new,
    "decorated_pot" => decorated_pot::DecoratedPotBlockEntity => decorated_pot::DecoratedPotBlockEntity::new,
    "crafter" => crafter::CrafterBlockEntity => crafter::CrafterBlockEntity::new,
    "trial_spawner" => trial_spawner::TrialSpawnerBlockEntity => trial_spawner::TrialSpawnerBlockEntity::new,
    "vault" => vault::VaultBlockEntity => vault::VaultBlockEntity::new,
    "test_block" => test_block::TestBlockBlockEntity => test_block::TestBlockBlockEntity::new,
    "test_instance_block" => test_instance_block::TestInstanceBlockBlockEntity => test_instance_block::TestInstanceBlockBlockEntity::new,
    "copper_golem_statue" => copper_golem_statue::CopperGolemStatueBlockEntity => copper_golem_statue::CopperGolemStatueBlockEntity::new,
    "potent_sulfur" => potent_sulfur::PotentSulfurBlockEntity => potent_sulfur::PotentSulfurBlockEntity::new,
    "map" => map::MapBlockEntity => |position| map::MapBlockEntity::new(position, 0),
}

/// Returns the tag sent to clients for a block entity type, given its saved tag.
///
/// This mirrors vanilla's `getUpdateTag`. Containers send an empty tag because
/// their contents only reach the client when a window opens.
#[must_use]
pub fn block_entity_client_update_tag(type_id: u16, saved: NbtCompound) -> NbtCompound {
    if block_entity_sends_saved_update_tag(type_id) {
        saved
    } else {
        NbtCompound::new()
    }
}

#[must_use]
pub fn create_block_entity_for_state(
    state_id: BlockStateId,
    position: BlockPos,
) -> Option<Arc<dyn BlockEntity>> {
    use pumpkin_data::BlockState;
    use pumpkin_data::block_properties::StructureblockMode;

    let state = BlockState::from_id(state_id);
    if state.block_entity_type == u16::MAX {
        return None;
    }
    let block = Block::from_state_id(state_id);
    let name = BLOCK_ENTITY_TYPES.get(state.block_entity_type as usize)?;
    match *name {
        "structure_block" => {
            let mode = structure_block::StructureBlockBlockEntity::mode_from_block_state(state_id)
                .unwrap_or(StructureblockMode::Load);
            Some(Arc::new(
                structure_block::StructureBlockBlockEntity::new_with_mode(position, mode),
            ))
        }
        "command_block" => Some(Arc::new(command_block::CommandBlockEntity::new(
            position,
            true,
            block == &Block::CHAIN_COMMAND_BLOCK,
        ))),
        _ => create_block_entity(state.block_entity_type, position),
    }
}

#[cfg(test)]
mod test {
    use super::{
        BLOCK_ENTITY_TYPES, BlockEntity, barrel::BarrelBlockEntity,
        blasting_furnace::BlastingFurnaceBlockEntity, block_entity_client_update_tag,
        block_entity_from_nbt, block_entity_sends_saved_update_tag,
        brewing_stand::BrewingStandBlockEntity, chest::ChestBlockEntity,
        crafter::CrafterBlockEntity, create_block_entity, dispenser::DispenserBlockEntity,
        dropper::DropperBlockEntity, furnace::FurnaceBlockEntity, hopper::HopperBlockEntity,
        shulker_box::ShulkerBoxBlockEntity, sign::SignBlockEntity, smoker::SmokerBlockEntity,
        trapped_chest::TrappedChestBlockEntity,
    };
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::Inventory;
    use pumpkin_nbt::compound::NbtCompound;
    use pumpkin_util::math::position::BlockPos;
    use std::sync::Arc;

    #[test]
    fn container_update_tags_are_empty() {
        let position = BlockPos::new(0, 0, 0);
        let mut saved = NbtCompound::new();
        saved.put_string("CustomName", "stored".to_string());

        let containers: [Arc<dyn BlockEntity>; 12] = [
            Arc::new(ChestBlockEntity::new(position)),
            Arc::new(TrappedChestBlockEntity::new(position)),
            Arc::new(BarrelBlockEntity::new(position)),
            Arc::new(ShulkerBoxBlockEntity::new(position)),
            Arc::new(DispenserBlockEntity::new(position)),
            Arc::new(DropperBlockEntity::new(position)),
            Arc::new(HopperBlockEntity::new(position)),
            Arc::new(FurnaceBlockEntity::new(position)),
            Arc::new(BlastingFurnaceBlockEntity::new(position)),
            Arc::new(SmokerBlockEntity::new(position)),
            Arc::new(BrewingStandBlockEntity::new(position)),
            Arc::new(CrafterBlockEntity::new(position)),
        ];
        for container in containers {
            let type_id = u16::try_from(container.get_id()).unwrap_or(u16::MAX);
            assert!(
                !block_entity_sends_saved_update_tag(type_id),
                "{} must not send its saved contents",
                container.resource_location()
            );
            assert!(
                block_entity_client_update_tag(type_id, saved.clone()).is_empty(),
                "{} must send an empty client tag",
                container.resource_location()
            );
        }

        let sign = SignBlockEntity::empty(position);
        let type_id = u16::try_from(sign.get_id()).unwrap_or(u16::MAX);
        assert!(block_entity_sends_saved_update_tag(type_id));
        assert_eq!(
            block_entity_client_update_tag(type_id, saved.clone()),
            saved
        );
    }

    #[test]
    fn every_block_entity_type_can_be_constructed() {
        for (type_id, name) in BLOCK_ENTITY_TYPES.iter().enumerate() {
            let Ok(type_id) = u16::try_from(type_id) else {
                continue;
            };
            let block_entity = create_block_entity(type_id, BlockPos::ZERO);
            assert!(block_entity.is_some(), "missing constructor for {name}");
            if let Some(block_entity) = block_entity {
                assert_eq!(
                    block_entity.get_id(),
                    u32::from(type_id),
                    "type id mismatch for {name}"
                );
            }
        }
    }

    /// A loaded block entity is serialized back into its chunk with
    /// `write_internal`, so whatever it holds has to survive that round trip or
    /// it is gone the next time the chunk is read.
    #[tokio::test]
    async fn furnace_contents_survive_a_chunk_round_trip() {
        let position = BlockPos::new(0, 100, 0);
        let furnace = Arc::new(FurnaceBlockEntity::new(position));
        furnace.set_stack(0, ItemStack::new(5, &Item::DIAMOND));

        let mut nbt = NbtCompound::new();
        furnace.write_internal(&mut nbt);

        let inventory = block_entity_from_nbt(&nbt).and_then(BlockEntity::get_inventory);
        assert!(
            inventory.is_some(),
            "furnace should be readable back from its own NBT"
        );

        if let Some(inventory) = inventory {
            let stack = inventory.get_stack(0);
            assert_eq!(stack.get_item().id, Item::DIAMOND.id);
            assert_eq!(stack.item_count, 5);
        }
    }
}
