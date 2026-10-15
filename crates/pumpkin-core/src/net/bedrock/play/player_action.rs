#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    #[expect(clippy::match_same_arms)]
    #[expect(clippy::too_many_lines)]
    pub fn handle_player_action(
        &self,
        player: &Arc<Player>,
        server: &Arc<Server>,
        packet: &SPlayerAction,
    ) {
        if !player.has_client_loaded()
            || ((player.living_entity.dead.load(Ordering::Relaxed)
                || player.living_entity.health.load() <= 0.0)
                && !matches!(packet.action, PlayerAction::Respawn))
        {
            return;
        }
        player.update_last_action_time();

        match packet.action {
            PlayerAction::StartDestroyBlock
            | PlayerAction::CreativeDestroyBlock
            | PlayerAction::ContinueDestroyBlock => {
                let location = packet.block_position;
                if !player.can_interact_with_block_at(&location, 1.0) {
                    return;
                }

                let entity = &player.get_entity();
                let world = entity.world.load_full();
                let (block, state) = world.get_block_and_state(&location);

                if player.mining.load(Ordering::Relaxed)
                    && *player
                        .mining_pos
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        != location
                {
                    player.stop_mining();
                }

                if player.gamemode.load() == GameMode::Creative {
                    let new_state = world.break_block(
                        &location,
                        Some(player),
                        BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_DROPS,
                    );
                    if new_state.is_some() {
                        server
                            .block_registry
                            .broken(&world, block, player, &location, server, state);
                    }
                } else if !state.is_air() {
                    let starts_breaking = !player.mining.load(Ordering::Relaxed)
                        || *player
                            .mining_pos
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            != location;
                    if starts_breaking {
                        server
                            .block_registry
                            .attack(&world, block, state, &location, player);
                    }
                    let speed = crate::block::calc_block_breaking(player, state, block);
                    if speed >= 1.0 {
                        player.stop_mining();
                        let broken_state = world.get_block_state(&location);
                        let can_harvest = player.can_harvest(broken_state, block);
                        let flags = if can_harvest {
                            BlockFlags::NOTIFY_ALL
                        } else {
                            BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL
                        };
                        let new_state = world.break_block(&location, Some(player), flags);
                        if new_state.is_some() {
                            server.block_registry.broken(
                                &world,
                                block,
                                player,
                                &location,
                                server,
                                broken_state,
                            );
                            player.apply_tool_damage_for_block_break(broken_state);
                            if can_harvest {
                                player.add_exhaustion(MINE_BLOCK_EXHAUSTION);
                            }
                            let item_id = player.inventory().held_item().item.id;
                            player.increment_stat(
                                pumpkin_data::statistic::StatisticCategory::Used,
                                item_id as i32,
                                1,
                            );
                            player.increment_stat(
                                pumpkin_data::statistic::StatisticCategory::Mined,
                                block.id.as_u16() as i32,
                                1,
                            );
                        }
                    } else {
                        let mut mining_pos = player
                            .mining_pos
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        let progress = if starts_breaking {
                            player.start_mining_time.store(
                                player.tick_counter.load(Ordering::Relaxed),
                                Ordering::Relaxed,
                            );
                            player.mining.store(true, Ordering::Relaxed);
                            *mining_pos = location;
                            (speed * 10.0) as i32
                        } else {
                            player.current_block_destroy_stage.load(Ordering::Relaxed)
                        };
                        drop(mining_pos);
                        let old_speed = player
                            .current_block_breaking_speed
                            .swap(speed.to_bits(), Ordering::Relaxed);
                        if starts_breaking {
                            world.set_block_breaking(
                                entity,
                                location,
                                BlockBreakingProgress::Start {
                                    stage: progress,
                                    speed,
                                },
                            );
                            player
                                .current_block_destroy_stage
                                .store(progress, Ordering::Relaxed);
                        } else if old_speed != speed.to_bits() {
                            world.set_block_breaking(
                                entity,
                                location,
                                BlockBreakingProgress::Update {
                                    stage: progress,
                                    speed: Some(speed),
                                },
                            );
                        }
                    }
                }
            }
            action @ (PlayerAction::PredictDestroyBlock | PlayerAction::StopDestroyBlock) => {
                let location = packet.block_position;
                if !player.can_interact_with_block_at(&location, 1.0) {
                    return;
                }

                let entity = &player.get_entity();
                let world = entity.world.load_full();

                let (block, state) = world.get_block_and_state(&location);
                if player.gamemode.load() != GameMode::Creative && !state.is_air() {
                    let speed = crate::block::calc_block_breaking(player, state, block);
                    let elapsed = player.tick_counter.load(Ordering::Relaxed)
                        - player.start_mining_time.load(Ordering::Relaxed)
                        + 1;
                    let same_block = *player
                        .mining_pos
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        == location;
                    if player.mining.load(Ordering::Relaxed)
                        && same_block
                        && speed * elapsed as f32 >= MIN_PREDICTED_BREAK_PROGRESS
                    {
                        player.stop_mining();

                        let can_harvest = player.can_harvest(state, block);
                        let flags = if can_harvest {
                            BlockFlags::NOTIFY_ALL
                        } else {
                            BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL
                        };
                        if world.break_block(&location, Some(player), flags).is_some() {
                            server
                                .block_registry
                                .broken(&world, block, player, &location, server, state);
                            player.apply_tool_damage_for_block_break(state);
                            if can_harvest {
                                player.add_exhaustion(MINE_BLOCK_EXHAUSTION);
                            }
                            let item_id = player.inventory().held_item().item.id;
                            player.increment_stat(
                                pumpkin_data::statistic::StatisticCategory::Used,
                                item_id as i32,
                                1,
                            );
                            player.increment_stat(
                                pumpkin_data::statistic::StatisticCategory::Mined,
                                block.id.as_u16() as i32,
                                1,
                            );
                        }
                    } else {
                        let runtime_id = pumpkin_data::BlockState::to_be_network_id(state.id);
                        self.try_enqueue_client_packet(&CUpdateBlock::new(location, runtime_id));
                        if matches!(action, PlayerAction::StopDestroyBlock) {
                            player.stop_mining();
                        } else {
                            world.set_block_breaking(
                                entity,
                                location,
                                BlockBreakingProgress::Update {
                                    stage: player
                                        .current_block_destroy_stage
                                        .load(Ordering::Relaxed),
                                    speed: Some(speed),
                                },
                            );
                        }
                    }
                } else if matches!(action, PlayerAction::StopDestroyBlock) {
                    player.stop_mining();
                }
            }
            PlayerAction::CrackBlock => {
                // Don't do anything for this action. It is no longer used. Block
                // cracking is done fully server-side.
            }
            PlayerAction::AbortDestroyBlock => {
                player.stop_mining();
            }
            PlayerAction::DropItem => {
                player.drop_held_item(false);
            }
            PlayerAction::Respawn
                if player.living_entity.dead.load(Ordering::Relaxed)
                    || player.living_entity.health.load() <= 0.0 =>
            {
                let player_c = player.clone();
                player.spawn_task(async move {
                    player_c.world().respawn_player(&player_c, false).await;
                });
            }
            PlayerAction::StartSprinting => {
                if !player.get_entity().is_sprinting() {
                    send_cancellable_blocking! {{
                        server;
                        PlayerToggleSprintEvent::new(player.clone(), true);
                        'after: {
                            player.set_sprinting(event.is_sprinting);
                            player.update_player_pose();
                        }
                    }}
                }
            }
            PlayerAction::StopSprinting => {
                if player.get_entity().is_sprinting() {
                    send_cancellable_blocking! {{
                        server;
                        PlayerToggleSprintEvent::new(player.clone(), false);
                        'after: {
                            player.set_sprinting(event.is_sprinting);
                            player.update_player_pose();
                        }
                    }}
                }
            }
            PlayerAction::StartSneaking => {
                let entity = player.get_entity();
                if !entity.is_sneaking() {
                    send_cancellable_blocking! {{
                        server;
                        PlayerToggleSneakEvent::new(player.clone(), true);
                        'after: {
                            entity.set_sneaking(event.is_sneaking);
                            if event.is_sneaking {
                                let vehicle = entity
                                    .vehicle
                                    .lock()
                                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                                    .clone();
                                if let Some(vehicle) = vehicle {
                                    vehicle.get_entity().remove_passenger(player.entity_id());
                                }
                            }
                            player.update_player_pose();
                        }
                    }}
                }
            }
            PlayerAction::StopSneaking => {
                let entity = player.get_entity();
                if entity.is_sneaking() {
                    send_cancellable_blocking! {{
                        server;
                        PlayerToggleSneakEvent::new(player.clone(), false);
                        'after: {
                            entity.set_sneaking(event.is_sneaking);
                            player.update_player_pose();
                        }
                    }}
                }
            }
            PlayerAction::StartGliding => {
                let entity = player.get_entity();
                let fall_flying = entity.check_fall_flying();
                if entity.is_fall_flying() != fall_flying {
                    let mut event = crate::plugin::api::events::entity::entity_toggle_glide::EntityToggleGlideEvent::new(
                        entity.entity_id,
                        fall_flying,
                    );
                    server.plugin_manager.fire_blocking(server, &mut event);
                    if !event.cancelled {
                        entity.set_fall_flying(event.is_gliding);
                        player.update_player_pose();
                    }
                }
            }
            PlayerAction::StopGliding => {
                let entity = player.get_entity();
                if entity.is_fall_flying() {
                    let mut event = crate::plugin::api::events::entity::entity_toggle_glide::EntityToggleGlideEvent::new(
                        entity.entity_id,
                        false,
                    );
                    server.plugin_manager.fire_blocking(server, &mut event);
                    if !event.cancelled {
                        entity.set_fall_flying(false);
                        player.update_player_pose();
                    }
                }
            }
            PlayerAction::StartSwimming => {
                player.get_entity().set_swimming(true);
                player.update_player_pose();
            }
            PlayerAction::StopSwimming => {
                player.get_entity().set_swimming(false);
                player.update_player_pose();
            }
            PlayerAction::StartSpinAttack => {
                player.living_entity.set_auto_spin_attack(true);
                player.update_player_pose();
            }
            PlayerAction::StopSpinAttack => {
                player.living_entity.set_auto_spin_attack(false);
                player.update_player_pose();
            }
            PlayerAction::StartCrawling => {
                player.get_entity().set_pose(pumpkin_data::entity::EntityPose::Swimming);
            }
            PlayerAction::StopCrawling => {
                player.update_player_pose();
            }
            PlayerAction::StartFlying => {
                let flying = {
                    player
                        .abilities
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .flying
                };
                if !flying {
                    send_cancellable_blocking! {{
                        server;
                        PlayerToggleFlightEvent::new(player.clone(), true);
                        'after: {
                            player.living_entity.fall_distance.store(0.0);
                            {
                                player.abilities.lock().unwrap_or_else(std::sync::PoisonError::into_inner).flying = true;
                            };
                            player.send_abilities_update();
                        }
                        'cancelled: {
                            player.send_abilities_update();
                        }
                    }}
                }
            }
            PlayerAction::StopFlying => {
                let flying = {
                    player
                        .abilities
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .flying
                };
                if flying {
                    send_cancellable_blocking! {{
                        server;
                        PlayerToggleFlightEvent::new(player.clone(), false);
                        'after: {
                            {
                                player.abilities.lock().unwrap_or_else(std::sync::PoisonError::into_inner).flying = false;
                            };
                            player.send_abilities_update();
                        }
                        'cancelled: {
                            player.send_abilities_update();
                        }
                    }}
                }
            }
            PlayerAction::StopSleeping => {
                player.wake_up();
            }
            PlayerAction::StartJump => {
                player.increment_stat(
                    pumpkin_data::statistic::StatisticCategory::Custom,
                    pumpkin_data::statistic::CustomStatistic::Jump as i32,
                    1,
                );
            }
            PlayerAction::StopItemUseOn => {
                let item_in_use = player
                    .living_entity
                    .item_in_use
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone();
                if let Some(stack) = item_in_use {
                    server.item_registry.on_stopped_using(&stack, player);
                }

                player.living_entity.clear_active_hand();
            }
            _ => {}
        }
    }
}
