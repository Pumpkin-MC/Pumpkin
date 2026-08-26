#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_set_creative_slot(
        &self,
        player: &Arc<Player>,
        packet: SSetCreativeSlot,
    ) -> Result<(), InventoryError> {
        if player.gamemode.load() != GameMode::Creative {
            return Err(InventoryError::PermissionError);
        }
        let is_negative = packet.slot < 0;
        let valid_slot = packet.slot >= 1 && packet.slot as usize <= 45;
        let item_stack = packet
            .clicked_item
            .to_stack_for_version(&self.version.load());
        let mut creative_event =
            crate::plugin::api::events::inventory::inventory_creative::InventoryCreativeEvent::new(
                player.clone(),
                packet.slot,
                item_stack.item.registry_key.to_string(),
                item_stack.item_count,
            );
        if let Some(server) = player.world().server.upgrade() {
            server
                .plugin_manager
                .fire(&server, &mut creative_event)
                .await;
        }
        if creative_event.cancelled {
            return Ok(());
        }

        let is_legal =
            item_stack.is_empty() || item_stack.item_count <= item_stack.get_max_stack_size();

        if valid_slot && is_legal {
            let mut player_screen_handler = player
                .player_screen_handler
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            let is_armor_equipped = player_screen_handler
                .get_slot(packet.slot as usize)
                .get_stack()
                .are_equal(&item_stack);
            if !is_armor_equipped {
                if (5..9).contains(&packet.slot) {
                    player.enqueue_equipment_change(
                        &match packet.slot {
                            5 => EquipmentSlot::HEAD,
                            6 => EquipmentSlot::CHEST,
                            7 => EquipmentSlot::LEGS,
                            8 => EquipmentSlot::FEET,
                            _ => {
                                tracing::error!("Invalid armor slot: {}", packet.slot);
                                EquipmentSlot::HEAD
                            }
                        },
                        &item_stack,
                    );
                } else if (36..45).contains(&packet.slot) {
                    let slot = packet.slot - 36;
                    if player.inventory().get_selected_slot() == slot as u8 {
                        let equipment = &[(EquipmentSlot::MAIN_HAND, item_stack.clone())];
                        player.living_entity.send_equipment_changes(equipment);
                    }
                }
            }

            player_screen_handler
                .get_slot(packet.slot as usize)
                .set_stack(item_stack.clone());
            player_screen_handler.set_received_stack(packet.slot as usize, item_stack);
            player_screen_handler.send_content_updates();
            drop(player_screen_handler);
        } else if is_negative && is_legal {
            // Item drop
            player.drop_item(item_stack);
        }
        Ok(())
    }
}
