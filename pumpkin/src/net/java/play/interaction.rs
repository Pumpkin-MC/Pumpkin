use crate::block;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::entity::player::statistics::CustomStatistic;
use crate::entity::player::statistics::StatisticCategory;
use crate::net::java::JavaClient;
use crate::plugin::player::player_interact_entity_event::PlayerInteractEntityEvent;
use crate::plugin::player::player_interact_event::InteractAction;
use crate::plugin::player::player_interact_event::PlayerInteractEvent;
use crate::plugin::player::player_interact_unknown_entity_event::PlayerInteractUnknownEntityEvent;
use crate::server::Server;
use crate::world::World;
use pumpkin_data::Block;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::sound::Sound;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::translation;
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CBlockUpdate;
use pumpkin_protocol::java::client::play::CSetCamera;
use pumpkin_protocol::java::server::play::ActionType;
use pumpkin_protocol::java::server::play::SAttack;
use pumpkin_protocol::java::server::play::SInteract;
use pumpkin_protocol::java::server::play::SPlayerAction;
use pumpkin_protocol::java::server::play::SSwingArm;
use pumpkin_protocol::java::server::play::Status;
use pumpkin_util::GameMode;
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tracing::debug;
use tracing::error;
use tracing::warn;

impl JavaClient {
    pub async fn handle_swing_arm(&self, player: &Arc<Player>, swing_arm: SSwingArm) {
        player.update_last_action_time();
        let Ok(hand) = Hand::try_from(swing_arm.hand.0) else {
            self.kick(TextComponent::text("Invalid hand")).await;
            return;
        };

        let (yaw, pitch) = player.rotation();
        let hit_result = player
            .world()
            .raycast(
                player.eye_position(),
                player
                    .eye_position()
                    .add(&(Vector3::rotation_vector(f64::from(pitch), f64::from(yaw)) * 4.5)),
                async |pos, world| {
                    let block = world.get_block(pos);
                    block != &Block::AIR && block != &Block::WATER && block != &Block::LAVA
                },
            )
            .await;

        let event = if let Some((hit_pos, _hit_dir)) = hit_result {
            PlayerInteractEvent::new(
                player,
                InteractAction::LeftClickBlock,
                player.world().get_block(&hit_pos),
                Some(hit_pos),
            )
        } else {
            PlayerInteractEvent::new(player, InteractAction::LeftClickAir, &Block::AIR, None)
        };

        let Some(server) = player.world().server.upgrade() else {
            return;
        };

        send_cancellable! {{
            server;
            event;
            'after: {
                player.swing_hand(hand, false).await;
            }
        }}
    }

    pub async fn handle_attack(&self, player: &Arc<Player>, attack: SAttack, server: &Arc<Server>) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();
        let entity_id = attack.entity_id;
        let player_entity = &player.get_entity();
        let world = player_entity.world.load_full();

        let config = &server.advanced_config.pvp;
        if !config.enabled {
            return;
        }

        if entity_id.0 == player.entity_id() {
            self.kick(TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_ENTITY_ATTACKED,
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_ENTITY_ATTACKED,
                [],
            ))
            .await;
            return;
        }

        let player_target = world.get_player_by_id(entity_id.0);
        let target: Option<Arc<dyn EntityBase>> = player_target
            .as_ref()
            .map(|p| Arc::clone(p) as Arc<dyn EntityBase>)
            .or_else(|| world.get_entity_by_id(entity_id.0));
        let Some(target) = target else {
            self.kick(TextComponent::translate_cross(
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_ENTITY_ATTACKED,
                translation::java::MULTIPLAYER_DISCONNECT_INVALID_ENTITY_ATTACKED,
                [],
            ))
            .await;
            return;
        };
        let target_bounds = target.get_entity().bounding_box.load();
        if !player.is_within_entity_interaction_range(&target_bounds, 3.0) {
            return;
        }
        if let Some(player_victim) = &player_target {
            if player_victim.living_entity.health.load() <= 0.0 {
                return;
            }
            if config.protect_creative && player_victim.gamemode.load() == GameMode::Creative {
                world.play_sound(
                    Sound::EntityPlayerAttackNodamage,
                    SoundCategory::Players,
                    &player_victim.position(),
                );
                return;
            }
        }
        player.attack(target).await;
    }

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
            if action == ActionType::Attack {
                let target_bounds = target.get_entity().bounding_box.load();
                if !player.is_within_entity_interaction_range(&target_bounds, 3.0) {
                    return;
                }
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
                            let held = player.inventory.held_item();
                            let mut stack = held.lock().await.clone();
                            let interacted = event.target.interact(player, &mut stack).await;
                            if !interacted {
                                server
                                    .item_registry
                                    .use_on_entity(&mut stack, player, event.target)
                                    .await;
                            }
                            *held.lock().await = stack;
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

    #[expect(clippy::too_many_lines)]
    pub async fn handle_player_action(
        &self,
        player: &Arc<Player>,
        player_action: SPlayerAction,
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
                        self.update_sequence(player, player_action.sequence.0);
                        return;
                    }
                    let position = player_action.position;
                    let entity = &player.get_entity();
                    let world = entity.world.load_full();
                    let (block, state) = world.get_block_and_state(&position);

                    if block == &pumpkin_data::Block::NOTE_BLOCK {
                        let props =
                            pumpkin_data::block_properties::NoteBlockLikeProperties::from_state_id(
                                state.id, block,
                            );
                        crate::block::blocks::note::NoteBlock::play_note(&props, &world, &position)
                            .await;
                        player
                            .increment_stat(
                                StatisticCategory::Custom,
                                CustomStatistic::PlayNoteblock as i32,
                                1,
                            )
                            .await;
                    }

                    let inventory = player.inventory();
                    let held = inventory.held_item();
                    if !server
                        .item_registry
                        .can_mine(held.lock().await.item, player)
                    {
                        self.enqueue_packet(&CBlockUpdate::new(
                            position,
                            VarInt(i32::from(state.id.as_u16())),
                        ))
                        .await;
                        self.update_sequence(player, player_action.sequence.0);
                        return;
                    }

                    // TODO: do validation
                    // TODO: Config
                    if player.gamemode.load() == GameMode::Creative {
                        // Block break & play sound
                        let new_state = world
                            .break_block(
                                &position,
                                Some(player.clone()),
                                BlockFlags::NOTIFY_NEIGHBORS | BlockFlags::SKIP_DROPS,
                            )
                            .await;
                        if new_state.is_some() {
                            server
                                .block_registry
                                .broken(&world, block, player, &position, server, state)
                                .await;
                        }
                        self.sync_block_state_to_client(&world, position).await;
                        self.update_sequence(player, player_action.sequence.0);
                        return;
                    }
                    player.start_mining_time.store(
                        player.tick_counter.load(Ordering::Relaxed),
                        Ordering::Relaxed,
                    );
                    if !state.is_air() {
                        let speed = block::calc_block_breaking(player, state, block).await;
                        // Instant break
                        if speed >= 1.0 {
                            let broken_state = world.get_block_state(&position);
                            let new_state = world
                                .break_block(
                                    &position,
                                    Some(player.clone()),
                                    BlockFlags::NOTIFY_NEIGHBORS,
                                )
                                .await;
                            if new_state.is_some() {
                                server
                                    .block_registry
                                    .broken(&world, block, player, &position, server, broken_state)
                                    .await;
                                player.apply_tool_damage_for_block_break(broken_state).await;
                                let item_id = player.inventory().held_item().lock().await.item.id;
                                player
                                    .increment_stat(StatisticCategory::Used, item_id as i32, 1)
                                    .await;
                                player
                                    .increment_stat(
                                        StatisticCategory::Mined,
                                        broken_state.id.as_u16() as i32,
                                        1,
                                    )
                                    .await;
                            }
                            self.sync_block_state_to_client(&world, position).await;
                        } else {
                            player.mining.store(true, Ordering::Relaxed);
                            *player.mining_pos.lock().await = position;
                            let progress = (speed * 10.0) as i32;
                            world.set_block_breaking(entity, position, progress).await;
                            player
                                .current_block_destroy_stage
                                .store(progress, Ordering::Relaxed);
                        }
                    }
                    self.update_sequence(player, player_action.sequence.0);
                }
                Status::CancelledDigging => {
                    if !player.can_interact_with_block_at(&player_action.position, 1.0) {
                        warn!(
                            "Player {0} tried to interact with block out of reach at {1}",
                            player.gameprofile.name, player_action.position
                        );
                        self.update_sequence(player, player_action.sequence.0);
                        return;
                    }
                    player.mining.store(false, Ordering::Relaxed);
                    let entity = &player.get_entity();
                    entity
                        .world
                        .load()
                        .set_block_breaking(entity, player_action.position, -1)
                        .await;
                    self.update_sequence(player, player_action.sequence.0);
                }
                Status::FinishedDigging => {
                    // TODO: do validation
                    let location = player_action.position;
                    if !player.can_interact_with_block_at(&location, 1.0) {
                        warn!(
                            "Player {0} tried to interact with block out of reach at {1}",
                            player.gameprofile.name, player_action.position
                        );
                        self.update_sequence(player, player_action.sequence.0);
                        return;
                    }

                    // Block break & play sound
                    let entity = &player.get_entity();
                    let world = entity.world.load_full();

                    player.mining.store(false, Ordering::Relaxed);
                    world.set_block_breaking(entity, location, -1).await;

                    let (block, state) = world.get_block_and_state(&location);
                    let block_drop = player.gamemode.load() != GameMode::Creative
                        && player.can_harvest(state, block).await;

                    let new_state = world
                        .break_block(
                            &location,
                            Some(player.clone()),
                            if block_drop {
                                BlockFlags::NOTIFY_NEIGHBORS
                            } else {
                                BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_NEIGHBORS
                            },
                        )
                        .await;
                    if new_state.is_some() {
                        server
                            .block_registry
                            .broken(&world, block, player, &location, server, state)
                            .await;

                        player.apply_tool_damage_for_block_break(state).await;
                        let item_id = player.inventory().held_item().lock().await.item.id;
                        player
                            .increment_stat(StatisticCategory::Used, item_id as i32, 1)
                            .await;
                        player
                            .increment_stat(StatisticCategory::Mined, state.id.as_u16() as i32, 1)
                            .await;
                    }

                    self.sync_block_state_to_client(&world, location).await;

                    self.update_sequence(player, player_action.sequence.0);
                }
                Status::DropItem => {
                    player.drop_held_item(false).await;
                }
                Status::DropItemStack => {
                    player.drop_held_item(true).await;
                }
                Status::ReleaseItemInUse => {
                    let item_in_use = player.living_entity.item_in_use.lock().await.clone();
                    if let Some(stack) = item_in_use {
                        server.item_registry.on_stopped_using(&stack, player).await;
                    }

                    player.living_entity.clear_active_hand().await;
                }
                Status::SwapItem => {
                    player.swap_item().await;
                }
                Status::SpearJab => {
                    debug!("todo");
                }
            },
            Err(_) => self.kick(TextComponent::text("Invalid status")).await,
        }
    }

    pub fn update_sequence(&self, _player: &Player, sequence: i32) {
        if sequence < 0 {
            error!("Expected packet sequence >= 0");
        }
        self.packet_sequence.store(
            self.packet_sequence.load(Ordering::Relaxed).max(sequence),
            Ordering::Relaxed,
        );
    }

    async fn sync_block_state_to_client(&self, world: &World, position: BlockPos) {
        world
            .enqueue_block_state_corrections(self, &[position])
            .await;
    }
}
