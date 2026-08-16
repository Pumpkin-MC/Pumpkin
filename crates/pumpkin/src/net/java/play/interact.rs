#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    #[expect(clippy::too_many_lines)]
    pub async fn handle_interact(
        &self,
        player: &Arc<Player>,
        interact: SInteract,
        server: &Arc<Server>,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();
        let entity_id = interact.entity_id;

        let sneaking = interact.sneaking;
        let player_entity = &player.get_entity();
        if player_entity.is_sneaking() != sneaking {
            player_entity.set_sneaking(sneaking).await;
        }
        let Ok(action) = ActionType::try_from(interact.r#type.0) else {
            self.kick(TextComponent::text("Invalid action type")).await;
            return;
        };

        // Resolve the target entity for the event
        let world = player_entity.world.load_full();
        let player_target = world.get_player_by_id(entity_id.0);
        let target: Option<Arc<dyn EntityBase>> = player_target
            .as_ref()
            .map(|p| Arc::clone(p) as Arc<dyn EntityBase>)
            .or_else(|| world.get_entity_by_id(entity_id.0));

        if let Some(target) = target {
            if player.gamemode.load() == GameMode::Spectator {
                player.camera_target_id.store(Some(entity_id.0));
                player
                    .client
                    .send_packet_now(&CSetCamera::new(entity_id))
                    .await;
                return;
            }
            send_cancellable! {{
                server;
                PlayerInteractEntityEvent::new(
                    player,
                    Arc::clone(&target),
                    action.clone(),
                    interact.target_position,
                    sneaking,
                );

                'after: {
                    match event.action {
                        ActionType::Attack => {
                            let config = &server.advanced_config.pvp;
                            if !config.enabled {
                                return;
                            }

                            if entity_id.0 == player.entity_id() {
                                self.kick(TextComponent::translate_cross(translation::java::MULTIPLAYER_DISCONNECT_INVALID_ENTITY_ATTACKED, translation::java::MULTIPLAYER_DISCONNECT_INVALID_ENTITY_ATTACKED, [],))
                                .await;
                                return;
                            }

                            if let Some(player_victim) = &player_target {
                                if player_victim.living_entity.health.load() <= 0.0 {
                                    return;
                                }
                                if config.protect_creative
                                    && player_victim.gamemode.load() == GameMode::Creative
                                {
                                    world
                                        .play_sound(
                                            Sound::EntityPlayerAttackNodamage,
                                            SoundCategory::Players,
                                            &player_victim.position(),
                                        )
                                        ;
                                    return;
                                }
                            }
                            player.attack(event.target).await;
                        }
                        ActionType::Interact | ActionType::InteractAt => {
                            if event.action == ActionType::InteractAt
                                && let Some(pos) = interact.target_position
                            {
                                let mut at_event = crate::plugin::api::events::player::player_interact_at_entity::PlayerInteractAtEntityEvent::new(
                                    player.clone(),
                                    entity_id.0,
                                    pos.x,
                                    pos.y,
                                    pos.z,
                                    u8::from(interact.hand.map_or(0, |h| h.0) != 0),
                                );
                                server.plugin_manager.fire(server, &mut at_event).await;
                                if at_event.cancelled {
                                    return;
                                }
                            }
                            let mut stack = player.inventory().held_item().await;
                            let target_entity = event.target.get_entity();
                            if target_entity.entity_type.resource_name == "zombie_villager"
                                && stack.item.registry_key == "golden_apple"
                            {
                                player.trigger_advancement(crate::entity::player::advancement::trigger::AdvancementTrigger::CuredZombieVillager).await;
                            }

                            let interacted = event.target.interact(player, &mut stack).await;
                            if !interacted {
                                server
                                    .item_registry
                                    .use_on_entity(&mut stack, player, event.target)
                                    .await;
                            }
                            player.inventory().set_held_item(stack).await;
                        }
                    }
                }
            }}
        } else {
            // Entity not found
            send_cancellable! {{
                server;
                PlayerInteractUnknownEntityEvent::new(player, entity_id.0, action);

                'after: {
                    if event.action == ActionType::Attack {
                        error!(
                            "Player id {} interacted with entity id {}, which was not found.",
                            player.entity_id(),
                            event.entity_id
                        );
                        self.kick(TextComponent::translate_cross(translation::java::MULTIPLAYER_DISCONNECT_INVALID_ENTITY_ATTACKED, translation::java::MULTIPLAYER_DISCONNECT_INVALID_ENTITY_ATTACKED, [],))
                        .await;
                    }
                }
            }}
        }
    }
}
