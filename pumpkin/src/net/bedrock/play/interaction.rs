use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::net::bedrock::BedrockClient;
use crate::server::Server;
use pumpkin_protocol::bedrock::client::container_open::CContainerOpen;
use pumpkin_protocol::bedrock::server::animate::AnimateAction;
use pumpkin_protocol::bedrock::server::animate::SAnimate;
use pumpkin_protocol::bedrock::server::emote::SEmote;
use pumpkin_protocol::bedrock::server::interaction::Action;
use pumpkin_protocol::bedrock::server::interaction::SInteraction;
use pumpkin_protocol::bedrock::server::player_action::Action as PlayerAction;
use pumpkin_protocol::bedrock::server::player_action::SPlayerAction;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::java::client::play::Animation;
use pumpkin_protocol::java::client::play::CEntityAnimation;
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tracing::debug;

impl BedrockClient {
    pub async fn handle_player_block_action(
        &self,
        player: &Arc<Player>,
        server: &Server,
        packet: pumpkin_protocol::bedrock::server::player_auth_input::PlayerBlockAction,
    ) {
        use pumpkin_protocol::bedrock::server::player_action::Action as PlayerAction;
        let action = PlayerAction::try_from(packet.action.0).unwrap();
        self.handle_player_action(
            player,
            server,
            SPlayerAction {
                runtime_id: VarInt(0), // Unused
                action,
                block_pos: packet.block_pos,
                result_pos: BlockPos::ZERO,
                face: packet.face,
            },
        )
        .await;
    }

    pub async fn handle_animate(&self, player: &Arc<Player>, _server: &Server, packet: &SAnimate) {
        if !player.has_client_loaded() {
            return;
        }

        let entity = &player.get_entity();
        let world = entity.world.load();

        let java_animation = match packet.action {
            AnimateAction::SwingArm => Some(Animation::SwingMainArm),
            AnimateAction::WakeUp => Some(Animation::LeaveBed),
            AnimateAction::CriticalHit => Some(Animation::CriticalEffect),
            AnimateAction::MagicCriticalHit => Some(Animation::MagicCriticaleffect),
            AnimateAction::StopSleep => None, // TODO
        };

        if let Some(animation) = java_animation {
            let je_packet = CEntityAnimation::new(VarInt(entity.entity_id), animation);
            let be_packet = SAnimate {
                action: packet.action,
                runtime_entity_id: VarULong(entity.entity_id as u64),
                data: 0.0,
                swing_source: None,
            };
            world.broadcast_editioned(&je_packet, &be_packet).await;
        }
    }

    pub async fn handle_emote(&self, player: &Arc<Player>, _server: &Server, packet: SEmote<'_>) {
        if !player.has_client_loaded() {
            return;
        }

        let entity = &player.living_entity.entity;
        let world = entity.world.load();

        let mut broadcast_packet = packet;
        broadcast_packet.flags |= pumpkin_protocol::bedrock::server::emote::EMOTE_FLAG_SERVER_SIDE;

        world
            .broadcast_packet_except_editioned(
                &[player.gameprofile.id],
                &CEntityAnimation::new(
                    VarInt(entity.entity_id),
                    Animation::SwingMainArm, // Fallback for Java? Or just ignore
                ),
                &broadcast_packet,
            )
            .await;
    }

    // pub fn handle_emote_list(
    //     &self,
    //     player: &Arc<Player>,
    //     _server: &Server,
    //     packet: &SEmoteList,
    // ) {
    //     debug!(
    //         "Player {} sent emote list: {:?}",
    //         player.gameprofile.name, packet.emote_pieces
    //     );
    // }

    pub async fn handle_interaction(&self, player: &Arc<Player>, packet: SInteraction) {
        match packet.action {
            Action::OpenInventory => {
                if self.inventory_opened.load(Ordering::Relaxed) {
                    return;
                }
                self.inventory_opened.store(true, Ordering::Relaxed);
                self.enqueue_packet(&CContainerOpen {
                    container_id: 0,
                    container_type: 0xff,
                    position: BlockPos::ZERO,
                    target_entity_id: VarLong(-1),
                })
                .await;
            }
            // No longer used in newer versions
            Action::Attack => {
                let target_runtime_id = packet.target_runtime_id.0 as i32;
                let world = player.world();
                if let Some(target) = world.get_entity_by_id(target_runtime_id) {
                    let target_bounds = target.get_entity().bounding_box.load();
                    if player.is_within_entity_interaction_range(&target_bounds, 3.0) {
                        player.attack(target).await;
                    }
                }
            }
            _ => {}
        }
    }
    #[expect(clippy::match_same_arms)]
    #[expect(clippy::too_many_lines)]
    pub async fn handle_player_action(
        &self,
        player: &Arc<Player>,
        server: &Server,
        packet: SPlayerAction,
    ) {
        if !player.has_client_loaded() {
            return;
        }
        player.update_last_action_time();

        match packet.action {
            PlayerAction::StartBreak
            | PlayerAction::CreativePlayerDestroyBlock
            | PlayerAction::ContinueDestroyBlock => {
                let location = packet.block_pos;
                if !player.can_interact_with_block_at(&location, 1.0) {
                    return;
                }

                let entity = &player.get_entity();
                let world = entity.world.load_full();
                let (block, state) = world.get_block_and_state(&location);

                if player.gamemode.load() == GameMode::Creative {
                    let new_state = world
                        .break_block(
                            &location,
                            Some(player.clone()),
                            BlockFlags::NOTIFY_NEIGHBORS | BlockFlags::SKIP_DROPS,
                        )
                        .await;
                    if new_state.is_some() {
                        server
                            .block_registry
                            .broken(&world, block, player, &location, server, state)
                            .await;
                    }
                } else if !state.is_air() {
                    let speed = crate::block::calc_block_breaking(player, state, block).await;
                    if speed >= 1.0 {
                        let broken_state = world.get_block_state(&location);
                        let new_state = world
                            .break_block(
                                &location,
                                Some(player.clone()),
                                BlockFlags::NOTIFY_NEIGHBORS,
                            )
                            .await;
                        if new_state.is_some() {
                            server
                                .block_registry
                                .broken(&world, block, player, &location, server, broken_state)
                                .await;
                            player.apply_tool_damage_for_block_break(broken_state).await;
                        }
                    } else {
                        player.mining.store(true, Ordering::Relaxed);
                        *player.mining_pos.lock().await = location;
                        let progress = (speed * 10.0) as i32;
                        world.set_block_breaking(entity, location, progress).await;
                        player
                            .current_block_destroy_stage
                            .store(progress, Ordering::Relaxed);
                    }
                }
            }
            PlayerAction::PredictDestroyBlock | PlayerAction::StopBreak => {
                let location = packet.block_pos;
                if !player.can_interact_with_block_at(&location, 1.0) {
                    return;
                }

                let entity = &player.get_entity();
                let world = entity.world.load_full();

                player.mining.store(false, Ordering::Relaxed);
                world.set_block_breaking(entity, location, -1).await;

                let (block, state) = world.get_block_and_state(&location);
                if player.gamemode.load() != GameMode::Creative {
                    let block_drop = player.can_harvest(state, block).await;

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
                    }
                }
            }
            PlayerAction::CrackBreak => {
                // Don't do anything for this action. It is no longer used. Block
                // cracking is done fully server-side.
            }
            PlayerAction::AbortBreak => {
                let location = packet.block_pos;
                let entity = &player.get_entity();
                let world = entity.world.load();

                player.mining.store(false, Ordering::Relaxed);
                world.set_block_breaking(entity, location, -1).await;
            }
            PlayerAction::DropItem => {
                player.drop_held_item(false).await;
            }
            // TODO
            _ => {}
        }
    }
    pub async fn handle_request_ability(
        &self,
        player: &Arc<Player>,
        packet: pumpkin_protocol::bedrock::server::request_ability::SRequestAbility,
    ) {
        player.update_last_action_time();
        let ability_id = packet.ability.0;
        match ability_id {
            9 => {
                // Flying
                if let pumpkin_protocol::bedrock::server::request_ability::AbilityValue::Bool(
                    requested_flying,
                ) = packet.value
                {
                    let mut abilities = player.abilities.lock().await;
                    if abilities.allow_flying {
                        abilities.flying = requested_flying;
                    } else {
                        abilities.flying = false;
                    }
                    drop(abilities);
                    player.send_abilities_update().await;
                }
            }
            _ => {
                debug!("Received RequestAbility packet for unhandled ability {ability_id}");
            }
        }
    }
}
