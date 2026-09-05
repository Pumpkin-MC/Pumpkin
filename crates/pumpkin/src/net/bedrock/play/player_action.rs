#[allow(clippy::wildcard_imports)]
use super::*;

impl BedrockClient {
    #[expect(clippy::match_same_arms)]
    #[expect(clippy::too_many_lines)]
    pub fn handle_player_action(
        &self,
        player: &Arc<Player>,
        server: &Server,
        packet: SPlayerAction,
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
                        let starts_breaking =
                            !player.mining.load(Ordering::Relaxed) || *mining_pos != location;
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
                            if block == &pumpkin_data::Block::NOTE_BLOCK {
                                let props =
                                    pumpkin_data::block_properties::NoteBlockLikeProperties::from_state_id(
                                        state.id,
                                    );
                                crate::block::blocks::note::NoteBlock::play_note(
                                    &props, &world, &location,
                                );
                                player.increment_stat(
                                    pumpkin_data::statistic::StatisticCategory::Custom,
                                    pumpkin_data::statistic::CustomStatistic::PlayNoteblock as i32,
                                    1,
                                );
                            }
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
                        self.try_enqueue_client_packet(&CUpdateBlock::new(
                            location,
                            runtime_id as u32,
                        ));
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
            // TODO
            _ => {}
        }
    }
}
