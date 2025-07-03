use std::num::NonZeroU8;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{SystemTime, UNIX_EPOCH};

use pumpkin_util::PermissionLvl;
use rsa::pkcs1v15::{Signature as RsaPkcs1v15Signature, VerifyingKey};
use rsa::signature::Verifier;
use sha1::Sha1;
use thiserror::Error;

use pumpkin_config::{BASIC_CONFIG, advanced_config};
use pumpkin_data::block_properties::{
    BlockProperties, WaterLikeProperties, get_block_by_item, get_state_by_state_id,
};
use pumpkin_data::entity::{EntityType, entity_from_egg};
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_inventory::InventoryError;
use pumpkin_inventory::equipment_slot::EquipmentSlot;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::ScreenHandler;
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{
    Animation, CBlockUpdate, CCommandSuggestions, CEntityAnimation, CEntityPositionSync, CHeadRot,
    COpenSignEditor, CPingResponse, CPlayerInfoUpdate, CPlayerPosition, CSetSelectedSlot,
    CSystemChatMessage, CUpdateEntityPos, CUpdateEntityPosRot, CUpdateEntityRot, InitChat,
    PlayerAction,
};
use pumpkin_protocol::java::server::play::{
    Action, ActionType, CommandBlockMode, FLAG_ON_GROUND, SChangeGameMode, SChatCommand,
    SChatMessage, SChunkBatch, SClientCommand, SClientInformationPlay, SCloseContainer,
    SCommandSuggestion, SConfirmTeleport, SCookieResponse as SPCookieResponse, SInteract,
    SKeepAlive, SPickItemFromBlock, SPlayPingRequest, SPlayerAbilities, SPlayerAction,
    SPlayerCommand, SPlayerInput, SPlayerPosition, SPlayerPositionRotation, SPlayerRotation,
    SPlayerSession, SSetCommandBlock, SSetCreativeSlot, SSetHeldItem, SSetPlayerGround, SSwingArm,
    SUpdateSign, SUseItem, SUseItemOn, Status,
};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::{polynomial_rolling_hash, position::BlockPos, wrap_degrees};
use pumpkin_util::text::color::NamedColor;
use pumpkin_util::{GameMode, text::TextComponent};
use pumpkin_world::block::entities::BlockEntity;
use pumpkin_world::block::entities::command_block::CommandBlockEntity;
use pumpkin_world::block::entities::sign::SignBlockEntity;
use pumpkin_world::item::ItemStack;
use pumpkin_world::world::BlockFlags;
use uuid::Uuid;

use crate::block::registry::BlockActionResult;
use crate::block::{self, BlockIsReplacing};
use crate::command::CommandSender;
use crate::entity::EntityBase;
use crate::entity::player::{ChatMode, ChatSession, Hand, Player};
use crate::entity::r#type::from_type;
use crate::error::PumpkinError;
use crate::net::PlayerConfig;
use crate::plugin::player::player_chat::PlayerChatEvent;
use crate::plugin::player::player_command_send::PlayerCommandSendEvent;
use crate::plugin::player::player_interact_event::{InteractAction, PlayerInteractEvent};
use crate::plugin::player::player_move::PlayerMoveEvent;
use crate::server::{Server, seasonal_events};
use crate::world::{World, chunker};

/// In secure chat mode, Player will be kicked if they send a chat message with a timestamp that is older than this (in ms)
/// Vanilla: 2 minutes
const CHAT_MESSAGE_MAX_AGE: i64 = 1000 * 60 * 2;

#[derive(Debug, Error)]
pub enum BlockPlacingError {
    BlockOutOfReach,
    InvalidBlockFace,
    BlockOutOfWorld,
    InvalidGamemode,
}

impl std::fmt::Display for BlockPlacingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl PumpkinError for BlockPlacingError {
    fn is_kick(&self) -> bool {
        match self {
            Self::BlockOutOfReach | Self::BlockOutOfWorld | Self::InvalidGamemode => false,
            Self::InvalidBlockFace => true,
        }
    }

    fn severity(&self) -> log::Level {
        match self {
            Self::BlockOutOfWorld | Self::InvalidGamemode => log::Level::Trace,
            Self::BlockOutOfReach | Self::InvalidBlockFace => log::Level::Warn,
        }
    }

    fn client_kick_reason(&self) -> Option<String> {
        match self {
            Self::BlockOutOfReach | Self::BlockOutOfWorld | Self::InvalidGamemode => None,
            Self::InvalidBlockFace => Some("Invalid block face".into()),
        }
    }
}

#[derive(Debug, Error)]
pub enum ChatError {
    #[error("sent an oversized message")]
    OversizedMessage,
    #[error("sent a message with illegal characters")]
    IllegalCharacters,
    #[error("sent a chat with invalid/no signature")]
    UnsignedChat,
    #[error("has too many unacknowledged chats queued")]
    TooManyPendingChats,
    #[error("sent a chat that couldn't be validated")]
    ChatValidationFailed,
    #[error("sent a chat with an out of order timestamp")]
    OutOfOrderChat,
    #[error("has an expired public key")]
    ExpiredPublicKey,
    #[error("attempted to initialize a session with an invalid public key")]
    InvalidPublicKey,
}

impl PumpkinError for ChatError {
    fn is_kick(&self) -> bool {
        true
    }

    fn severity(&self) -> log::Level {
        log::Level::Warn
    }

    fn client_kick_reason(&self) -> Option<String> {
        match self {
            Self::OversizedMessage => Some("Chat message too long".into()),
            Self::IllegalCharacters => Some(
                TextComponent::translate("multiplayer.disconnect.illegal_characters", [])
                    .get_text(),
            ),
            Self::UnsignedChat => Some(
                TextComponent::translate("multiplayer.disconnect.unsigned_chat", []).get_text(),
            ),
            Self::TooManyPendingChats => Some(
                TextComponent::translate("multiplayer.disconnect.too_many_pending_chats", [])
                    .get_text(),
            ),
            Self::ChatValidationFailed => Some(
                TextComponent::translate("multiplayer.disconnect.chat_validation_failed", [])
                    .get_text(),
            ),
            Self::OutOfOrderChat => Some(
                TextComponent::translate("multiplayer.disconnect.out_of_order_chat", []).get_text(),
            ),
            Self::ExpiredPublicKey => Some(
                TextComponent::translate("multiplayer.disconnect.expired_public_key", [])
                    .get_text(),
            ),
            Self::InvalidPublicKey => Some(
                TextComponent::translate("multiplayer.disconnect.invalid_public_key_signature", [])
                    .get_text(),
            ),
        }
    }
}

/// Handles all Play packets sent by a real player.
/// NEVER TRUST THE CLIENT. HANDLE EVERY ERROR; UNWRAP/EXPECT ARE FORBIDDEN.
impl Player {
    pub async fn handle_confirm_teleport(&self, confirm_teleport: SConfirmTeleport) {
        let mut awaiting_teleport = self.awaiting_teleport.lock().await;
        if let Some((id, position)) = awaiting_teleport.as_ref() {
            if id == &confirm_teleport.teleport_id {
                // We should set the position now to what we requested in the teleport packet.
                // This may fix issues when the client sends the position while being teleported.
                self.living_entity.set_pos(*position);

                *awaiting_teleport = None;
            } else {
                self.kick(TextComponent::text("Wrong teleport id")).await;
            }
        } else {
            self.kick(TextComponent::text(
                "Send Teleport confirm, but we did not teleport",
            ))
            .await;
        }
    }

    pub async fn handle_change_game_mode(self: &Arc<Self>, change_game_mode: SChangeGameMode) {
        if self.permission_lvl.load() >= PermissionLvl::Two {
            self.set_gamemode(change_game_mode.game_mode).await;
            let gamemode_string = format!("{:?}", change_game_mode.game_mode).to_lowercase();
            let gamemode_string = format!("gameMode.{gamemode_string}");
            self.send_system_message(&TextComponent::translate(
                "commands.gamemode.success.self",
                [TextComponent::translate(gamemode_string, [])],
            ))
            .await;
        }
    }

    fn clamp_horizontal(pos: f64) -> f64 {
        pos.clamp(-3.0E7, 3.0E7)
    }

    fn clamp_vertical(pos: f64) -> f64 {
        pos.clamp(-2.0E7, 2.0E7)
    }

    pub fn handle_player_loaded(self: &Arc<Self>) {
        self.set_client_loaded(true);
    }

    /// Returns whether syncing the position was needed
    async fn sync_position(
        &self,
        world: &World,
        pos: Vector3<f64>,
        last_pos: Vector3<f64>,
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    ) -> bool {
        let delta = Vector3::new(pos.x - last_pos.x, pos.y - last_pos.y, pos.z - last_pos.z);
        let entity_id = self.entity_id();

        // Teleport when more than 8 blocks (-8..=7.999755859375) (checking 8²)
        if delta.length_squared() < 64.0 {
            return false;
        }
        // Sync position with all other players.
        world
            .broadcast_packet_except(
                &[self.gameprofile.id],
                &CEntityPositionSync::new(
                    entity_id.into(),
                    pos,
                    Vector3::new(0.0, 0.0, 0.0),
                    yaw,
                    pitch,
                    on_ground,
                ),
            )
            .await;
        true
    }

    pub async fn handle_position(self: &Arc<Self>, packet: SPlayerPosition) {
        if !self.has_client_loaded() {
            return;
        }
        // y = feet Y
        let position = packet.position;
        if position.x.is_nan() || position.y.is_nan() || position.z.is_nan() {
            self.kick(TextComponent::translate(
                "multiplayer.disconnect.invalid_player_movement",
                [],
            ))
            .await;
            return;
        }
        let position = Vector3::new(
            Self::clamp_horizontal(position.x),
            Self::clamp_vertical(position.y),
            Self::clamp_horizontal(position.z),
        );

        send_cancellable! {{
            PlayerMoveEvent {
                player: self.clone(),
                from: self.living_entity.entity.pos.load(),
                to: position,
                cancelled: false,
            };

            'after: {
                let pos = event.to;
                let entity = &self.living_entity.entity;
                let last_pos = entity.pos.load();
                self.living_entity.set_pos(pos);

                let height_difference = pos.y - last_pos.y;
                if entity.on_ground.load(Ordering::Relaxed) && packet.collision & FLAG_ON_GROUND == 0 && height_difference > 0.0 {
                    self.jump().await;
                }

                entity.on_ground.store(packet.collision & FLAG_ON_GROUND != 0, Ordering::Relaxed);
                let world = &self.world().await;

                // TODO: Warn when player moves to quickly
                if !self.sync_position(world, pos, last_pos, entity.yaw.load(), entity.pitch.load(), packet.collision & FLAG_ON_GROUND != 0).await {
                    // Send the new position to all other players.
                    world
                        .broadcast_packet_except(
                            &[self.gameprofile.id],
                            &CUpdateEntityPos::new(
                                self.entity_id().into(),
                                Vector3::new(
                                    pos.x.mul_add(4096.0, -(last_pos.x * 4096.0)) as i16,
                                    pos.y.mul_add(4096.0, -(last_pos.y * 4096.0)) as i16,
                                    pos.z.mul_add(4096.0, -(last_pos.z * 4096.0)) as i16,
                                ),
                                packet.collision & FLAG_ON_GROUND != 0,
                            ),
                        )
                        .await;
                }

                if !self.abilities.lock().await.flying {
                    self.living_entity
                        .update_fall_distance(
                            height_difference,
                            packet.collision & FLAG_ON_GROUND != 0,
                            self.gamemode.load() == GameMode::Creative,
                        )
                        .await;
                }
                chunker::update_position(self).await;
                self.progress_motion(Vector3::new(
                    pos.x - last_pos.x,
                    pos.y - last_pos.y,
                    pos.z - last_pos.z,
                ))
                .await;
            }

            'cancelled: {
                self.client.enqueue_packet(&CPlayerPosition::new(
                    self.teleport_id_count.load(std::sync::atomic::Ordering::Relaxed).into(),
                    self.living_entity.entity.pos.load(),
                    Vector3::new(0.0, 0.0, 0.0),
                    self.living_entity.entity.yaw.load(),
                    self.living_entity.entity.pitch.load(),
                    &[],
                )).await;
            }
        }}
    }

    pub async fn handle_position_rotation(self: &Arc<Self>, packet: SPlayerPositionRotation) {
        if !self.has_client_loaded() {
            return;
        }
        // y = feet Y
        let position = packet.position;
        if !position.x.is_finite()
            || !position.y.is_finite()
            || !position.z.is_finite()
            || !packet.yaw.is_finite()
            || !packet.pitch.is_finite()
        {
            self.kick(TextComponent::translate(
                "multiplayer.disconnect.invalid_player_movement",
                [],
            ))
            .await;
            return;
        }

        let position = Vector3::new(
            Self::clamp_horizontal(position.x),
            Self::clamp_vertical(position.y),
            Self::clamp_horizontal(position.z),
        );

        send_cancellable! {{
            PlayerMoveEvent::new(
                self.clone(),
                self.living_entity.entity.pos.load(),
                position,
            );

            'after: {
                let pos = event.to;
                let entity = &self.living_entity.entity;
                let last_pos = entity.pos.load();
                self.living_entity.set_pos(pos);

                let height_difference = pos.y - last_pos.y;
                if entity.on_ground.load(std::sync::atomic::Ordering::Relaxed)
                    && (packet.collision & FLAG_ON_GROUND) != 0
                    && height_difference > 0.0
                {
                    self.jump().await;
                }
                entity
                    .on_ground
                    .store((packet.collision & FLAG_ON_GROUND) != 0, std::sync::atomic::Ordering::Relaxed);

                entity.set_rotation(wrap_degrees(packet.yaw) % 360.0, wrap_degrees(packet.pitch));

                let entity_id = entity.entity_id;

                let yaw = (entity.yaw.load() * 256.0 / 360.0).rem_euclid(256.0);
                let pitch = (entity.pitch.load() * 256.0 / 360.0).rem_euclid(256.0);
                // let head_yaw = (entity.head_yaw * 256.0 / 360.0).floor();
                let world = &entity.world.read().await;

                // TODO: Warn when player moves to quickly
                if !self
                    .sync_position(world, pos, last_pos, yaw, pitch, (packet.collision & FLAG_ON_GROUND) != 0)
                    .await
                {
                    // Send the new position to all other players.
                    world
                        .broadcast_packet_except(
                            &[self.gameprofile.id],
                            &CUpdateEntityPosRot::new(
                                entity_id.into(),
                                Vector3::new(
                                    pos.x.mul_add(4096.0, -(last_pos.x * 4096.0)) as i16,
                                    pos.y.mul_add(4096.0, -(last_pos.y * 4096.0)) as i16,
                                    pos.z.mul_add(4096.0, -(last_pos.z * 4096.0)) as i16,
                                ),
                                yaw as u8,
                                pitch as u8,
                                (packet.collision & FLAG_ON_GROUND) != 0,
                            ),
                        )
                        .await;
                }

                world
                    .broadcast_packet_except(
                        &[self.gameprofile.id],
                        &CHeadRot::new(entity_id.into(), yaw as u8),
                    )
                    .await;
                if !self.abilities.lock().await.flying {
                    self.living_entity
                        .update_fall_distance(
                            height_difference,
                            (packet.collision & FLAG_ON_GROUND) != 0,
                            self.gamemode.load() == GameMode::Creative,
                        )
                        .await;
                }
                chunker::update_position(self).await;
                self.progress_motion(Vector3::new(
                    pos.x - last_pos.x,
                    pos.y - last_pos.y,
                    pos.z - last_pos.z,
                ))
                .await;
            }

            'cancelled: {
                self.force_tp(position).await;
            }
        }}
    }

    pub async fn force_tp(&self, position: Vector3<f64>) {
        let teleport_id = self
            .teleport_id_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        *self.awaiting_teleport.lock().await = Some((teleport_id.into(), position));
        self.client
            .enqueue_packet(&CPlayerPosition::new(
                teleport_id.into(),
                self.living_entity.entity.pos.load(),
                Vector3::new(0.0, 0.0, 0.0),
                self.living_entity.entity.yaw.load(),
                self.living_entity.entity.pitch.load(),
                &[],
            ))
            .await;
    }

    pub async fn handle_rotation(&self, rotation: SPlayerRotation) {
        if !self.has_client_loaded() {
            return;
        }
        if !rotation.yaw.is_finite() || !rotation.pitch.is_finite() {
            self.kick(TextComponent::translate(
                "multiplayer.disconnect.invalid_player_movement",
                [],
            ))
            .await;
            return;
        }
        let entity = &self.living_entity.entity;
        entity
            .on_ground
            .store(rotation.ground, std::sync::atomic::Ordering::Relaxed);
        entity.set_rotation(
            wrap_degrees(rotation.yaw) % 360.0,
            wrap_degrees(rotation.pitch),
        );
        // Send the new position to all other players.
        let entity_id = entity.entity_id;
        let yaw = (entity.yaw.load() * 256.0 / 360.0).rem_euclid(256.0);
        let pitch = (entity.pitch.load() * 256.0 / 360.0).rem_euclid(256.0);
        // let head_yaw = modulus(entity.head_yaw * 256.0 / 360.0, 256.0);

        let world = &entity.world.read().await;
        let packet =
            CUpdateEntityRot::new(entity_id.into(), yaw as u8, pitch as u8, rotation.ground);
        world
            .broadcast_packet_except(&[self.gameprofile.id], &packet)
            .await;
        let packet = CHeadRot::new(entity_id.into(), yaw as u8);
        world
            .broadcast_packet_except(&[self.gameprofile.id], &packet)
            .await;
    }

    pub async fn handle_chat_command(
        self: &Arc<Self>,
        server: &Arc<Server>,
        command: &SChatCommand,
    ) {
        let player_clone = self.clone();
        let server_clone = server.clone();
        send_cancellable! {{
            PlayerCommandSendEvent {
                player: self.clone(),
                command: command.command.clone(),
                cancelled: false
            };

            'after: {
                let command = event.command;
                let command_clone = command.clone();
                // Some commands can take a long time to execute. If they do, they block packet processing for the player.
                // That's why we will spawn a task instead.
                server.spawn_task(async move {
                    let dispatcher = server_clone.command_dispatcher.read().await;
                    dispatcher
                        .handle_command(
                            &mut CommandSender::Player(player_clone),
                            &server_clone,
                            &command_clone,
                        )
                        .await;
                });

                if advanced_config().commands.log_console {
                    log::info!(
                        "Player ({}): executed command /{}",
                        self.gameprofile.name,
                        command
                    );
                }
            }
        }}
    }

    pub fn handle_player_ground(&self, ground: &SSetPlayerGround) {
        self.living_entity
            .entity
            .on_ground
            .store(ground.on_ground, std::sync::atomic::Ordering::Relaxed);
    }

    pub async fn handle_pick_item_from_block(&self, pick_item: SPickItemFromBlock) {
        if !self.can_interact_with_block_at(&pick_item.pos, 1.0) {
            return;
        }

        let world = self.world().await;
        let block = world.get_block(&pick_item.pos).await;

        if block.item_id == 0 {
            // Invalid block id (blocks such as tall seagrass)
            return;
        }

        let stack = ItemStack::new(1, Item::from_id(block.item_id).unwrap());

        let slot_with_stack = self.inventory().get_slot_with_stack(&stack).await;

        if slot_with_stack != -1 {
            if PlayerInventory::is_valid_hotbar_index(slot_with_stack as usize) {
                self.inventory.set_selected_slot(slot_with_stack as u8);
            } else {
                self.inventory
                    .swap_slot_with_hotbar(slot_with_stack as usize)
                    .await;
            }
        } else if self.gamemode.load() == GameMode::Creative {
            self.inventory.swap_stack_with_hotbar(stack).await;
        }

        self.client
            .enqueue_packet(&CSetSelectedSlot::new(
                self.inventory.get_selected_slot() as i8
            ))
            .await;
        self.player_screen_handler
            .lock()
            .await
            .send_content_updates()
            .await;
    }

    // pub fn handle_pick_item_from_entity(&self, _pick_item: SPickItemFromEntity) {
    //     // TODO: Implement and merge any redundant code with pick_item_from_block
    // }

    pub async fn handle_set_command_block(&self, command: SSetCommandBlock) {
        // TODO: check things
        let pos = command.pos;
        if let Some((nbt, block_entity)) = self.world().await.get_block_entity(&pos).await {
            let command_entity = CommandBlockEntity::from_nbt(&nbt, pos);

            if block_entity.resource_location() != command_entity.resource_location() {
                log::warn!(
                    "Client tried to change Command block but not Command block entity found"
                );
                return;
            }

            let Ok(command_block_mode) = CommandBlockMode::try_from(command.mode) else {
                self.kick(TextComponent::text("Invalid Command block mode"))
                    .await;
                return;
            };

            let _block_state = match command_block_mode {
                CommandBlockMode::Chain => Block::CHAIN_COMMAND_BLOCK,
                CommandBlockMode::Repeating => Block::REPEATING_COMMAND_BLOCK,
                CommandBlockMode::Impulse => Block::COMMAND_BLOCK,
            };
        }
    }

    pub async fn handle_player_command(&self, command: SPlayerCommand) {
        if command.entity_id != self.entity_id().into() {
            return;
        }
        if !self.has_client_loaded() {
            return;
        }

        if let Ok(action) = Action::try_from(command.action.0) {
            let entity = &self.living_entity.entity;
            match action {
                pumpkin_protocol::java::server::play::Action::StartSprinting => {
                    if !entity.sprinting.load(std::sync::atomic::Ordering::Relaxed) {
                        entity.set_sprinting(true).await;
                    }
                }
                pumpkin_protocol::java::server::play::Action::StopSprinting => {
                    if entity.sprinting.load(std::sync::atomic::Ordering::Relaxed) {
                        entity.set_sprinting(false).await;
                    }
                }
                pumpkin_protocol::java::server::play::Action::LeaveBed => self.wake_up().await,

                pumpkin_protocol::java::server::play::Action::StartHorseJump
                | pumpkin_protocol::java::server::play::Action::StopHorseJump
                | pumpkin_protocol::java::server::play::Action::OpenVehicleInventory => {
                    log::debug!("todo");
                }
                pumpkin_protocol::java::server::play::Action::StartFlyingElytra => {
                    let fall_flying = entity.check_fall_flying();
                    if entity
                        .fall_flying
                        .load(std::sync::atomic::Ordering::Relaxed)
                        != fall_flying
                    {
                        entity.set_fall_flying(fall_flying).await;
                    }
                } // TODO
            }
        } else {
            self.kick(TextComponent::text("Invalid player command"))
                .await;
        }
    }

    pub async fn handle_player_input(&self, input: SPlayerInput) {
        let sneak = input.input & SPlayerInput::SNEAK != 0;
        if self
            .get_entity()
            .sneaking
            .load(std::sync::atomic::Ordering::Relaxed)
            != sneak
        {
            self.get_entity().set_sneaking(sneak).await;
        }
    }

    pub async fn handle_swing_arm(self: &Arc<Self>, swing_arm: SSwingArm) {
        let animation = match swing_arm.hand.0 {
            0 => Animation::SwingMainArm,
            1 => Animation::SwingOffhand,
            _ => {
                self.kick(TextComponent::text("Invalid hand")).await;
                return;
            }
        };
        // Invert hand if player is left handed
        let animation = match self.config.read().await.main_hand {
            Hand::Left => match animation {
                Animation::SwingMainArm => Animation::SwingOffhand,
                Animation::SwingOffhand => Animation::SwingMainArm,
                _ => unreachable!(),
            },
            Hand::Right => animation,
        };

        let id = self.entity_id();
        let world = self.world().await;

        let inventory = self.inventory();
        let item = inventory.held_item();

        let (yaw, pitch) = self.rotation();
        let hit_result = self
            .world()
            .await
            .raycast(
                self.eye_position(),
                self.eye_position()
                    .add(&(Vector3::rotation_vector(f64::from(pitch), f64::from(yaw)) * 4.5)),
                async |pos, world| {
                    let block = world.get_block(pos).await;
                    block != &Block::AIR && block != &Block::WATER && block != &Block::LAVA
                },
            )
            .await;

        let event = if let Some((hit_pos, _hit_dir)) = hit_result {
            PlayerInteractEvent::new(
                self,
                InteractAction::LeftClickBlock,
                &item,
                self.world().await.get_block(&hit_pos).await,
                Some(hit_pos),
            )
        } else {
            PlayerInteractEvent::new(self, InteractAction::LeftClickAir, &item, &Block::AIR, None)
        };

        send_cancellable! {{
            event;
            'after: {
                world
                    .broadcast_packet_except(
                        &[self.gameprofile.id],
                        &CEntityAnimation::new(id.into(), animation),
                    )
                    .await;
            }
        }}
    }

    pub async fn handle_chat_message(self: &Arc<Self>, chat_message: SChatMessage) {
        let gameprofile = &self.gameprofile;

        if let Err(err) = self.validate_chat_message(&chat_message).await {
            log::log!(
                err.severity(),
                "{} (uuid {}) {}",
                gameprofile.name,
                gameprofile.id,
                err
            );
            if err.is_kick() {
                if let Some(reason) = err.client_kick_reason() {
                    self.kick(TextComponent::text(reason)).await;
                }
            }
            return;
        }

        send_cancellable! {{
            PlayerChatEvent::new(self.clone(), chat_message.message.clone(), vec![]);

            'after: {
                log::info!("<chat> {}: {}", gameprofile.name, event.message);

                let config = advanced_config();

                let message = match seasonal_events::modify_chat_message(&event.message) {
                    Some(m) => m,
                    None => event.message.clone(),
                };

                let decorated_message = &TextComponent::chat_decorated(
                    config.chat.format.clone(),
                    gameprofile.name.clone(),
                    message,
                );

                let entity = &self.living_entity.entity;
                let world = &entity.world.read().await;
                if BASIC_CONFIG.allow_chat_reports {
                    world.broadcast_secure_player_chat(self, &chat_message, decorated_message).await;
                } else {
                    let no_reports_packet = &CSystemChatMessage::new(
                        decorated_message,
                        false,
                    );
                    world.broadcast_packet_all(no_reports_packet).await;
                }
            }
        }}
    }

    /// Runs all vanilla checks for a valid chat message
    pub async fn validate_chat_message(
        &self,
        chat_message: &SChatMessage,
    ) -> Result<(), ChatError> {
        // Check for oversized messages
        if chat_message.message.len() > 256 {
            return Err(ChatError::OversizedMessage);
        }
        // Check for illegal characters
        if chat_message
            .message
            .chars()
            .any(|c| c == '§' || c < ' ' || c == '\x7F')
        {
            return Err(ChatError::IllegalCharacters);
        }
        // These checks are only run in secure chat mode
        if BASIC_CONFIG.allow_chat_reports {
            // Check for unsigned chat
            if let Some(signature) = &chat_message.signature {
                if signature.len() != 256 {
                    return Err(ChatError::UnsignedChat); // Signature is the wrong length
                }
            } else {
                return Err(ChatError::UnsignedChat); // There is no signature
            }

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;

            // Verify message timestamp
            if chat_message.timestamp > now || chat_message.timestamp < (now - CHAT_MESSAGE_MAX_AGE)
            {
                return Err(ChatError::OutOfOrderChat);
            }

            // Verify session expiry
            if self.chat_session.lock().await.expires_at < now {
                return Err(ChatError::ExpiredPublicKey);
            }

            // Validate previous signature checksum (new in 1.21.5)
            // The client can bypass this check by sending 0
            if chat_message.checksum != 0 {
                let checksum =
                    polynomial_rolling_hash(self.signature_cache.lock().await.last_seen.as_ref());
                if checksum != chat_message.checksum {
                    return Err(ChatError::ChatValidationFailed);
                }
            }
        }
        Ok(())
    }

    pub async fn handle_chat_session_update(
        self: &Arc<Self>,
        server: &Server,
        session: SPlayerSession,
    ) {
        // Keep the chat session default if we don't want reports
        if !BASIC_CONFIG.allow_chat_reports {
            return;
        }

        if let Err(err) = self.validate_chat_session(server, &session).await {
            log::log!(
                err.severity(),
                "{} (uuid {}) {}",
                self.gameprofile.name,
                self.gameprofile.id,
                err
            );
            if err.is_kick() {
                if let Some(reason) = err.client_kick_reason() {
                    self.kick(TextComponent::text(reason)).await;
                }
            }
            return;
        }

        // Update the chat session fields
        let mut chat_session = self.chat_session.lock().await; // Await the lock

        // Update the chat session fields
        *chat_session = ChatSession::new(
            session.session_id,
            session.expires_at,
            session.public_key.clone(),
            session.key_signature.clone(),
        );

        server
            .broadcast_packet_all(&CPlayerInfoUpdate::new(
                0x02,
                &[pumpkin_protocol::java::client::play::Player {
                    uuid: self.gameprofile.id,
                    actions: &[PlayerAction::InitializeChat(Some(InitChat {
                        session_id: session.session_id,
                        expires_at: session.expires_at,
                        public_key: session.public_key.clone(),
                        signature: session.key_signature.clone(),
                    }))],
                }],
            ))
            .await;
    }

    /// Runs vanilla checks for a valid player session
    pub async fn validate_chat_session(
        &self,
        server: &Server,
        session: &SPlayerSession,
    ) -> Result<(), ChatError> {
        // Verify session expiry
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;
        if session.expires_at < now {
            return Err(ChatError::InvalidPublicKey);
        }

        // Verify signature with RSA-SHA1
        let mojang_verifying_keys = server
            .mojang_public_keys
            .lock()
            .await
            .iter()
            .map(|key| VerifyingKey::<Sha1>::new(key.clone()))
            .collect::<Vec<_>>();

        let key_signature = RsaPkcs1v15Signature::try_from(session.key_signature.as_ref())
            .map_err(|_| ChatError::InvalidPublicKey)?;

        let mut signable = Vec::new();
        signable.extend_from_slice(self.gameprofile.id.as_bytes());
        signable.extend_from_slice(&session.expires_at.to_be_bytes());
        signable.extend_from_slice(&session.public_key);

        // Verify that the signable is valid for any one of Mojang's public keys
        if !mojang_verifying_keys
            .iter()
            .any(|key| key.verify(&signable, &key_signature).is_ok())
        {
            return Err(ChatError::InvalidPublicKey);
        }

        Ok(())
    }

    pub async fn handle_client_information(
        self: &Arc<Self>,
        client_information: SClientInformationPlay,
    ) {
        if let (Ok(main_hand), Ok(chat_mode)) = (
            Hand::try_from(client_information.main_hand.0),
            ChatMode::try_from(client_information.chat_mode.0),
        ) {
            if client_information.view_distance <= 0 {
                self.kick(TextComponent::text(
                    "Cannot have zero or negative view distance!",
                ))
                .await;
                return;
            }

            let (update_settings, update_watched) = {
                let mut config = self.config.write().await;
                let update_settings = config.main_hand != main_hand
                    || config.skin_parts != client_information.skin_parts;

                let old_view_distance = config.view_distance;

                let update_watched =
                    if old_view_distance.get() == client_information.view_distance as u8 {
                        false
                    } else {
                        log::debug!(
                            "Player {} ({}) updated their render distance: {} -> {}.",
                            self.gameprofile.name,
                            self.client.id,
                            old_view_distance,
                            client_information.view_distance
                        );

                        true
                    };

                *config = PlayerConfig {
                    locale: client_information.locale,
                    // A negative view distance would be impossible and makes no sense, right? Mojang: Let's make it signed :D
                    // client_information.view_distance was checked above to be > 0, so compiler should optimize this out.
                    view_distance: match NonZeroU8::new(client_information.view_distance as u8) {
                        Some(dist) => dist,
                        None => {
                            // Unreachable branch
                            return;
                        }
                    },
                    chat_mode,
                    chat_colors: client_information.chat_colors,
                    skin_parts: client_information.skin_parts,
                    main_hand,
                    text_filtering: client_information.text_filtering,
                    server_listing: client_information.server_listing,
                };
                (update_settings, update_watched)
            };

            if update_watched {
                chunker::update_position(self).await;
            }

            if update_settings {
                log::debug!(
                    "Player {} ({}) updated their skin.",
                    self.gameprofile.name,
                    self.client.id,
                );
                self.send_client_information().await;
            }
        } else {
            self.kick(TextComponent::text("Invalid hand or chat type"))
                .await;
        }
    }

    pub async fn handle_client_status(self: &Arc<Self>, client_status: SClientCommand) {
        match client_status.action_id.0 {
            0 => {
                // Perform respawn
                if self.living_entity.health.load() > 0.0 {
                    return;
                }
                self.world().await.respawn_player(self, false).await;

                let screen_handler = self.current_screen_handler.lock().await;
                let mut screen_handler = screen_handler.lock().await;
                screen_handler.sync_state().await;
                drop(screen_handler);

                // Restore abilities based on gamemode after respawn
                let mut abilities = self.abilities.lock().await;
                abilities.set_for_gamemode(self.gamemode.load());
                drop(abilities);
                self.send_abilities_update().await;
            }
            1 => {
                // Request stats
                log::debug!("todo");
            }
            _ => {
                self.kick(TextComponent::text("Invalid client status"))
                    .await;
            }
        }
    }

    pub async fn handle_interact(&self, interact: SInteract) {
        if !self.has_client_loaded() {
            return;
        }

        let sneaking = interact.sneaking;
        let entity = &self.living_entity.entity;
        if entity.sneaking.load(std::sync::atomic::Ordering::Relaxed) != sneaking {
            entity.set_sneaking(sneaking).await;
        }
        let Ok(action) = ActionType::try_from(interact.r#type.0) else {
            self.kick(TextComponent::text("Invalid action type")).await;
            return;
        };

        match action {
            ActionType::Attack => {
                let entity_id = interact.entity_id;
                let config = &advanced_config().pvp;
                // TODO: do validation and stuff
                if !config.enabled {
                    return;
                }

                // TODO: set as camera entity when spectator

                let world = &entity.world.read().await;
                let player_victim = world.get_player_by_id(entity_id.0).await;
                if entity_id.0 == self.entity_id() {
                    // This can't be triggered from a non-modded client.
                    self.kick(TextComponent::translate(
                        "multiplayer.disconnect.invalid_entity_attacked",
                        [],
                    ))
                    .await;
                    return;
                }
                if let Some(player_victim) = player_victim {
                    if player_victim.living_entity.health.load() <= 0.0 {
                        // You can trigger this from a non-modded / innocent client,
                        // so we shouldn't kick the player.
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
                            .await;
                        return;
                    }
                    self.attack(player_victim).await;
                } else if let Some(entity_victim) = world.get_entity_by_id(entity_id.0).await {
                    self.attack(entity_victim).await;
                } else {
                    log::error!(
                        "Player id {} interacted with entity id {}, which was not found.",
                        self.entity_id(),
                        entity_id.0
                    );
                    self.kick(TextComponent::translate(
                        "multiplayer.disconnect.invalid_entity_attacked",
                        [],
                    ))
                    .await;
                }
            }
            ActionType::Interact | ActionType::InteractAt => {
                log::debug!("todo");
            }
        }
    }

    #[expect(clippy::too_many_lines)]
    pub async fn handle_player_action(
        self: &Arc<Self>,
        player_action: SPlayerAction,
        server: &Server,
    ) {
        if !self.has_client_loaded() {
            return;
        }
        match Status::try_from(player_action.status.0) {
            Ok(status) => match status {
                Status::StartedDigging => {
                    if !self.can_interact_with_block_at(&player_action.location, 1.0) {
                        log::warn!(
                            "Player {0} tried to interact with block out of reach at {1}",
                            self.gameprofile.name,
                            player_action.location
                        );
                        return;
                    }
                    let location = player_action.location;
                    let entity = &self.living_entity.entity;
                    let world = &entity.world.read().await;
                    let (block, state) = world.get_block_and_block_state(&location).await;

                    let inventory = self.inventory();
                    let held = inventory.held_item();
                    if !server.item_registry.can_mine(held.lock().await.item, self) {
                        self.client
                            .enqueue_packet(&CBlockUpdate::new(
                                location,
                                VarInt(i32::from(state.id)),
                            ))
                            .await;
                        self.update_sequence(player_action.sequence.0);
                        return;
                    }

                    // TODO: do validation
                    // TODO: Config
                    if self.gamemode.load() == GameMode::Creative {
                        // Block break & play sound
                        world
                            .break_block(
                                &location,
                                Some(self.clone()),
                                BlockFlags::NOTIFY_NEIGHBORS | BlockFlags::SKIP_DROPS,
                            )
                            .await;
                        server
                            .block_registry
                            .broken(world, block, self, &location, server, state)
                            .await;
                        self.update_sequence(player_action.sequence.0);
                        return;
                    }
                    self.start_mining_time.store(
                        self.tick_counter.load(std::sync::atomic::Ordering::Relaxed),
                        std::sync::atomic::Ordering::Relaxed,
                    );
                    if !state.is_air() {
                        let speed = block::calc_block_breaking(self, state, block.name).await;
                        // Instant break
                        if speed >= 1.0 {
                            let broken_state = world.get_block_state(&location).await;
                            world
                                .break_block(
                                    &location,
                                    Some(self.clone()),
                                    BlockFlags::NOTIFY_NEIGHBORS,
                                )
                                .await;
                            server
                                .block_registry
                                .broken(world, block, self, &location, server, broken_state)
                                .await;
                        } else {
                            self.mining
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                            *self.mining_pos.lock().await = location;
                            let progress = (speed * 10.0) as i32;
                            world.set_block_breaking(entity, location, progress).await;
                            self.current_block_destroy_stage
                                .store(progress, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    self.update_sequence(player_action.sequence.0);
                }
                Status::CancelledDigging => {
                    if !self.can_interact_with_block_at(&player_action.location, 1.0) {
                        log::warn!(
                            "Player {0} tried to interact with block out of reach at {1}",
                            self.gameprofile.name,
                            player_action.location
                        );
                        return;
                    }
                    self.mining
                        .store(false, std::sync::atomic::Ordering::Relaxed);
                    let entity = &self.living_entity.entity;
                    let world = &entity.world.read().await;
                    world
                        .set_block_breaking(entity, player_action.location, -1)
                        .await;
                    self.update_sequence(player_action.sequence.0);
                }
                Status::FinishedDigging => {
                    // TODO: do validation
                    let location = player_action.location;
                    if !self.can_interact_with_block_at(&location, 1.0) {
                        log::warn!(
                            "Player {0} tried to interact with block out of reach at {1}",
                            self.gameprofile.name,
                            player_action.location
                        );
                        return;
                    }

                    // Block break & play sound
                    let entity = &self.living_entity.entity;
                    let world = &entity.world.read().await;

                    self.mining
                        .store(false, std::sync::atomic::Ordering::Relaxed);
                    world.set_block_breaking(entity, location, -1).await;

                    let (block, state) = world.get_block_and_block_state(&location).await;
                    let drop = self.gamemode.load() != GameMode::Creative
                        && self.can_harvest(state, block.name).await;

                    world
                        .break_block(
                            &location,
                            Some(self.clone()),
                            if drop {
                                BlockFlags::NOTIFY_NEIGHBORS
                            } else {
                                BlockFlags::SKIP_DROPS | BlockFlags::NOTIFY_NEIGHBORS
                            },
                        )
                        .await;

                    server
                        .block_registry
                        .broken(world, block, self, &location, server, state)
                        .await;

                    self.update_sequence(player_action.sequence.0);
                }
                Status::DropItem => {
                    self.drop_held_item(false).await;
                }
                Status::DropItemStack => {
                    self.drop_held_item(true).await;
                }
                Status::ShootArrowOrFinishEating => {
                    log::debug!("todo");
                }
                Status::SwapItem => {
                    self.swap_item().await;
                }
            },
            Err(_) => self.kick(TextComponent::text("Invalid status")).await,
        }
    }

    pub async fn handle_keep_alive(&self, keep_alive: SKeepAlive) {
        if self
            .wait_for_keep_alive
            .load(std::sync::atomic::Ordering::Relaxed)
            && keep_alive.keep_alive_id
                == self
                    .keep_alive_id
                    .load(std::sync::atomic::Ordering::Relaxed)
        {
            self.wait_for_keep_alive
                .store(false, std::sync::atomic::Ordering::Relaxed);
        } else {
            self.kick(TextComponent::text("Timeout")).await;
        }
    }

    pub fn update_sequence(&self, sequence: i32) {
        if sequence < 0 {
            log::error!("Expected packet sequence >= 0");
        }
        self.packet_sequence.store(
            self.packet_sequence
                .load(std::sync::atomic::Ordering::Relaxed)
                .max(sequence),
            std::sync::atomic::Ordering::Relaxed,
        );
    }

    pub async fn handle_player_abilities(&self, player_abilities: SPlayerAbilities) {
        let mut abilities = self.abilities.lock().await;

        // Set the flying ability
        let flying = player_abilities.flags & 0x02 != 0 && abilities.allow_flying;
        if flying {
            self.living_entity.fall_distance.store(0.0);
        }
        abilities.flying = flying;
    }

    pub async fn handle_play_ping_request(&self, request: SPlayPingRequest) {
        self.client
            .enqueue_packet(&CPingResponse::new(request.payload))
            .await;
    }

    pub async fn handle_use_item_on(
        &self,
        use_item_on: SUseItemOn,
        server: &Arc<Server>,
    ) -> Result<(), Box<dyn PumpkinError>> {
        if !self.has_client_loaded() {
            return Ok(());
        }
        self.update_sequence(use_item_on.sequence.0);

        let location = use_item_on.location;
        let mut should_try_decrement = false;

        if !self.can_interact_with_block_at(&location, 1.0) {
            // TODO: maybe log?
            return Err(BlockPlacingError::BlockOutOfReach.into());
        }

        let Ok(face) = BlockDirection::try_from(use_item_on.face.0) else {
            return Err(BlockPlacingError::InvalidBlockFace.into());
        };

        let inventory = self.inventory();
        let held_item = inventory.held_item();

        let entity = &self.living_entity.entity;
        let world = &entity.world.read().await;
        let block = world.get_block(&location).await;

        let sneaking = self
            .living_entity
            .entity
            .sneaking
            .load(std::sync::atomic::Ordering::Relaxed);
        if held_item.lock().await.is_empty() {
            if !sneaking {
                // Using block with empty hand
                server
                    .block_registry
                    .on_use(block, self, &location, server, world)
                    .await;
            }
            return Ok(());
        }
        if !sneaking {
            let action_result = server
                .block_registry
                .use_with_item(block, self, &location, &held_item, server, world)
                .await;
            match action_result {
                BlockActionResult::Continue => {}
                BlockActionResult::Consume => {
                    return Ok(());
                }
            }
            server
                .item_registry
                .use_on_block(
                    held_item.lock().await.item,
                    self,
                    location,
                    face,
                    block,
                    server,
                )
                .await;
            self.update_sequence(use_item_on.sequence.0);
        }

        // Check if the item is a block, because not every item can be placed :D
        if let Some(block) = get_block_by_item(held_item.lock().await.item.id) {
            should_try_decrement = self
                .run_is_block_place(block, server, use_item_on, location, face)
                .await?;
        }

        // Check if the item is a spawn egg
        if let Some(entity) = entity_from_egg(held_item.lock().await.item.id) {
            self.spawn_entity_from_egg(entity, location, face).await;
            should_try_decrement = true;
        }

        if should_try_decrement {
            // TODO: Config
            // Decrease block count
            if self.gamemode.load() != GameMode::Creative {
                held_item.lock().await.decrement(1);
            }
        }

        Ok(())
    }

    pub async fn handle_sign_update(&self, sign_data: SUpdateSign) {
        let world = &self.living_entity.entity.world.read().await;
        let updated_sign = SignBlockEntity::new(
            sign_data.location,
            sign_data.is_front_text,
            [
                sign_data.line_1,
                sign_data.line_2,
                sign_data.line_3,
                sign_data.line_4,
            ],
        );

        world.add_block_entity(Arc::new(updated_sign)).await;
    }

    pub async fn handle_use_item(self: &Arc<Self>, use_item: &SUseItem, server: &Server) {
        if !self.has_client_loaded() {
            return;
        }

        let inventory = self.inventory();
        let binding = inventory.held_item();

        let hit_result = self
            .world()
            .await
            .raycast(
                self.eye_position(),
                self.eye_position().add(
                    &(Vector3::rotation_vector(f64::from(use_item.pitch), f64::from(use_item.yaw))
                        * 4.5),
                ),
                async |pos, world| {
                    let block = world.get_block(pos).await;
                    block != &Block::AIR && block != &Block::WATER && block != &Block::LAVA
                },
            )
            .await;

        let event = if let Some((hit_pos, _hit_dir)) = hit_result {
            PlayerInteractEvent::new(
                self,
                InteractAction::RightClickBlock,
                &binding,
                self.world().await.get_block(&hit_pos).await,
                Some(hit_pos),
            )
        } else {
            PlayerInteractEvent::new(
                self,
                InteractAction::RightClickAir,
                &binding,
                &Block::AIR,
                None,
            )
        };

        send_cancellable! {{
            event;
            'after: {
                let held = binding.lock().await;
                let item = held.item;
                drop(held);
                server.item_registry.on_use(item, self).await;
                self.update_sequence(use_item.sequence.0);
            }
        }}
    }

    pub async fn handle_set_held_item(&self, held: SSetHeldItem) {
        let slot = held.slot;
        if !(0..=8).contains(&slot) {
            self.kick(TextComponent::text("Invalid held slot")).await;
            return;
        }
        let inv = self.inventory();
        inv.set_selected_slot(slot as u8);
        let stack = *inv.held_item().lock().await;
        let equipment = &[(EquipmentSlot::MAIN_HAND, stack)];
        self.living_entity.send_equipment_changes(equipment).await;
    }

    pub async fn handle_set_creative_slot(
        &self,
        packet: SSetCreativeSlot,
    ) -> Result<(), InventoryError> {
        if self.gamemode.load() != GameMode::Creative {
            return Err(InventoryError::PermissionError);
        }
        let is_negative = packet.slot < 0;
        let valid_slot = packet.slot >= 1 && packet.slot as usize <= 45;
        let item_stack = packet.clicked_item.to_stack();
        let is_legal =
            item_stack.is_empty() || item_stack.item_count <= item_stack.get_max_stack_size();

        if valid_slot && is_legal {
            let mut player_screen_handler = self.player_screen_handler.lock().await;
            player_screen_handler
                .get_slot(packet.slot as usize)
                .await
                .set_stack(item_stack)
                .await;
            player_screen_handler.set_received_stack(packet.slot as usize, item_stack);
            player_screen_handler.send_content_updates().await;
        } else if is_negative && is_legal {
            // Item drop
            self.drop_item(item_stack).await;
        }
        Ok(())
    }

    pub async fn handle_chunk_batch(&self, packet: SChunkBatch) {
        let mut chunk_manager = self.chunk_manager.lock().await;
        chunk_manager.handle_acknowledge(packet.chunks_per_tick);
        log::trace!(
            "Client {} requested {} chunks per tick",
            self.client.id,
            packet.chunks_per_tick
        );
    }

    pub async fn handle_close_container(&self, _server: &Server, _packet: SCloseContainer) {
        self.on_handled_screen_closed().await;
    }

    pub async fn handle_command_suggestion(
        self: &Arc<Self>,
        packet: SCommandSuggestion,
        server: &Arc<Server>,
    ) {
        let mut src = CommandSender::Player(self.clone());
        let Some(cmd) = &packet.command.get(1..) else {
            return;
        };

        let Some((last_word_start, _)) = cmd.char_indices().rfind(|(_, c)| c.is_whitespace())
        else {
            return;
        };

        let dispatcher = server.command_dispatcher.read().await;
        let suggestions = dispatcher.find_suggestions(&mut src, server, cmd).await;

        let response = CCommandSuggestions::new(
            packet.id,
            (last_word_start + 2).try_into().unwrap(),
            (cmd.len() - last_word_start - 1).try_into().unwrap(),
            suggestions.into(),
        );

        self.client.enqueue_packet(&response).await;
    }

    pub fn handle_cookie_response(&self, packet: &SPCookieResponse) {
        // TODO: allow plugins to access this
        log::debug!(
            "Received cookie_response[play]: key: \"{}\", payload_length: \"{:?}\"",
            packet.key,
            packet.payload.as_ref().map(|p| p.len())
        );
    }

    async fn spawn_entity_from_egg(
        &self,
        entity_type: EntityType,
        location: BlockPos,
        face: BlockDirection,
    ) {
        let world_pos = BlockPos(location.0 + face.to_offset());
        // Align the position like Vanilla does
        let pos = Vector3::new(
            f64::from(world_pos.0.x) + 0.5,
            f64::from(world_pos.0.y),
            f64::from(world_pos.0.z) + 0.5,
        );
        // Create rotation like Vanilla
        let yaw = wrap_degrees(rand::random::<f32>() * 360.0) % 360.0;

        let world = self.world().await;
        // Create a new mob and UUID based on the spawn egg id
        let mob = from_type(entity_type, pos, &world, Uuid::new_v4());

        // Set the rotation
        mob.get_entity().set_rotation(yaw, 0.0);

        // Broadcast the new mob to all players
        world.spawn_entity(mob).await;

        // TODO: send/configure additional commands/data based on the type of entity (horse, slime, etc)
    }

    const WORLD_LOWEST_Y: i8 = -64;
    const WORLD_MAX_Y: u16 = 320;

    #[allow(clippy::too_many_lines)]
    async fn run_is_block_place(
        &self,
        block: &'static Block,
        server: &Server,
        use_item_on: SUseItemOn,
        location: BlockPos,
        face: BlockDirection,
    ) -> Result<bool, Box<dyn PumpkinError>> {
        let entity = &self.living_entity.entity;
        let world = &entity.world.read().await;

        // Check if the block is under the world
        if location.0.y + face.to_offset().y < i32::from(Self::WORLD_LOWEST_Y) {
            return Err(BlockPlacingError::BlockOutOfWorld.into());
        }

        // Check the world's max build height
        if location.0.y + face.to_offset().y >= i32::from(Self::WORLD_MAX_Y) {
            self.send_system_message_raw(
                &TextComponent::translate(
                    "build.tooHigh",
                    vec![TextComponent::text((Self::WORLD_MAX_Y - 1).to_string())],
                )
                .color_named(NamedColor::Red),
                true,
            )
            .await;
            return Err(BlockPlacingError::BlockOutOfWorld.into());
        }

        match self.gamemode.load() {
            GameMode::Spectator | GameMode::Adventure => {
                return Err(BlockPlacingError::InvalidGamemode.into());
            }
            _ => {}
        }

        let clicked_block_pos = BlockPos(location.0);
        let (clicked_block, clicked_block_state) =
            world.get_block_and_block_state(&clicked_block_pos).await;

        let replace_clicked_block = if clicked_block == block {
            world
                .block_registry
                .can_update_at(
                    world,
                    clicked_block,
                    clicked_block_state.id,
                    &clicked_block_pos,
                    face,
                    &use_item_on,
                    self,
                )
                .await
                .then_some(BlockIsReplacing::Itself(clicked_block_state.id))
        } else if clicked_block_state.replaceable() {
            if clicked_block == &Block::WATER {
                let water_props =
                    WaterLikeProperties::from_state_id(clicked_block_state.id, clicked_block);
                Some(BlockIsReplacing::Water(water_props.level))
            } else {
                Some(BlockIsReplacing::Other)
            }
        } else {
            None
        };

        let (final_block_pos, final_face, replacing) =
            if let Some(replacing) = replace_clicked_block {
                (clicked_block_pos, face, replacing)
            } else {
                let block_pos = BlockPos(location.0 + face.to_offset());
                let (previous_block, previous_block_state) =
                    world.get_block_and_block_state(&block_pos).await;

                let replace_previous_block = if previous_block == block {
                    world
                        .block_registry
                        .can_update_at(
                            world,
                            previous_block,
                            previous_block_state.id,
                            &block_pos,
                            face.opposite(),
                            &use_item_on,
                            self,
                        )
                        .await
                        .then_some(BlockIsReplacing::Itself(previous_block_state.id))
                } else {
                    previous_block_state.replaceable().then(|| {
                        if previous_block == &Block::WATER {
                            let water_props = WaterLikeProperties::from_state_id(
                                previous_block_state.id,
                                previous_block,
                            );
                            BlockIsReplacing::Water(water_props.level)
                        } else {
                            BlockIsReplacing::None
                        }
                    })
                };

                match replace_previous_block {
                    Some(replacing) => (block_pos, face.opposite(), replacing),
                    None => {
                        // Don't place and don't decrement if the previous block is not replaceable
                        return Ok(false);
                    }
                }
            };

        if !server
            .block_registry
            .can_place_at(
                Some(server),
                Some(world),
                world.as_ref(),
                Some(self),
                block,
                &final_block_pos,
                final_face,
                Some(&use_item_on),
            )
            .await
        {
            return Ok(false);
        }

        let new_state = server
            .block_registry
            .on_place(
                server,
                world,
                self,
                block,
                &final_block_pos,
                final_face,
                replacing,
                &use_item_on,
            )
            .await;

        // Check if there is a player in the way of the block being placed
        let shapes = get_state_by_state_id(new_state)
            .unwrap()
            .get_block_collision_shapes();
        for player in world.get_nearby_players(location.0.to_f64(), 3.0).await {
            let player_box = player.1.living_entity.entity.bounding_box.load();
            for shape in &shapes {
                if shape.at_pos(final_block_pos).intersects(&player_box) {
                    return Ok(false);
                }
            }
        }

        let _replaced_id = world
            .set_block_state(&final_block_pos, new_state, BlockFlags::NOTIFY_ALL)
            .await;

        server
            .block_registry
            .player_placed(world, block, new_state, &final_block_pos, face, self)
            .await;

        // The block was placed successfully, so decrement their inventory
        Ok(true)
    }

    /// Checks if the block placed was a sign, then opens a dialog.
    pub async fn send_sign_packet(&self, block_position: BlockPos) {
        self.client
            .enqueue_packet(&COpenSignEditor::new(block_position, true))
            .await;
    }
}
