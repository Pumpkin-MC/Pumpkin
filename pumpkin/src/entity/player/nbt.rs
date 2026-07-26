use super::DATA_VERSION;
use super::Player;
use super::RespawnPoint;
use crate::entity::NBTStorage;
use crate::entity::NBTStorageInit;
use crate::entity::NbtFuture;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_inventory::player::ender_chest_inventory::EnderChestInventory;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::GameMode;
use pumpkin_util::math::experience;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::inventory::Inventory;
use std::sync::atomic::Ordering;
use tracing::warn;

impl NBTStorage for Player {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            nbt.put_int("DataVersion", DATA_VERSION);
            self.living_entity.write_nbt(nbt).await;
            self.inventory.write_nbt(nbt).await;
            self.ender_chest_inventory.write_nbt(nbt).await;

            self.abilities.lock().await.write_nbt(nbt).await;

            let total_exp =
                experience::points_to_level(self.experience_level.load(Ordering::Relaxed))
                    + self.experience_points.load(Ordering::Relaxed);
            nbt.put_int("XpTotal", total_exp);
            nbt.put_byte("playerGameType", self.gamemode.load() as i8);
            if let Some(previous_gamemode) = self.previous_gamemode.load() {
                nbt.put_byte("previousPlayerGameType", previous_gamemode as i8);
            }

            nbt.put_bool(
                "HasPlayedBefore",
                self.has_played_before.load(Ordering::Relaxed),
            );

            // Store food level, saturation, exhaustion, and tick timer
            self.hunger_manager.write_nbt(nbt).await;

            nbt.put_string(
                "Dimension",
                self.world().dimension.minecraft_name.to_string(),
            );

            if let Some(respawn) = self.respawn_point.lock().await.as_ref() {
                nbt.put_int("SpawnX", respawn.position.0.x);
                nbt.put_int("SpawnY", respawn.position.0.y);
                nbt.put_int("SpawnZ", respawn.position.0.z);
                nbt.put_string(
                    "SpawnDimension",
                    respawn.dimension.minecraft_name.to_owned(),
                );
                nbt.put_bool("SpawnForced", respawn.force);
            }
            nbt.put_int("XpSeed", self.enchantment_seed.load(Ordering::Relaxed));
            self.stats.lock().await.write_nbt(nbt);
        })
    }

    fn read_nbt<'a>(&'a mut self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.living_entity.read_nbt(nbt).await;
            self.inventory.read_nbt_non_mut(nbt).await;
            self.ender_chest_inventory.read_nbt_non_mut(nbt).await;
            self.abilities.lock().await.read_nbt(nbt).await;

            // Load from total XP
            let total_exp = nbt.get_int("XpTotal").unwrap_or(0);
            let (level, points) = experience::total_to_level_and_points(total_exp);
            let progress = experience::progress_in_level(level, points);
            self.experience_level.store(level, Ordering::Relaxed);
            self.experience_progress.store(progress);
            self.experience_points.store(points, Ordering::Relaxed);

            self.gamemode.store(
                GameMode::try_from(nbt.get_byte("playerGameType").unwrap_or(0))
                    .unwrap_or(GameMode::Survival),
            );

            self.previous_gamemode.store(
                nbt.get_byte("previousPlayerGameType")
                    .and_then(|byte| GameMode::try_from(byte).ok()),
            );

            self.has_played_before.store(
                nbt.get_bool("HasPlayedBefore").unwrap_or(false),
                Ordering::Relaxed,
            );

            self.hunger_manager.read_nbt(nbt).await;

            // Load any saved spawnpoint data (SpawnX/SpawnY/SpawnZ, SpawnDimension, SpawnForced)
            if let (Some(x), Some(y), Some(z)) = (
                nbt.get_int("SpawnX"),
                nbt.get_int("SpawnY"),
                nbt.get_int("SpawnZ"),
            ) {
                let dim = nbt
                    .get_string("SpawnDimension")
                    .and_then(|s| Dimension::from_name(s).cloned())
                    .unwrap_or_else(|| self.world().dimension.clone());
                let force = nbt.get_bool("SpawnForced").unwrap_or(false);
                *self.respawn_point.lock().await = Some(RespawnPoint {
                    dimension: dim,
                    position: BlockPos(Vector3::new(x, y, z)),
                    yaw: 0.0,
                    force,
                });
            }
            self.enchantment_seed.store(
                nbt.get_int("XpSeed").unwrap_or(rand::random()),
                Ordering::Relaxed,
            );
            self.stats.lock().await.read_nbt(nbt);
        })
    }
}

impl NBTStorageInit for Player {}

impl NBTStorage for PlayerInventory {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            // Save the selected slot (hotbar)
            nbt.put_int("SelectedItemSlot", i32::from(self.get_selected_slot()));

            // Create inventory list with the correct capacity (inventory size)
            let mut items: Vec<NbtTag> = Vec::with_capacity(41);
            for (i, item) in self.main_inventory.iter().enumerate() {
                let stack = item.lock().await;
                if !stack.is_empty() {
                    let mut item_compound = NbtCompound::new();
                    item_compound.put_byte("Slot", i as i8);
                    stack.write_item_stack(&mut item_compound);
                    drop(stack);
                    items.push(NbtTag::Compound(item_compound));
                }
            }

            let mut equipment_compound = NbtCompound::new();
            for slot in self.equipment_slots.values() {
                let stack_binding = self.entity_equipment.lock().await.get(slot);
                let stack = stack_binding.lock().await;
                if !stack.is_empty() {
                    let mut item_compound = NbtCompound::new();
                    stack.write_item_stack(&mut item_compound);
                    drop(stack);
                    match slot {
                        EquipmentSlot::OffHand(_) => {
                            equipment_compound.put_compound("offhand", item_compound);
                        }
                        EquipmentSlot::Feet(_) => {
                            equipment_compound.put_compound("feet", item_compound);
                        }
                        EquipmentSlot::Legs(_) => {
                            equipment_compound.put_compound("legs", item_compound);
                        }
                        EquipmentSlot::Chest(_) => {
                            equipment_compound.put_compound("chest", item_compound);
                        }
                        EquipmentSlot::Head(_) => {
                            equipment_compound.put_compound("head", item_compound);
                        }
                        _ => {
                            warn!("Invalid equipment slot for a player");
                        }
                    }
                }
            }
            nbt.put_compound("equipment", equipment_compound);
            nbt.put("Inventory", NbtTag::List(items));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            // Read selected hotbar slot
            self.set_selected_slot(nbt.get_int("SelectedItemSlot").unwrap_or(0) as u8);
            // Process inventory list
            if let Some(inventory_list) = nbt.get_list("Inventory") {
                for tag in inventory_list {
                    if let Some(item_compound) = tag.extract_compound()
                        && let Some(slot_byte) = item_compound.get_byte("Slot")
                    {
                        let slot = slot_byte as usize;
                        if let Some(item_stack) = ItemStack::read_item_stack(item_compound) {
                            self.set_stack(slot, item_stack).await;
                        }
                    }
                }
            }

            if let Some(equipment) = nbt.get_compound("equipment") {
                if let Some(offhand) = equipment.get_compound("offhand")
                    && let Some(item_stack) = ItemStack::read_item_stack(offhand)
                {
                    self.set_stack(40, item_stack).await;
                }

                if let Some(head) = equipment.get_compound("head")
                    && let Some(item_stack) = ItemStack::read_item_stack(head)
                {
                    self.set_stack(39, item_stack).await;
                }

                if let Some(chest) = equipment.get_compound("chest")
                    && let Some(item_stack) = ItemStack::read_item_stack(chest)
                {
                    self.set_stack(38, item_stack).await;
                }

                if let Some(legs) = equipment.get_compound("legs")
                    && let Some(item_stack) = ItemStack::read_item_stack(legs)
                {
                    self.set_stack(37, item_stack).await;
                }

                if let Some(feet) = equipment.get_compound("feet")
                    && let Some(item_stack) = ItemStack::read_item_stack(feet)
                {
                    self.set_stack(36, item_stack).await;
                }
            }
        })
    }
}

impl NBTStorageInit for PlayerInventory {}

impl NBTStorage for EnderChestInventory {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            // Create item list with the correct capacity (inventory size)
            let mut items: Vec<NbtTag> = Vec::with_capacity(Self::INVENTORY_SIZE);
            for (i, item) in self.items.iter().enumerate() {
                let stack = item.lock().await;
                if !stack.is_empty() {
                    let mut item_compound = NbtCompound::new();
                    item_compound.put_byte("Slot", i as i8);
                    stack.write_item_stack(&mut item_compound);
                    drop(stack);
                    items.push(NbtTag::Compound(item_compound));
                }
            }

            nbt.put("EnderItems", NbtTag::List(items));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            // Process item list
            if let Some(item_list) = nbt.get_list("EnderItems") {
                for tag in item_list {
                    if let Some(item_compound) = tag.extract_compound()
                        && let Some(slot_byte) = item_compound.get_byte("Slot")
                    {
                        let slot = slot_byte as usize;
                        if let Some(item_stack) = ItemStack::read_item_stack(item_compound) {
                            self.set_stack(slot, item_stack).await;
                        }
                    }
                }
            }
        })
    }
}

impl NBTStorageInit for EnderChestInventory {}
