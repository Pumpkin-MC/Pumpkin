use super::ItemCooldown;
use super::Player;
use super::statistics;
use crate::entity::Entity;
use crate::entity::EntityBase;
use crate::entity::item::ItemEntity;
use crate::net::ClientPlatform;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::data_component_impl::EquippableImpl;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::screen::WindowType;
use pumpkin_data::sound::Sound;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::statistic::StatisticCategory;
use pumpkin_inventory::player::ender_chest_inventory::EnderChestInventory;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_inventory::screen_handler::PlayerFuture;
use pumpkin_inventory::screen_handler::ScreenHandler;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CItemCooldown;
use pumpkin_protocol::java::client::play::CSetContainerContent;
use pumpkin_protocol::java::client::play::CSetContainerProperty;
use pumpkin_protocol::java::client::play::CSetContainerSlot;
use pumpkin_protocol::java::client::play::CSetCursorItem;
use pumpkin_protocol::java::client::play::CSetEquipment;
use pumpkin_protocol::java::client::play::CSetPlayerInventory;
use pumpkin_protocol::java::client::play::CSetSelectedSlot;
use pumpkin_util::GameMode;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::inventory::Inventory;
use std::f64::consts::TAU;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tracing::debug;

impl Player {
    pub async fn start_cooldown(&self, group: String, duration: i32) {
        let mut cooldowns = self.item_cooldowns.lock().await;
        cooldowns.insert(
            group.clone(),
            ItemCooldown {
                start_tick: self.tick_counter.load(Ordering::Relaxed),
                duration,
            },
        );
        self.client
            .send_packet_now(&CItemCooldown::new(group, VarInt(duration)))
            .await;
    }

    pub async fn get_cooldown(&self, group: &str) -> f32 {
        let cooldowns = self.item_cooldowns.lock().await;
        if let Some(cooldown) = cooldowns.get(group) {
            let current_tick = self.tick_counter.load(Ordering::Relaxed);
            let elapsed = current_tick - cooldown.start_tick;
            if elapsed < cooldown.duration {
                return 1.0 - (elapsed as f32 / cooldown.duration as f32);
            }
        }
        0.0
    }

    pub async fn is_on_cooldown(&self, group: &str) -> bool {
        let mut cooldowns = self.item_cooldowns.lock().await;
        if let Some(cooldown) = cooldowns.get(group) {
            let current_tick = self.tick_counter.load(Ordering::Relaxed);
            if current_tick - cooldown.start_tick < cooldown.duration {
                return true;
            }
            cooldowns.remove(group);
        }
        false
    }

    pub const fn inventory(&self) -> &Arc<PlayerInventory> {
        &self.inventory
    }

    pub const fn ender_chest_inventory(&self) -> &Arc<EnderChestInventory> {
        &self.ender_chest_inventory
    }

    pub async fn drop_item(&self, item_stack: ItemStack) {
        self.increment_stat(
            statistics::StatisticCategory::Dropped,
            item_stack.item.id as i32,
            item_stack.item_count as i32,
        )
        .await;
        self.increment_stat(
            statistics::StatisticCategory::Custom,
            statistics::CustomStatistic::Drop as i32,
            1,
        )
        .await;
        let item_pos = self.living_entity.entity.pos.load()
            + Vector3::new(0.0, self.living_entity.entity.get_eye_height() - 0.3, 0.0);
        let entity = Entity::new(self.world(), item_pos, &EntityType::ITEM);

        let pitch = f64::from(self.living_entity.entity.pitch.load()).to_radians();
        let yaw = f64::from(self.living_entity.entity.yaw.load()).to_radians();
        let pitch_sin = pitch.sin();
        let pitch_cos = pitch.cos();
        let yaw_sin = yaw.sin();
        let yaw_cos = yaw.cos();
        let horizontal_offset = rand::random::<f64>() * TAU;
        let l = 0.02 * rand::random::<f64>();

        let velocity = Vector3::new(
            (-yaw_sin * pitch_cos).mul_add(0.3, horizontal_offset.cos() * l),
            (rand::random::<f64>() - rand::random::<f64>())
                .mul_add(0.1, (-pitch_sin).mul_add(0.3, 0.1)),
            (yaw_cos * pitch_cos).mul_add(0.3, horizontal_offset.sin() * l),
        );

        // TODO: Merge stacks together
        let item_entity = Arc::new(ItemEntity::new_with_velocity(
            entity, item_stack, velocity, 40,
        ));
        self.world().spawn_entity(item_entity).await;
    }

    pub async fn drop_held_item(&self, drop_stack: bool) {
        // Do not hold both item stack and screen handler locks at the same time.
        let (dropped_stack, updated_stack, selected_slot) = {
            let binding = self.inventory.held_item();
            let mut item_stack = binding.lock().await;

            if item_stack.is_empty() {
                return;
            }

            let drop_amount = if drop_stack { item_stack.item_count } else { 1 };
            let dropped_stack = item_stack.copy_with_count(drop_amount);
            item_stack.decrement(drop_amount);
            let updated_stack = item_stack.clone();
            let selected_slot = self.inventory.get_selected_slot();

            (dropped_stack, updated_stack, selected_slot)
        };

        self.drop_item(dropped_stack).await;

        let inv: Arc<dyn Inventory> = self.inventory.clone();
        let screen_binding = self.current_screen_handler.lock().await;
        let mut screen_handler = screen_binding.lock().await;
        if let Some(slot_index) = screen_handler
            .get_slot_index(&inv, selected_slot as usize)
            .await
        {
            screen_handler.set_received_stack(slot_index, updated_stack);
            screen_handler.send_content_updates().await;
        }
    }

    pub async fn swap_item(&self) {
        let (main_hand_item, off_hand_item) = self.inventory.swap_item().await;
        let equipment = &[
            (EquipmentSlot::MAIN_HAND, main_hand_item),
            (EquipmentSlot::OFF_HAND, off_hand_item),
        ];
        self.living_entity.send_equipment_changes(equipment);
        // todo this.player.stopUsingItem();
    }

    /// Find arrow in inventory (main hand, offhand, or inventory slots)
    pub async fn find_arrow(&self) -> Option<usize> {
        use pumpkin_data::item::Item;
        let inventory = self.inventory();

        // Check offhand first
        let stack = inventory.get_stack(PlayerInventory::OFF_HAND_SLOT).await;
        let item = stack.lock().await;
        if item.item.id == Item::ARROW.id && item.item_count > 0 {
            return Some(PlayerInventory::OFF_HAND_SLOT);
        }
        drop(item);

        // Check hotbar and main inventory
        for slot in 0..PlayerInventory::MAIN_SIZE {
            let stack = inventory.get_stack(slot).await;
            let item = stack.lock().await;
            if item.item.id == Item::ARROW.id && item.item_count > 0 {
                return Some(slot);
            }
        }

        None
    }

    /// Consume one arrow from the specified slot
    pub async fn consume_arrow(&self, slot: usize) -> bool {
        let gamemode = self.gamemode.load();
        if gamemode == GameMode::Creative {
            return true; // Don't consume in creative
        }

        let inventory = self.inventory();
        let stack_arc = inventory.get_stack(slot).await;
        let mut stack = stack_arc.lock().await;
        match stack.item_count {
            2.. => {
                stack.item_count -= 1;
                true
            }
            1 => {
                *stack = ItemStack::EMPTY.clone();
                true
            }
            _ => false,
        }
    }

    pub async fn has_item_in_inventory(&self, item: &pumpkin_data::item::Item) -> bool {
        for slot in &self.inventory.main_inventory {
            let stack = slot.lock().await;
            if !stack.is_empty() && stack.item.id == item.id {
                return true;
            }
        }
        let equipment = self.inventory.entity_equipment.lock().await;
        for slot_stack in equipment.equipment.values() {
            let stack = slot_stack.lock().await;
            if !stack.is_empty() && stack.item.id == item.id {
                return true;
            }
        }
        false
    }
}

impl InventoryPlayer for Player {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn drop_item(&self, item: ItemStack, _retain_ownership: bool) -> PlayerFuture<'_, ()> {
        Box::pin(async move {
            self.drop_item(item).await;
        })
    }

    // Synchronous methods remain unchanged
    fn has_infinite_materials(&self) -> bool {
        self.gamemode.load() == GameMode::Creative
    }

    fn is_creative(&self) -> bool {
        self.gamemode.load() == GameMode::Creative
    }

    fn experience_level(&self) -> i32 {
        self.experience_level
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    fn add_experience_levels(&self, levels: i32) -> PlayerFuture<'_, ()> {
        Box::pin(async move {
            self.add_experience_levels(levels).await;
        })
    }

    fn enchantment_seed(&self) -> i32 {
        self.enchantment_seed.load(Ordering::Relaxed)
    }

    fn set_enchantment_seed(&self, seed: i32) -> PlayerFuture<'_, ()> {
        Box::pin(async move {
            self.enchantment_seed.store(seed, Ordering::Relaxed);
        })
    }

    fn get_inventory(&self) -> Arc<PlayerInventory> {
        self.inventory.clone()
    }

    fn enqueue_inventory_packet<'a>(
        &'a self,
        packet: &'a CSetContainerContent,
    ) -> PlayerFuture<'a, ()> {
        Box::pin(async move {
            match self.client.as_ref() {
                ClientPlatform::Java(java) => {
                    java.enqueue_packet(packet).await;
                }
                ClientPlatform::Bedrock(bedrock) => {
                    use pumpkin_protocol::bedrock::{
                        client::inventory_content::CInventoryContent,
                        network_item::{
                            ContainerName, FullContainerName, NetworkItemStackDescriptor,
                        },
                    };
                    use pumpkin_protocol::codec::var_uint::VarUInt;

                    let window_id = packet.window_id.0 as u32;
                    let slots: Vec<NetworkItemStackDescriptor> = packet
                        .slot_data
                        .iter()
                        .map(|s| NetworkItemStackDescriptor::from(&*s.0))
                        .collect();

                    if window_id == 0 {
                        let bedrock_packet = CInventoryContent {
                            container_id: VarUInt(0),
                            slots,
                            full_container_name: FullContainerName {
                                container_name: ContainerName::Inventory,
                                dynamic_id: None,
                            },
                            storage_item: NetworkItemStackDescriptor::default(),
                        };
                        bedrock.enqueue_packet(&bedrock_packet).await;
                    }
                }
            }
        })
    }

    fn enqueue_slot_packet<'a>(&'a self, packet: &'a CSetContainerSlot) -> PlayerFuture<'a, ()> {
        Box::pin(async move {
            match self.client.as_ref() {
                ClientPlatform::Java(java) => {
                    java.enqueue_packet(packet).await;
                }
                ClientPlatform::Bedrock(bedrock) => {
                    use pumpkin_protocol::bedrock::{
                        client::inventory_slot::CInventorySlot,
                        network_item::{
                            ContainerName, FullContainerName, NetworkItemStackDescriptor,
                        },
                    };
                    use pumpkin_protocol::codec::var_uint::VarUInt;

                    let window_id = packet.window_id;
                    tracing::info!(
                        "enqueue_slot_packet: window_id={}, slot={}",
                        window_id,
                        packet.slot
                    );

                    if window_id == 0 {
                        tracing::info!(
                            "enqueue_slot_packet: window_id is 0, sending CInventorySlot to Bedrock client"
                        );
                        let slot_idx = packet.slot as usize;
                        let item_desc = NetworkItemStackDescriptor::from(&*packet.slot_data.0);

                        let bedrock_packet = CInventorySlot {
                            window_id: VarUInt(0),
                            inventory_slot: VarUInt(slot_idx as u32),
                            container_name: Some(FullContainerName {
                                container_name: ContainerName::Inventory,
                                dynamic_id: None,
                            }),
                            storage: None,
                            item: item_desc,
                        };
                        bedrock.enqueue_packet(&bedrock_packet).await;
                    } else {
                        let slot_idx = packet.slot as usize;
                        let item_desc = NetworkItemStackDescriptor::from(&*packet.slot_data.0);

                        // Container screen
                        let current_handler = self.current_screen_handler.lock().await.clone();
                        let handler = current_handler.lock().await;
                        let window_type = handler.window_type();
                        let total_slots = handler.get_behaviour().slots.len();
                        let bedrock_info = if total_slots >= 36 {
                            let container_slots = total_slots - 36;
                            if slot_idx < container_slots {
                                if window_type == Some(WindowType::Crafting) {
                                    if slot_idx == 0 {
                                        Some((ContainerName::CreatedOutput, 0))
                                    } else {
                                        Some((
                                            ContainerName::CraftingInput,
                                            (32 + slot_idx - 1) as u8,
                                        ))
                                    }
                                } else {
                                    Some((ContainerName::LevelEntity, slot_idx as u8))
                                }
                            } else {
                                let inv_slot = slot_idx - container_slots;
                                if inv_slot < 27 {
                                    Some((ContainerName::Inventory, (inv_slot + 9) as u8))
                                } else {
                                    Some((ContainerName::Inventory, (inv_slot - 27) as u8))
                                }
                            }
                        } else {
                            None
                        };

                        if let Some((container_name, slot_id)) = bedrock_info {
                            let bedrock_packet = CInventorySlot {
                                window_id: VarUInt(window_id as u32),
                                inventory_slot: VarUInt(slot_id as u32),
                                container_name: Some(FullContainerName {
                                    container_name,
                                    dynamic_id: None,
                                }),
                                storage: None,
                                item: item_desc,
                            };
                            bedrock.enqueue_packet(&bedrock_packet).await;
                        }
                    }
                }
            }
        })
    }

    fn enqueue_cursor_packet<'a>(&'a self, packet: &'a CSetCursorItem) -> PlayerFuture<'a, ()> {
        Box::pin(async move {
            match self.client.as_ref() {
                ClientPlatform::Java(java) => {
                    java.enqueue_packet(packet).await;
                }
                ClientPlatform::Bedrock(bedrock) => {
                    use pumpkin_protocol::bedrock::{
                        client::inventory_content::CInventoryContent,
                        network_item::{
                            ContainerName, FullContainerName, NetworkItemStackDescriptor,
                        },
                    };
                    use pumpkin_protocol::codec::var_uint::VarUInt;

                    let item_desc = NetworkItemStackDescriptor::from(&*packet.stack.0);
                    let bedrock_packet = CInventoryContent {
                        container_id: VarUInt(59),
                        slots: vec![item_desc],
                        full_container_name: FullContainerName {
                            container_name: ContainerName::Cursor,
                            dynamic_id: None,
                        },
                        storage_item: NetworkItemStackDescriptor::default(),
                    };
                    bedrock.enqueue_packet(&bedrock_packet).await;
                }
            }
        })
    }

    fn enqueue_property_packet<'a>(
        &'a self,
        packet: &'a CSetContainerProperty,
    ) -> PlayerFuture<'a, ()> {
        Box::pin(async move {
            self.client.enqueue_packet(packet).await;
        })
    }

    fn enqueue_slot_set_packet<'a>(
        &'a self,
        packet: &'a CSetPlayerInventory,
    ) -> PlayerFuture<'a, ()> {
        Box::pin(async move {
            match self.client.as_ref() {
                ClientPlatform::Java(java) => {
                    java.enqueue_packet(packet).await;
                }
                ClientPlatform::Bedrock(bedrock) => {
                    use pumpkin_protocol::bedrock::{
                        client::inventory_slot::CInventorySlot,
                        network_item::{
                            ContainerName, FullContainerName, NetworkItemStackDescriptor,
                        },
                    };
                    use pumpkin_protocol::codec::var_uint::VarUInt;

                    tracing::info!(
                        "enqueue_slot_set_packet: slot={}, sending CInventorySlot to Bedrock client",
                        packet.slot.0
                    );

                    let item_stack = &*packet.item.0;
                    let item_desc = NetworkItemStackDescriptor::from(item_stack);
                    let bedrock_packet = CInventorySlot {
                        window_id: VarUInt(0),
                        inventory_slot: VarUInt(packet.slot.0 as u32),
                        container_name: Some(FullContainerName {
                            container_name: ContainerName::Inventory,
                            dynamic_id: None,
                        }),
                        storage: None,
                        item: item_desc,
                    };
                    bedrock.enqueue_packet(&bedrock_packet).await;
                }
            }
        })
    }

    fn enqueue_set_held_item_packet<'a>(
        &'a self,
        packet: &'a CSetSelectedSlot,
    ) -> PlayerFuture<'a, ()> {
        Box::pin(async move {
            self.client
                .enqueue_packet_editioned(
                    packet,
                    &pumpkin_protocol::bedrock::client::CPlayerHotbar {
                        selected_slot: pumpkin_protocol::codec::var_uint::VarUInt(
                            packet.slot as u32,
                        ),
                        container_id: 0,
                        should_select_block: true,
                    },
                )
                .await;
        })
    }

    fn enqueue_equipment_change<'a>(
        &'a self,
        slot: &'a EquipmentSlot,
        stack: &'a ItemStack,
    ) -> PlayerFuture<'a, ()> {
        Box::pin(async move {
            let chunk_pos = self.living_entity.entity.chunk_pos.load();
            self.world().broadcast_to_chunk_except(
                chunk_pos,
                &[self.get_entity().entity_uuid],
                &CSetEquipment::new(
                    self.entity_id().into(),
                    vec![(
                        slot.discriminant(),
                        ItemStackSerializer::from(stack.clone()),
                    )],
                ),
            );

            // Play equip sound at the player. Prefer item equip_sound component;
            // fall back to a material guess so iron/diamond still sound right.
            if !stack.is_empty() {
                let pos = self.position();
                if let Some(equippable) = stack.get_data_component::<EquippableImpl>() {
                    self.world().play_sound_event(
                        &equippable.equip_sound,
                        SoundCategory::Players,
                        &pos,
                    );
                } else {
                    let name = stack.item.registry_key;
                    let sound = if name.contains("netherite") {
                        Sound::ItemArmorEquipNetherite
                    } else if name.contains("diamond") {
                        Sound::ItemArmorEquipDiamond
                    } else if name.contains("iron") {
                        Sound::ItemArmorEquipIron
                    } else if name.contains("gold") {
                        Sound::ItemArmorEquipGold
                    } else if name.contains("chain") {
                        Sound::ItemArmorEquipChain
                    } else if name.contains("leather") {
                        Sound::ItemArmorEquipLeather
                    } else if name.contains("copper") {
                        Sound::ItemArmorEquipCopper
                    } else if name.contains("turtle") {
                        Sound::ItemArmorEquipTurtle
                    } else if name.contains("elytra") {
                        Sound::ItemArmorEquipElytra
                    } else {
                        Sound::ItemArmorEquipGeneric
                    };
                    self.world().play_sound(sound, SoundCategory::Players, &pos);
                }
            }
        })
    }

    fn award_experience(&self, amount: i32) -> PlayerFuture<'_, ()> {
        Box::pin(async move {
            debug!("Player::award_experience called with amount={amount}");
            if amount > 0 {
                debug!("Player: adding {amount} experience points");
                if let Some(player) = self.world().get_player_by_uuid(self.gameprofile.id) {
                    player.add_experience_points(amount).await;
                }
            }
        })
    }

    fn increment_stat(
        &self,
        category: StatisticCategory,
        stat_id: i32,
        amount: i32,
    ) -> PlayerFuture<'_, ()> {
        Box::pin(async move {
            self.increment_stat(category, stat_id, amount).await;
        })
    }
}
