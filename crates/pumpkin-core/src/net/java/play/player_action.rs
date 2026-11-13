#[allow(clippy::wildcard_imports)]
use super::*;

/// Vanilla `ServerPlayerGameMode.handleBlockBreakAction` accepts a finished dig once the
/// break progress reaches 0.7, so a client finish that arrives a little early still counts.
const FINISHED_DIGGING_PROGRESS_THRESHOLD: f32 = 0.7;

impl JavaClient {
    #[expect(clippy::too_many_lines)]
    pub fn handle_player_action(
        &self,
        player: &Arc<Player>,
        player_action: &SPlayerAction,
        server: &Server,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();
        match Status::try_from(player_action.status.0) {
            Ok(status) => match status {
                Status::StartedDigging => {
                    if !player.can_interact_with_block_at(&player_action.position, 1.0) {
                        warn!(
                            "Player {0} tried to interact with block out of reach at {1}",
                            player.gameprofile.name, player_action.position
                        );
                        self.update_sequence(player_action.sequence.0);
                        return;
                    }
                    let position = player_action.position;
                    let entity = &player.get_entity();
                    let world = entity.world.load_full();
                    let (block, state) = world.get_block_and_state(&position);

                    if let Some(server_arc) = world.server.upgrade() {
                        let mut interact_event = PlayerInteractEvent::new(
                            player,
                            InteractAction::LeftClickBlock,
                            block,
                            Some(position),
                        );
                        server_arc
                            .plugin_manager
                            .fire_blocking(&server_arc, &mut interact_event);
                        if interact_event.cancelled {
                            player.try_send_client_packet(&CBlockUpdate::new(
                                position,
                                VarInt(i32::from(state.id.as_u16())),
                            ));
                            self.update_sequence(player_action.sequence.0);
                            return;
                        }

                        let mut event =
                            crate::plugin::api::events::block::block_damage::BlockDamageEvent::new(
                                player.clone(),
                                block,
                                position,
                                false,
                            );
                        server_arc
                            .plugin_manager
                            .fire_blocking(&server_arc, &mut event);
                        if event.cancelled {
                            self.update_sequence(player_action.sequence.0);
                            return;
                        }
                    }

                    let gamemode = player.gamemode.load();
                    let cannot_mine = player.cannot_mine_block(Some(server), block, gamemode);
                    if cannot_mine {
                        player.try_send_client_packet(&CBlockUpdate::new(
                            position,
                            VarInt(i32::from(state.id.as_u16())),
                        ));
                        self.update_sequence(player_action.sequence.0);
                        return;
                    }

                    // TODO: do validation
                    // TODO: Config
                    if gamemode == GameMode::Creative {
                        // Block break & play sound
                        let new_state = world.break_block(
                            &position,
                            Some(player),
                            BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_DROPS,
                        );
                        if new_state.is_some() {
                            server
                                .block_registry
                                .broken(&world, block, player, &position, server, state);
                        }
                        self.sync_block_state_to_client(&world, position);
                        self.update_sequence(player_action.sequence.0);
                        return;
                    }
                    player.start_mining_time.store(
                        player.tick_counter.load(Ordering::Relaxed),
                        Ordering::Relaxed,
                    );
                    if !state.is_air() {
                        server
                            .block_registry
                            .attack(&world, block, state, &position, player);
                        let speed = block::calc_block_breaking(player, state, block);
                        // Instant break
                        if speed >= 1.0 {
                            let broken_state = world.get_block_state(&position);
                            let can_harvest = player.can_harvest(broken_state, block);
                            let flags = if can_harvest {
                                BlockFlags::NOTIFY_ALL
                            } else {
                                BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_ALL
                            };
                            let new_state = world.break_block(&position, Some(player), flags);
                            if new_state.is_some() {
                                server.block_registry.broken(
                                    &world,
                                    block,
                                    player,
                                    &position,
                                    server,
                                    broken_state,
                                );
                                player.apply_tool_damage_for_block_break(broken_state);
                                if can_harvest {
                                    player.add_exhaustion(MINE_BLOCK_EXHAUSTION);
                                }
                                let item_id = player.inventory().held_item().item.id;
                                player.increment_stat(StatisticCategory::Used, item_id as i32, 1);
                                player.increment_stat(
                                    StatisticCategory::Mined,
                                    block.id.as_u16() as i32,
                                    1,
                                );
                            }
                            self.sync_block_state_to_client(&world, position);
                        } else {
                            player.mining.store(true, Ordering::Relaxed);
                            *player
                                .mining_pos
                                .lock()
                                .unwrap_or_else(std::sync::PoisonError::into_inner) = position;
                            let progress = (speed * 10.0) as i32;
                            player
                                .current_block_breaking_speed
                                .store(speed.to_bits(), Ordering::Relaxed);
                            world.set_block_breaking(
                                entity,
                                position,
                                BlockBreakingProgress::Start {
                                    stage: progress,
                                    speed,
                                },
                            );
                            player
                                .current_block_destroy_stage
                                .store(progress, Ordering::Relaxed);
                        }
                    }
                    self.update_sequence(player_action.sequence.0);
                }
                Status::CancelledDigging => {
                    if !player.can_interact_with_block_at(&player_action.position, 1.0) {
                        warn!(
                            "Player {0} tried to interact with block out of reach at {1}",
                            player.gameprofile.name, player_action.position
                        );
                        self.update_sequence(player_action.sequence.0);
                        return;
                    }
                    let entity = &player.get_entity();
                    let world = entity.world.load_full();
                    if let Some(server_arc) = world.server.upgrade() {
                        let mut abort_event = crate::plugin::api::events::block::block_damage_abort::BlockDamageAbortEvent::new(
                            player.clone(),
                            player_action.position,
                            world.clone(),
                            player.inventory().held_item(),
                        );
                        server_arc
                            .plugin_manager
                            .fire_blocking(&server_arc, &mut abort_event);
                    }

                    player.mining.store(false, Ordering::Relaxed);
                    world.set_block_breaking(
                        entity,
                        player_action.position,
                        BlockBreakingProgress::Stop,
                    );
                    self.update_sequence(player_action.sequence.0);
                }
                Status::FinishedDigging => {
                    let location = player_action.position;
                    if !player.can_interact_with_block_at(&location, 1.0) {
                        warn!(
                            "Player {0} tried to interact with block out of reach at {1}",
                            player.gameprofile.name, player_action.position
                        );
                        self.update_sequence(player_action.sequence.0);
                        return;
                    }

                    let entity = &player.get_entity();
                    let world = entity.world.load_full();
                    let gamemode = player.gamemode.load();
                    // Vanilla STOP_DESTROY_BLOCK only applies to the current destroy target.
                    let is_mining_target = player.mining.load(Ordering::Relaxed)
                        && *player
                            .mining_pos
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            == location;
                    if gamemode != GameMode::Creative && !is_mining_target {
                        self.update_sequence(player_action.sequence.0);
                        return;
                    }

                    let Some(state) = world.get_block_state_if_loaded(&location) else {
                        self.update_sequence(player_action.sequence.0);
                        return;
                    };
                    let block = Block::from_state_id(state.id);
                    // Vanilla `getDestroyProgress`, which returns 0 for unbreakable blocks.
                    let speed = block::calc_block_breaking(player, state, block);
                    if state.is_air()
                        || speed <= 0.0
                        || player.cannot_mine_block(Some(server), block, gamemode)
                    {
                        player.stop_mining();
                        self.sync_block_state_to_client(&world, location);
                        self.update_sequence(player_action.sequence.0);
                        return;
                    }

                    if gamemode == GameMode::Creative {
                        player.stop_mining();
                        player.break_block(&location);
                        self.sync_block_state_to_client(&world, location);
                        self.update_sequence(player_action.sequence.0);
                        return;
                    }

                    let progress = speed
                        * (player.tick_counter.load(Ordering::Relaxed)
                            - player.start_mining_time.load(Ordering::Relaxed)
                            + 1) as f32;
                    if progress >= FINISHED_DIGGING_PROGRESS_THRESHOLD {
                        player.stop_mining();
                        player.break_block(&location);
                        self.sync_block_state_to_client(&world, location);
                    } else {
                        // Vanilla `hasDelayedDestroy`: a finish can arrive before the server
                        // tick has caught up, so keep mining until progress reaches 1.0.
                        player.start_delayed_destroy(location);
                    }

                    self.update_sequence(player_action.sequence.0);
                }
                Status::DropItem => {
                    player.drop_held_item(false);
                }
                Status::DropItemStack => {
                    player.drop_held_item(true);
                }
                Status::ReleaseItemInUse => {
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
                Status::SwapItem => {
                    player.swap_item();
                }
                Status::SpearJab => {
                    if player.gamemode.load() == GameMode::Spectator {
                        return;
                    }

                    let stack = player.inventory().held_item();
                    server.item_registry.on_spear_jab(&stack, player);
                }
                // The block being mined does not change, so there is nothing to update
                Status::ChangeDestroyDirection => {}
            },
            Err(_) => self.try_kick(&TextComponent::text("Invalid status")),
        }
    }

    pub fn update_sequence(&self, sequence: i32) {
        if sequence < 0 {
            error!("Expected packet sequence >= 0");
        }
        self.packet_sequence.store(
            self.packet_sequence.load(Ordering::Relaxed).max(sequence),
            Ordering::Relaxed,
        );
    }

    fn sync_block_state_to_client(&self, world: &World, position: BlockPos) {
        let synced_state_id = world.get_block_state_id(&position);
        self.try_send_packet(&CBlockUpdate::new(
            position,
            VarInt(i32::from(synced_state_id.as_u16())),
        ));
    }
}
