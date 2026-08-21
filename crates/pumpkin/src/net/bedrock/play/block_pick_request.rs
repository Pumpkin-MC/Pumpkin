#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    #[allow(clippy::too_many_lines)]
    pub async fn handle_block_pick_request(&self, player: &Arc<Player>, packet: SBlockPickRequest) {
        if !player.can_interact_with_block_at(&packet.block_pos, 1.0) {
            return;
        }

        let world = player.world();
        let block = world.get_block(&packet.block_pos);

        if block.item_id == 0 {
            return;
        }

        let Some(item) = pumpkin_data::item::Item::from_id(block.item_id) else {
            return;
        };
        let stack = ItemStack::new(1, item);

        let target_hotbar_slot = packet.hotbar_slot as usize;
        if target_hotbar_slot >= 9 {
            return;
        }

        let slot_with_stack = player.inventory().get_slot_with_stack(&stack).await;

        if slot_with_stack != -1 {
            if pumpkin_inventory::player::player_inventory::PlayerInventory::is_valid_hotbar_index(
                slot_with_stack as usize,
            ) {
                if slot_with_stack as usize != target_hotbar_slot {
                    let target_stack = player.inventory().get_stack(target_hotbar_slot).await;
                    let source_stack = player.inventory().get_stack(slot_with_stack as usize).await;
                    player
                        .inventory()
                        .set_stack(target_hotbar_slot, source_stack)
                        .await;
                    player
                        .inventory()
                        .set_stack(slot_with_stack as usize, target_stack)
                        .await;
                }
            } else {
                let target_stack = player.inventory().get_stack(target_hotbar_slot).await;
                let source_stack = player.inventory().get_stack(slot_with_stack as usize).await;
                player
                    .inventory()
                    .set_stack(target_hotbar_slot, source_stack)
                    .await;
                player
                    .inventory
                    .set_stack(slot_with_stack as usize, target_stack)
                    .await;
            }
        } else if player.gamemode.load() == GameMode::Creative {
            player.inventory.set_stack(target_hotbar_slot, stack).await;
        } else {
            return;
        }

        player.inventory.set_selected_slot(target_hotbar_slot as u8);

        // Send hotbar updates
        player
            .client
            .enqueue_packet_editioned(
                &CSetSelectedSlot::new(player.inventory.get_selected_slot() as i8),
                &CPlayerHotbar {
                    selected_slot: VarUInt(player.inventory.get_selected_slot() as u32),
                    container_id: 0,
                    should_select_block: true,
                },
            )
            .await;

        // Send screen handler / Java inventory updates
        player
            .player_screen_handler
            .lock()
            .await
            .send_content_updates()
            .await;

        // Sync main hand equipment to other players
        let stack_in_hand = player.inventory().held_item().await;
        let equipment = &[(EquipmentSlot::MAIN_HAND, stack_in_hand)];
        player.living_entity.send_equipment_changes(equipment);

        // Sync bedrock inventory updates
        self.enqueue_client_packet(&CInventoryContent {
            container_id: VarUInt(0),
            slots: player
                .inventory()
                .main_inventory
                .read()
                .await
                .iter()
                .map(NetworkItemStackDescriptor::from)
                .collect(),
            full_container_name: FullContainerName {
                container_name: ContainerName::Inventory,
                dynamic_id: None,
            },
            storage_item: NetworkItemStackDescriptor::default(),
        })
        .await;
    }
}
