#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_use_item(
        &self,
        player: &Arc<Player>,
        use_item: &SUseItem,
        server: &Arc<Server>,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();

        let inventory = player.inventory();
        let Ok(hand) = Hand::from_packet_id(use_item.hand.0) else {
            self.kick(TextComponent::text("InvalidHand")).await;
            return;
        };
        self.update_sequence(player, use_item.sequence.0);

        let mut item_in_hand = inventory.get_stack_in_hand(hand);

        let mut consume_event =
            crate::plugin::api::events::player::player_item_consume::PlayerItemConsumeEvent::new(
                player.clone(),
                item_in_hand.item.registry_key.to_string(),
            );
        server.plugin_manager.fire(server, &mut consume_event).await;
        if consume_event.cancelled {
            return;
        }

        let (item_id, _item) = (item_in_hand.item.id, item_in_hand.item);
        player.increment_stat(StatisticCategory::Used, item_id as i32, 1);

        let hit_result = player.world().raycast(
            player.eye_position(),
            player.eye_position().add(
                &(Vector3::rotation_vector(f64::from(use_item.pitch), f64::from(use_item.yaw))
                    * 4.5),
            ),
            |pos, world| {
                let block = world.get_block(pos);
                block != &Block::AIR && block != &Block::WATER && block != &Block::LAVA
            },
        );

        let event = if let Some((hit_pos, _hit_dir)) = hit_result {
            PlayerInteractEvent::new(
                player,
                InteractAction::RightClickBlock,
                player.world().get_block(&hit_pos),
                Some(hit_pos),
            )
        } else {
            PlayerInteractEvent::new(player, InteractAction::RightClickAir, &Block::AIR, None)
        };
        let (item_for_use, stack_for_use) = (item_in_hand.item, item_in_hand.clone());
        Self::prepare_hand_item_for_use(player, hand, &mut item_in_hand);

        if !self
            .should_continue_use_after_fish_event(server, player, hand, item_for_use)
            .await
        {
            return;
        }

        send_cancellable! {{
            server;
            event;
            'after: {
                server.item_registry.on_use(&stack_for_use, player);
            }
        }}
    }

    fn prepare_hand_item_for_use(player: &Arc<Player>, hand: Hand, held: &mut ItemStack) {
        let inventory = player.inventory();

        if let Some(cooldown) = held.get_use_cooldown() {
            let group = cooldown
                .cooldown_group
                .clone()
                .unwrap_or_else(|| held.item.registry_key.to_string());
            if player.is_on_cooldown(&group) {
                return;
            }
        }

        if held.get_data_component::<ConsumableImpl>().is_some()
            || held.get_data_component::<BlocksAttacksImpl>().is_some()
        {
            // If its food we want to make sure we can actually consume it
            if let Some(food) = held.get_data_component::<FoodImpl>() {
                if player
                    .abilities
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .invulnerable
                    || food.can_always_eat
                    || player.hunger_manager.level.load() < 20
                {
                    player.living_entity.set_active_hand(
                        hand,
                        held.clone(),
                        held.get_max_use_time(),
                    );
                }
            } else {
                player
                    .living_entity
                    .set_active_hand(hand, held.clone(), held.get_max_use_time());
            }
        }
        let equipment_slot = held
            .get_data_component::<EquippableImpl>()
            .map(|equippable| equippable.slot.clone());
        if let Some(slot) = equipment_slot {
            // The equipment lock has to be released before touching the hand again:
            // the off hand lives in the same map, so holding it here would deadlock.
            let current_equipped = inventory
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(&slot);
            if current_equipped.are_items_and_components_equal(held) {
                return;
            }

            player.enqueue_equipment_change(&slot, held);

            let equipped = if current_equipped.is_empty() {
                let equipped = held.clone();
                held.decrement_unless_creative(player.gamemode.load(), 1);
                equipped
            } else {
                std::mem::replace(held, current_equipped)
            };
            inventory
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .put(&slot, equipped);
            inventory.set_stack_in_hand(hand, held.clone());
        }
    }

    async fn should_continue_use_after_fish_event(
        &self,
        server: &Arc<Server>,
        player: &Arc<Player>,
        hand: Hand,
        item_for_use: &Item,
    ) -> bool {
        if item_for_use.id != Item::FISHING_ROD.id {
            return true;
        }

        // TODO: Apply fishing rod durability on retrieval based on catch type.
        let mut fish_event = PlayerFishEvent::new(
            player.clone(),
            None,
            uuid::Uuid::nil(),
            String::new(),
            PlayerFishState::Fishing,
            hand,
            0,
        );
        server.plugin_manager.fire(server, &mut fish_event).await;
        !fish_event.cancelled
    }
}
