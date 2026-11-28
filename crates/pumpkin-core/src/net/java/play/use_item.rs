#[allow(clippy::wildcard_imports)]
use super::*;
use crate::item::items::cart_air_use::{CartAirUseResult, use_cart_in_air};
use pumpkin_util::{math::wrap_degrees, version::JavaMinecraftVersion};

impl JavaClient {
    #[expect(
        clippy::too_many_lines,
        reason = "The Java handler keeps the packet's source and veto in one dispatch path."
    )]
    pub fn handle_use_item(&self, player: &Arc<Player>, use_item: &SUseItem, server: &Arc<Server>) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();

        let inventory = player.inventory();
        let Ok(hand) = Hand::from_packet_id(use_item.hand.0) else {
            self.try_kick(&TextComponent::text("InvalidHand"));
            return;
        };
        let version = self.version.load();
        if version >= JavaMinecraftVersion::V_1_21
            && (!use_item.yaw.is_finite() || !use_item.pitch.is_finite())
        {
            self.try_kick(&TextComponent::text("Invalid item use rotation"));
            return;
        }
        self.update_sequence(use_item.sequence.0);
        if player.gamemode.load() == GameMode::Spectator {
            return;
        }

        let item_in_hand = inventory.get_stack_in_hand(hand);
        let item_id = item_in_hand.get_item().id;
        let boat = server.item_registry.boat_item(item_id);
        let is_cart = server.item_registry.minecart_item(item_id).is_some();
        let is_vehicle = boat.is_some() || is_cart;
        let vehicle_world = is_vehicle.then(|| player.world());
        let vehicle_slot_type = crate::entity::hand_equipment_slot(hand);
        let vehicle_slot = inventory.equipment_slot_index(&vehicle_slot_type);

        let consume_event =
            crate::plugin::api::events::player::player_item_consume::PlayerItemConsumeEvent::new(
                player.clone(),
                item_in_hand.item.registry_key.to_string(),
            );
        let consume_event = server.plugin_manager.fire_blocking(server, consume_event);
        let world_is_current = || {
            vehicle_world
                .as_ref()
                .is_none_or(|world| Arc::ptr_eq(world, &player.world()))
        };
        if consume_event.cancelled || !world_is_current() {
            return;
        }

        let (use_yaw, use_pitch) = if version < JavaMinecraftVersion::V_1_21 {
            player.rotation()
        } else if is_vehicle {
            let yaw = wrap_degrees(use_item.yaw);
            let pitch = wrap_degrees(use_item.pitch).clamp(-90.0, 90.0);
            player.get_entity().set_rotation(yaw, pitch);
            (yaw, pitch)
        } else {
            (use_item.yaw, use_item.pitch)
        };
        let (ray_yaw, ray_pitch) = if is_vehicle {
            (use_yaw, use_pitch)
        } else {
            (use_item.yaw, use_item.pitch)
        };
        let hit = player.world().clip(&pumpkin_world::clip::ClipContext::new(
            player.eye_position(),
            player
                .eye_position()
                .add(&(Vector3::rotation_vector(f64::from(ray_pitch), f64::from(ray_yaw)) * 4.5)),
            pumpkin_world::clip::ClipBlock::Outline,
            pumpkin_world::clip::ClipFluid::None,
        ));

        let event = if hit.is_miss {
            PlayerInteractEvent::new(player, InteractAction::RightClickAir, &Block::AIR, None)
        } else {
            let hit_pos = hit.block_pos;
            PlayerInteractEvent::new(
                player,
                InteractAction::RightClickBlock,
                player.world().get_block(&hit_pos),
                Some(hit_pos),
            )
        };
        let item = item_in_hand.item;
        if !Self::should_continue_use_after_fish_event(server, player, hand, item) {
            return;
        }

        send_cancellable_blocking! {{
            server;
            event;
            'after: {
                let held = inventory.get_stack_in_hand(hand);
                if held.uid != item_in_hand.uid || !held.are_equal(&item_in_hand) {
                    return;
                }
                if let Some(world) = vehicle_world.as_ref() {
                    let Some(slot) = vehicle_slot else {
                        return;
                    };
                    if !world_is_current()
                        || player.gamemode.load() == GameMode::Spectator
                        || inventory.equipment_slot_index(&vehicle_slot_type) != Some(slot)
                    {
                        return;
                    }
                    let swing = boat.map_or_else(
                        || use_cart_in_air(player, hand, slot, &held, world) == CartAirUseResult::Equip,
                        |boat| boat.place_from_hand_in_world(&held, player, use_yaw, use_pitch, hand, world),
                    );
                    if swing && version >= JavaMinecraftVersion::V_26_3 {
                        player.swing_after_vehicle_interaction(hand, &held, false, world);
                    }
                    return;
                }
                Self::use_held_item(server, player, hand, &held, use_yaw, use_pitch);
            }
        }}
    }

    fn use_held_item(
        server: &Arc<Server>,
        player: &Player,
        hand: Hand,
        held: &ItemStack,
        yaw: f32,
        pitch: f32,
    ) {
        if player.gamemode.load() == GameMode::Spectator {
            return;
        }
        let cooldown = held.get_use_cooldown();
        let group = cooldown
            .and_then(|c| c.cooldown_group.clone())
            .unwrap_or_else(|| held.item.registry_key.to_string());
        if player.is_on_cooldown(&group) {
            return;
        }

        let native_use = server.item_registry.overrides_air_use(held.item.id);
        let consumable = held.get_data_component::<ConsumableImpl>().is_some();
        if !native_use
            && !consumable
            && let Some(equippable) = held.get_data_component::<EquippableImpl>()
            && equippable.swappable
        {
            let before = player.screen_equipment();
            if equippable
                .allowed_entities
                .as_ref()
                .is_none_or(|allowed| allowed.contains(&pumpkin_data::entity::EntityType::PLAYER))
                && let Some(extras) = player.inventory().swap_equippable(
                    hand,
                    equippable.slot,
                    held,
                    player.gamemode.load() == GameMode::Creative,
                )
            {
                if held.get_base_use_time() == 0
                    && let Some(cooldown) = cooldown
                {
                    player.start_cooldown(group, (cooldown.seconds * 20.0) as i32);
                }
                player.increment_stat(StatisticCategory::Used, i32::from(held.item.id), 1);
                for extra in extras {
                    player.inventory().offer(extra, false, player);
                }
                player.publish_screen_equipment(&before);
                player.send_player_screen_updates();
            }
            return;
        }

        player.increment_stat(StatisticCategory::Used, i32::from(held.item.id), 1);
        let current = player.inventory().get_stack_in_hand(hand);
        if current.uid != held.uid || !current.are_equal(held) || player.is_on_cooldown(&group) {
            return;
        }
        let kinetic = !native_use
            && held
                .get_data_component::<pumpkin_data::data_component_impl::KineticWeaponImpl>()
                .is_some()
            && held
                .get_data_component::<pumpkin_data::data_component_impl::BlocksAttacksImpl>()
                .is_none();
        if !native_use
            && (consumable || kinetic)
            && held
                .get_data_component::<FoodImpl>()
                .is_none_or(|food| player.can_eat(food.can_always_eat))
        {
            player
                .living_entity
                .set_active_hand(hand, held.clone(), held.get_max_use_time());
        }
        server
            .item_registry
            .on_use_with_rotation(held, player, yaw, pitch, hand);
    }

    fn should_continue_use_after_fish_event(
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
        fish_event = server.plugin_manager.fire_blocking(server, fish_event);
        !fish_event.cancelled
    }
}
