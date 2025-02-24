use pumpkin_world::block::registry::State;
use std::{
    num::NonZeroU8,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicU32, Ordering},
    },
    time::{Duration, Instant},
};

use async_trait::async_trait;
use crossbeam::atomic::AtomicCell;
use pumpkin_config::{ADVANCED_CONFIG, BASIC_CONFIG};
use pumpkin_data::{
    damage::DamageType,
    entity::{EffectType, EntityType},
    item::Operation,
    particle::Particle,
    sound::{Sound, SoundCategory},
};
use pumpkin_inventory::player::PlayerInventory;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::{
    RawPacket, ServerPacket,
    bytebuf::packet::Packet,
    client::play::{
        CAcknowledgeBlockChange, CActionBar, CCombatDeath, CDisguisedChatMessage, CEntityStatus,
        CGameEvent, CHurtAnimation, CKeepAlive, CParticle, CPlayDisconnect, CPlayerAbilities,
        CPlayerInfoUpdate, CPlayerPosition, CRespawn, CSetExperience, CSetHealth, CSubtitle,
        CSystemChatMessage, CTitleText, CUnloadChunk, CUpdateMobEffect, GameEvent, MetaDataType,
        PlayerAction,
    },
    server::play::{
        SChatCommand, SChatMessage, SClientCommand, SClientInformationPlay, SClientTickEnd,
        SCommandSuggestion, SConfirmTeleport, SInteract, SPickItemFromBlock, SPlayerAbilities,
        SPlayerAction, SPlayerCommand, SPlayerInput, SPlayerPosition, SPlayerPositionRotation,
        SPlayerRotation, SSetCreativeSlot, SSetHeldItem, SSetPlayerGround, SSwingArm, SUpdateSign,
        SUseItem, SUseItemOn,
    },
};
use pumpkin_protocol::{
    client::play::CSoundEffect,
    server::play::{
        SCloseContainer, SCookieResponse as SPCookieResponse, SPlayPingRequest, SPlayerLoaded,
    },
};
use pumpkin_protocol::{client::play::CUpdateTime, codec::var_int::VarInt};
use pumpkin_protocol::{
    client::play::Metadata,
    server::play::{SClickContainer, SKeepAlive},
};
use pumpkin_util::{
    GameMode,
    math::{
        boundingbox::{BoundingBox, EntityDimensions},
        experience,
        position::BlockPos,
        vector2::Vector2,
        vector3::Vector3,
    },
    permission::PermissionLvl,
    text::TextComponent,
};
use pumpkin_world::{cylindrical_chunk_iterator::Cylindrical, item::ItemStack};
use tokio::sync::{Mutex, Notify, RwLock};

use super::{
    Entity, EntityBase, EntityId, NBTStorage,
    combat::{self, AttackType, player_attack_sound},
    effect::Effect,
    hunger::HungerManager,
    item::ItemEntity,
};
use crate::{
    block,
    command::{client_suggestions, dispatcher::CommandDispatcher},
    data::op_data::OPERATOR_CONFIG,
    net::{Client, PlayerConfig},
    server::Server,
    world::World,
};
use crate::{error::PumpkinError, net::GameProfile};

use super::living::LivingEntity;

/// Represents a Minecraft player entity.
///
/// A `Player` is a special type of entity that represents a human player connected to the server.
pub struct Player {
    /// The underlying living entity object that represents the player.
    pub living_entity: LivingEntity,
    /// The player's game profile information, including their username and UUID.
    pub gameprofile: GameProfile,
    /// The client connection associated with the player.
    pub client: Arc<Client>,
    /// Players Inventory
    pub inventory: Mutex<PlayerInventory>,
    /// The player's configuration settings. Changes when the Player changes their settings.
    pub config: Mutex<PlayerConfig>,
    /// The player's current gamemode (e.g., Survival, Creative, Adventure).
    pub gamemode: AtomicCell<GameMode>,
    /// The Hunger Manager manages Players hunger level
    pub hunger_manager: HungerManager,
    /// The ID of the currently open container (if any).
    pub open_container: AtomicCell<Option<u64>>,
    /// The item currently being held by the player.
    pub carried_item: AtomicCell<Option<ItemStack>>,
    /// send `send_abilities_update` when changed
    /// The player's abilities and special powers.
    ///
    /// This field represents the various abilities that the player possesses, such as flight, invulnerability, and other special effects.
    ///
    /// **Note:** When the `abilities` field is updated, the server should send a `send_abilities_update` packet to the client to notify them of the changes.
    pub abilities: Mutex<Abilities>,
    /// The current stage of the block the player is breaking.
    pub current_block_destroy_stage: AtomicI32,
    /// Indicates if the player is currently mining a block.
    pub mining: AtomicBool,
    pub start_mining_time: AtomicI32,
    pub tick_counter: AtomicI32,
    pub packet_sequence: AtomicI32,
    pub mining_pos: Mutex<BlockPos>,
    /// A counter for teleport IDs used to track pending teleports.
    pub teleport_id_count: AtomicI32,
    /// The pending teleport information, including the teleport ID and target location.
    pub awaiting_teleport: Mutex<Option<(VarInt, Vector3<f64>)>>,
    /// The coordinates of the chunk section the player is currently watching.
    pub watched_section: AtomicCell<Cylindrical>,
    /// Did we send a keep alive Packet and wait for the response?
    pub wait_for_keep_alive: AtomicBool,
    /// Whats the keep alive packet payload we send, The client should respond with the same id
    pub keep_alive_id: AtomicI64,
    /// Last time we send a keep alive
    pub last_keep_alive_time: AtomicCell<Instant>,
    /// Amount of ticks since last attack
    pub last_attacked_ticks: AtomicU32,
    /// The players op permission level
    pub permission_lvl: AtomicCell<PermissionLvl>,
    /// Tell tasks to stop if we are closing
    cancel_tasks: Notify,
    /// whether the client has reported it has loaded
    pub client_loaded: AtomicBool,
    /// timeout (in ticks) client has to report it has finished loading.
    pub client_loaded_timeout: AtomicU32,
    /// The player's experience level
    pub experience_level: AtomicI32,
    /// The player's experience progress (0.0 to 1.0)
    pub experience_progress: AtomicCell<f32>,
    /// The player's total experience points
    pub experience_points: AtomicI32,
}

impl Player {
    pub async fn new(
        client: Arc<Client>,
        world: Arc<World>,
        entity_id: EntityId,
        gamemode: GameMode,
    ) -> Self {
        let gameprofile = client.gameprofile.lock().await.clone().map_or_else(
            || {
                log::error!("Client {} has no game profile!", client.id);
                GameProfile {
                    id: uuid::Uuid::new_v4(),
                    name: String::new(),
                    properties: vec![],
                    profile_actions: None,
                }
            },
            |profile| profile,
        );
        let player_uuid = gameprofile.id;

        let gameprofile_clone = gameprofile.clone();
        let config = client.config.lock().await.clone().unwrap_or_default();
        let bounding_box_size = EntityDimensions {
            width: EntityType::PLAYER.dimension[0],
            height: EntityType::PLAYER.dimension[1],
        };

        Self {
            living_entity: LivingEntity::new(Entity::new(
                entity_id,
                player_uuid,
                world,
                Vector3::new(0.0, 0.0, 0.0),
                EntityType::PLAYER,
                EntityType::PLAYER.eye_height,
                AtomicCell::new(BoundingBox::new_default(&bounding_box_size)),
                AtomicCell::new(bounding_box_size),
                matches!(gamemode, GameMode::Creative | GameMode::Spectator),
            )),
            config: Mutex::new(config),
            gameprofile,
            client,
            awaiting_teleport: Mutex::new(None),
            // TODO: Load this from previous instance
            hunger_manager: HungerManager::default(),
            current_block_destroy_stage: AtomicI32::new(-1),
            open_container: AtomicCell::new(None),
            tick_counter: AtomicI32::new(0),
            packet_sequence: AtomicI32::new(-1),
            start_mining_time: AtomicI32::new(0),
            carried_item: AtomicCell::new(None),
            teleport_id_count: AtomicI32::new(0),
            mining: AtomicBool::new(false),
            mining_pos: Mutex::new(BlockPos(Vector3::new(0, 0, 0))),
            abilities: Mutex::new(Abilities::default()),
            gamemode: AtomicCell::new(gamemode),
            // We want this to be an impossible watched section so that `player_chunker::update_position`
            // will mark chunks as watched for a new join rather than a respawn
            // (We left shift by one so we can search around that chunk)
            watched_section: AtomicCell::new(Cylindrical::new(
                Vector2::new(i32::MAX >> 1, i32::MAX >> 1),
                unsafe { NonZeroU8::new_unchecked(1) },
            )),
            wait_for_keep_alive: AtomicBool::new(false),
            keep_alive_id: AtomicI64::new(0),
            last_keep_alive_time: AtomicCell::new(std::time::Instant::now()),
            last_attacked_ticks: AtomicU32::new(0),
            cancel_tasks: Notify::new(),
            client_loaded: AtomicBool::new(false),
            client_loaded_timeout: AtomicU32::new(60),
            // Minecraft has no why to change the default permission level of new players.
            // Minecrafts default permission level is 0
            permission_lvl: OPERATOR_CONFIG
                .read()
                .await
                .ops
                .iter()
                .find(|op| op.uuid == gameprofile_clone.id)
                .map_or(
                    AtomicCell::new(ADVANCED_CONFIG.commands.default_op_level),
                    |op| AtomicCell::new(op.level),
                ),
            inventory: Mutex::new(PlayerInventory::new()),
            experience_level: AtomicI32::new(0),
            experience_progress: AtomicCell::new(0.0),
            experience_points: AtomicI32::new(0),
        }
    }

    pub fn inventory(&self) -> &Mutex<PlayerInventory> {
        &self.inventory
    }

    /// Removes the Player out of the current World
    #[allow(unused_variables)]
    pub async fn remove(self: Arc<Self>) {
        let world = self.world().await;
        self.cancel_tasks.notify_waiters();

        world.remove_player(self.clone(), true).await;

        let cylindrical = self.watched_section.load();

        // Radial chunks are all of the chunks the player is theoretically viewing
        // Giving enough time, all of these chunks will be in memory
        let radial_chunks = cylindrical.all_chunks_within();

        log::debug!(
            "Removing player {} ({}), unwatching {} chunks",
            self.gameprofile.name,
            self.client.id,
            radial_chunks.len()
        );

        let level = &world.level;

        // Decrement value of watched chunks
        let chunks_to_clean = level.mark_chunks_as_not_watched(&radial_chunks);

        // Remove chunks with no watchers from the cache
        level.clean_chunks(&chunks_to_clean).await;
        // Remove left over entries from all possiblily loaded chunks
        level.clean_memory(&radial_chunks);

        log::debug!(
            "Removed player id {} ({}) ({} chunks remain cached)",
            self.gameprofile.name,
            self.client.id,
            level.loaded_chunk_count()
        );

        //self.world().level.list_cached();
    }

    pub async fn attack(&self, victim: Arc<dyn EntityBase>) {
        let world = self.world().await;
        let victim_entity = victim.get_entity();
        let victim_living_entity = victim.get_living_entity();
        let attacker_entity = &self.living_entity.entity;
        let config = &ADVANCED_CONFIG.pvp;

        let inventory = self.inventory().lock().await;
        let item_slot = inventory.held_item();

        let base_damage = 1.0;
        let base_attack_speed = 4.0;

        let mut damage_multiplier = 1.0;
        let mut add_damage = 0.0;
        let mut add_speed = 0.0;

        // get attack damage
        if let Some(item_stack) = item_slot {
            // TODO: this should be cached in memory
            if let Some(modifiers) = &item_stack.item.components.attribute_modifiers {
                for item_mod in modifiers.modifiers {
                    if item_mod.operation == Operation::AddValue {
                        if item_mod.id == "minecraft:base_attack_damage" {
                            add_damage = item_mod.amount;
                        }
                        if item_mod.id == "minecraft:base_attack_speed" {
                            add_speed = item_mod.amount;
                        }
                    }
                }
            }
        }
        drop(inventory);

        let attack_speed = base_attack_speed + add_speed;

        let attack_cooldown_progress = self.get_attack_cooldown_progress(0.5, attack_speed);
        self.last_attacked_ticks
            .store(0, std::sync::atomic::Ordering::Relaxed);

        // only reduce attack damage if in cooldown
        // TODO: Enchantments are reduced same way just without the square
        if attack_cooldown_progress < 1.0 {
            damage_multiplier = 0.2 + attack_cooldown_progress.powi(2) * 0.8;
        }
        // modify added damage based on multiplier
        let mut damage = base_damage + add_damage * damage_multiplier;

        let pos = victim_entity.pos.load();

        if let Some(living) = victim_living_entity {
            if !living.check_damage(damage as f32) {
                world
                    .play_sound(
                        Sound::EntityPlayerAttackNodamage,
                        SoundCategory::Players,
                        &pos,
                    )
                    .await;
                return;
            }
        }

        world
            .play_sound(Sound::EntityPlayerHurt, SoundCategory::Players, &pos)
            .await;

        let attack_type = AttackType::new(self, attack_cooldown_progress as f32).await;

        player_attack_sound(&pos, &world, attack_type).await;

        if matches!(attack_type, AttackType::Critical) {
            damage *= 1.5;
        }

        if let Some(living) = victim_living_entity {
            living
                .damage(damage as f32, DamageType::PLAYER_ATTACK)
                .await;
        }

        let mut knockback_strength = 1.0;
        match attack_type {
            AttackType::Knockback => knockback_strength += 1.0,
            AttackType::Sweeping => {
                combat::spawn_sweep_particle(attacker_entity, &world, &pos).await;
            }
            _ => {}
        };

        if config.knockback {
            combat::handle_knockback(attacker_entity, &world, victim_entity, knockback_strength)
                .await;
        }

        if config.hurt_animation {
            let entity_id = VarInt(victim_entity.entity_id);
            world
                .broadcast_packet_all(&CHurtAnimation::new(entity_id, attacker_entity.yaw.load()))
                .await;
        }

        if config.swing {}
    }

    pub async fn show_title(&self, text: &TextComponent, mode: &TitleMode) {
        match mode {
            TitleMode::Title => self.client.send_packet(&CTitleText::new(text)).await,
            TitleMode::SubTitle => self.client.send_packet(&CSubtitle::new(text)).await,
            TitleMode::ActionBar => self.client.send_packet(&CActionBar::new(text)).await,
        }
    }

    pub async fn spawn_particle(
        &self,
        position: Vector3<f64>,
        offset: Vector3<f32>,
        max_speed: f32,
        particle_count: i32,
        pariticle: Particle,
    ) {
        self.client
            .send_packet(&CParticle::new(
                false,
                false,
                position,
                offset,
                max_speed,
                particle_count,
                VarInt(pariticle as i32),
                &[],
            ))
            .await;
    }

    pub async fn play_sound(
        &self,
        sound_id: u16,
        category: SoundCategory,
        position: &Vector3<f64>,
        volume: f32,
        pitch: f32,
        seed: f64,
    ) {
        self.client
            .send_packet(&CSoundEffect::new(
                VarInt(i32::from(sound_id)),
                None,
                category,
                position,
                volume,
                pitch,
                seed,
            ))
            .await;
    }

    pub async fn await_cancel(&self) {
        self.cancel_tasks.notified().await;
    }

    pub async fn tick(&self) {
        if self
            .client
            .closed
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            return;
        }
        if self.packet_sequence.load(Ordering::Relaxed) > -1 {
            self.client
                .send_packet(&CAcknowledgeBlockChange::new(
                    self.packet_sequence.swap(-1, Ordering::Relaxed).into(),
                ))
                .await;
        }

        self.tick_counter.fetch_add(1, Ordering::Relaxed);

        if self.mining.load(Ordering::Relaxed) {
            let pos = self.mining_pos.lock().await;
            let world = self.world().await;
            let block = world.get_block(&pos).await.unwrap();
            let state = world.get_block_state(&pos).await.unwrap();
            // Is block broken ?
            if state.air {
                world
                    .set_block_breaking(&self.living_entity.entity, *pos, -1)
                    .await;
                self.current_block_destroy_stage
                    .store(-1, Ordering::Relaxed);
                self.mining.store(false, Ordering::Relaxed);
            } else {
                self.continue_mining(
                    *pos,
                    &world,
                    state,
                    &block.name,
                    self.start_mining_time.load(Ordering::Relaxed),
                )
                .await;
            }
        }

        self.last_attacked_ticks
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        self.living_entity.tick();
        self.hunger_manager.tick(self).await;

        // timeout/keep alive handling
        self.tick_client_load_timeout();

        let now = Instant::now();
        if now.duration_since(self.last_keep_alive_time.load()) >= Duration::from_secs(15) {
            // We never got a response from our last keep alive we send
            if self
                .wait_for_keep_alive
                .load(std::sync::atomic::Ordering::Relaxed)
            {
                self.kick(TextComponent::translate("disconnect.timeout", []))
                    .await;
                return;
            }
            self.wait_for_keep_alive
                .store(true, std::sync::atomic::Ordering::Relaxed);
            self.last_keep_alive_time.store(now);
            let id = now.elapsed().as_millis() as i64;
            self.keep_alive_id
                .store(id, std::sync::atomic::Ordering::Relaxed);
            self.client.send_packet(&CKeepAlive::new(id)).await;
        }
    }

    async fn continue_mining(
        &self,
        location: BlockPos,
        world: &World,
        state: &State,
        block_name: &str,
        starting_time: i32,
    ) {
        let time = self.tick_counter.load(Ordering::Relaxed) - starting_time;
        let speed = block::calc_block_breaking(self, state, block_name).await * (time + 1) as f32;
        let progress = (speed * 10.0) as i32;
        if progress != self.current_block_destroy_stage.load(Ordering::Relaxed) {
            world
                .set_block_breaking(&self.living_entity.entity, location, progress)
                .await;
            self.current_block_destroy_stage
                .store(progress, Ordering::Relaxed);
        }
    }

    pub async fn jump(&self) {
        if self.living_entity.entity.sprinting.load(Ordering::Relaxed) {
            self.add_exhaustion(0.2).await;
        } else {
            self.add_exhaustion(0.05).await;
        }
    }

    #[expect(clippy::cast_precision_loss)]
    pub async fn progress_motion(&self, delta_pos: Vector3<f64>) {
        // TODO: Swming, Glding...
        if self.living_entity.entity.on_ground.load(Ordering::Relaxed) {
            let delta = (delta_pos.horizontal_length() * 100.0).round() as i32;
            if delta > 0 {
                if self.living_entity.entity.sprinting.load(Ordering::Relaxed) {
                    self.add_exhaustion(0.1 * delta as f32 * 0.01).await;
                } else {
                    self.add_exhaustion(0.0 * delta as f32 * 0.01).await;
                }
            }
        }
    }

    pub fn has_client_loaded(&self) -> bool {
        self.client_loaded.load(Ordering::Relaxed)
            || self.client_loaded_timeout.load(Ordering::Relaxed) == 0
    }

    pub fn set_client_loaded(&self, loaded: bool) {
        if !loaded {
            self.client_loaded_timeout.store(60, Ordering::Relaxed);
        }
        self.client_loaded.store(loaded, Ordering::Relaxed);
    }

    pub fn get_attack_cooldown_progress(&self, base_time: f64, attack_speed: f64) -> f64 {
        let x = f64::from(
            self.last_attacked_ticks
                .load(std::sync::atomic::Ordering::Acquire),
        ) + base_time;

        let progress_per_tick = f64::from(BASIC_CONFIG.tps) / attack_speed;
        let progress = x / progress_per_tick;
        progress.clamp(0.0, 1.0)
    }

    pub const fn entity_id(&self) -> EntityId {
        self.living_entity.entity.entity_id
    }

    pub async fn world(&self) -> Arc<World> {
        self.living_entity.entity.world.read().await.clone()
    }

    pub fn position(&self) -> Vector3<f64> {
        self.living_entity.entity.pos.load()
    }

    /// Updates the current abilities the Player has
    pub async fn send_abilities_update(&self) {
        let mut b = 0i8;
        let abilities = &self.abilities.lock().await;

        if abilities.invulnerable {
            b |= 1;
        }
        if abilities.flying {
            b |= 2;
        }
        if abilities.allow_flying {
            b |= 4;
        }
        if abilities.creative {
            b |= 8;
        }
        self.client
            .send_packet(&CPlayerAbilities::new(
                b,
                abilities.fly_speed,
                abilities.walk_speed,
            ))
            .await;
    }

    /// syncs the players permission level with the client
    pub async fn send_permission_lvl_update(&self) {
        self.client
            .send_packet(&CEntityStatus::new(
                self.entity_id(),
                24 + self.permission_lvl.load() as i8,
            ))
            .await;
    }

    /// sets the players permission level and syncs it with the client
    pub async fn set_permission_lvl(
        self: &Arc<Self>,
        lvl: PermissionLvl,
        command_dispatcher: &RwLock<CommandDispatcher>,
    ) {
        self.permission_lvl.store(lvl);
        self.send_permission_lvl_update().await;
        client_suggestions::send_c_commands_packet(self, command_dispatcher).await;
    }

    /// Sends the world time to just the player.
    pub async fn send_time(&self, world: &World) {
        let l_world = world.level_time.lock().await;
        self.client
            .send_packet(&CUpdateTime::new(
                l_world.world_age,
                l_world.time_of_day,
                true,
            ))
            .await;
    }

    /// Sends the mobs to just the player.
    // TODO: This should be optimized for larger servers based on current player chunk
    pub async fn send_mobs(&self, world: &World) {
        let entities = world.entities.read().await.clone();
        for (_, entity) in entities {
            self.client
                .send_packet(&entity.get_entity().create_spawn_packet())
                .await;
        }
    }

    async fn unload_watched_chunks(&self, world: &World) {
        let radial_chunks = self.watched_section.load().all_chunks_within();
        let level = &world.level;
        let chunks_to_clean = level.mark_chunks_as_not_watched(&radial_chunks);
        level.clean_chunks(&chunks_to_clean).await;
        let client = self.client.clone();
        tokio::spawn(async move {
            for chunk in chunks_to_clean {
                if client.closed.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                client
                    .send_packet(&CUnloadChunk::new(chunk.x, chunk.z))
                    .await;
            }
        });
        self.watched_section.store(Cylindrical::new(
            Vector2::new(i32::MAX >> 1, i32::MAX >> 1),
            unsafe { NonZeroU8::new_unchecked(1) },
        ));
    }

    /// Teleports the player to a different world or dimension with an optional position, yaw, and pitch.
    pub async fn teleport_world(
        self: Arc<Self>,
        new_world: Arc<World>,
        position: Option<Vector3<f64>>,
        yaw: Option<f32>,
        pitch: Option<f32>,
    ) {
        self.set_client_loaded(false);
        let current_world = self.living_entity.entity.world.read().await.clone();
        let uuid = self.gameprofile.id;
        current_world.remove_player(self.clone(), false).await;
        *self.living_entity.entity.world.write().await = new_world.clone();
        new_world.players.write().await.insert(uuid, self.clone());
        self.unload_watched_chunks(&current_world).await;
        let last_pos = self.living_entity.last_pos.load();
        let death_dimension = self.world().await.dimension_type.name();
        let death_location = BlockPos(Vector3::new(
            last_pos.x.round() as i32,
            last_pos.y.round() as i32,
            last_pos.z.round() as i32,
        ));
        self.client
            .send_packet(&CRespawn::new(
                (new_world.dimension_type as u8).into(),
                new_world.dimension_type.name(),
                0, // seed
                self.gamemode.load() as u8,
                self.gamemode.load() as i8,
                false,
                false,
                Some((death_dimension, death_location)),
                0.into(),
                0.into(),
                1,
            ))
            .await;
        self.send_abilities_update().await;
        self.send_permission_lvl_update().await;
        let info = &new_world.level.level_info;
        let position = if let Some(pos) = position {
            pos
        } else {
            Vector3::new(
                f64::from(info.spawn_x),
                f64::from(
                    new_world
                        .get_top_block(Vector2::new(
                            f64::from(info.spawn_x) as i32,
                            f64::from(info.spawn_x) as i32,
                        ))
                        .await
                        + 1,
                ),
                f64::from(info.spawn_z),
            )
        };
        let yaw = yaw.unwrap_or(info.spawn_angle);
        let pitch = pitch.unwrap_or(10.0);
        self.request_teleport(position, yaw, pitch).await;
        self.living_entity.last_pos.store(position);

        new_world.send_world_info(&self, position, yaw, pitch).await;
    }

    /// Yaw and Pitch in degrees
    /// Rarly used, For example when waking up player from bed or first time spawn. Otherwise entity teleport is used
    /// Player should respond with the `SConfirmTeleport` packet
    pub async fn request_teleport(&self, position: Vector3<f64>, yaw: f32, pitch: f32) {
        // this is the ultra special magic code used to create the teleport id
        // This returns the old value
        // This operation wraps around on overflow.
        let i = self
            .teleport_id_count
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let teleport_id = i + 1;
        self.living_entity.set_pos(position);
        let entity = &self.living_entity.entity;
        entity.set_rotation(yaw, pitch);
        *self.awaiting_teleport.lock().await = Some((teleport_id.into(), position));
        self.client
            .send_packet(&CPlayerPosition::new(
                teleport_id.into(),
                position,
                Vector3::new(0.0, 0.0, 0.0),
                yaw,
                pitch,
                // TODO
                &[],
            ))
            .await;
    }

    pub fn block_interaction_range(&self) -> f64 {
        if self.gamemode.load() == GameMode::Creative {
            5.0
        } else {
            4.5
        }
    }

    pub fn can_interact_with_block_at(&self, pos: &BlockPos, additional_range: f64) -> bool {
        let d = self.block_interaction_range() + additional_range;
        let box_pos = BoundingBox::from_block(pos);
        let entity_pos = self.living_entity.entity.pos.load();
        let standing_eye_height = self.living_entity.entity.standing_eye_height;
        box_pos.squared_magnitude(Vector3 {
            x: entity_pos.x,
            y: entity_pos.y + f64::from(standing_eye_height),
            z: entity_pos.z,
        }) < d * d
    }

    /// Kicks the Client with a reason depending on the connection state
    pub async fn kick(&self, reason: TextComponent) {
        if self
            .client
            .closed
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            log::debug!(
                "Tried to kick id {} but connection is closed!",
                self.client.id
            );
            return;
        }

        let _ = self
            .client
            .try_send_packet(&CPlayDisconnect::new(reason.clone()))
            .await;

        log::info!(
            "Kicked Player {} ({}) for {}",
            self.gameprofile.name,
            self.client.id,
            reason.to_pretty_console()
        );

        self.client.close().await;
    }

    pub fn can_food_heal(&self) -> bool {
        let health = self.living_entity.health.load();
        let max_health = 20.0; // TODO
        health > 0.0 && health < max_health
    }

    pub async fn add_exhaustion(&self, exhaustion: f32) {
        let abilities = self.abilities.lock().await;
        if abilities.invulnerable {
            return;
        }
        self.hunger_manager.add_exhausten(exhaustion);
    }

    pub async fn heal(&self, additional_health: f32) {
        self.living_entity.heal(additional_health).await;
        self.send_health().await;
    }

    pub async fn send_health(&self) {
        self.client
            .send_packet(&CSetHealth::new(
                self.living_entity.health.load(),
                self.hunger_manager.level.load().into(),
                self.hunger_manager.saturation.load(),
            ))
            .await;
    }

    pub async fn set_health(&self, health: f32) {
        self.living_entity.set_health(health).await;
        self.send_health().await;
    }

    pub fn tick_client_load_timeout(&self) {
        if !self.client_loaded.load(Ordering::Relaxed) {
            let timeout = self.client_loaded_timeout.load(Ordering::Relaxed);
            self.client_loaded_timeout
                .store(timeout.saturating_sub(1), Ordering::Relaxed);
        }
    }

    pub async fn kill(&self) {
        self.living_entity.kill().await;
        self.set_client_loaded(false);
        self.client
            .send_packet(&CCombatDeath::new(
                self.entity_id().into(),
                &TextComponent::text("noob"),
            ))
            .await;
    }

    pub async fn set_gamemode(&self, gamemode: GameMode) {
        // We could send the same gamemode without problems. But why waste bandwidth ?
        assert_ne!(
            self.gamemode.load(),
            gamemode,
            "Setting the same gamemode as already is"
        );
        self.gamemode.store(gamemode);
        {
            // use another scope so we instantly unlock abilities
            let mut abilities = self.abilities.lock().await;
            abilities.set_for_gamemode(gamemode);
        };
        self.send_abilities_update().await;

        self.living_entity.entity.invulnerable.store(
            matches!(gamemode, GameMode::Creative | GameMode::Spectator),
            std::sync::atomic::Ordering::Relaxed,
        );
        self.living_entity
            .entity
            .world
            .read()
            .await
            .broadcast_packet_all(&CPlayerInfoUpdate::new(
                0x04,
                &[pumpkin_protocol::client::play::Player {
                    uuid: self.gameprofile.id,
                    actions: vec![PlayerAction::UpdateGameMode((gamemode as i32).into())],
                }],
            ))
            .await;

        self.client
            .send_packet(&CGameEvent::new(
                GameEvent::ChangeGameMode,
                gamemode as i32 as f32,
            ))
            .await;
    }

    /// Send skin layers and used hand to all players
    pub async fn send_client_information(&self) {
        let config = self.config.lock().await;
        self.living_entity
            .entity
            .send_meta_data(Metadata::new(17, MetaDataType::Byte, config.skin_parts))
            .await;
        self.living_entity
            .entity
            .send_meta_data(Metadata::new(
                18,
                MetaDataType::Byte,
                config.main_hand as u8,
            ))
            .await;
    }

    pub async fn can_harvest(&self, block: &State, block_name: &str) -> bool {
        !block.tool_required
            || self
                .inventory
                .lock()
                .await
                .held_item()
                .map_or_else(|| false, |e| e.is_correct_for_drops(block_name))
    }

    pub async fn get_mining_speed(&self, block_name: &str) -> f32 {
        let mut speed = self
            .inventory
            .lock()
            .await
            .get_mining_speed(block_name)
            .await;
        // Haste
        if self.living_entity.has_effect(EffectType::Haste).await
            || self
                .living_entity
                .has_effect(EffectType::ConduitPower)
                .await
        {
            speed *= 1.0 + (self.get_haste_amplifier().await + 1) as f32 * 0.2;
        }
        // Fatigue
        if let Some(fatigue) = self
            .living_entity
            .get_effect(EffectType::MiningFatigue)
            .await
        {
            let fatigue_speed = match fatigue.amplifier {
                0 => 0.3,
                1 => 0.09,
                2 => 0.0027,
                _ => 8.1E-4,
            };
            speed *= fatigue_speed;
        }
        // TODO: Handle when in Water
        if !self.living_entity.entity.on_ground.load(Ordering::Relaxed) {
            speed /= 5.0;
        }
        speed
    }

    async fn get_haste_amplifier(&self) -> u32 {
        let mut i = 0;
        let mut j = 0;
        if let Some(effect) = self.living_entity.get_effect(EffectType::Haste).await {
            i = effect.amplifier;
        }
        if let Some(effect) = self
            .living_entity
            .get_effect(EffectType::ConduitPower)
            .await
        {
            j = effect.amplifier;
        }
        u32::from(i.max(j))
    }

    pub async fn send_message(
        &self,
        message: &TextComponent,
        chat_type: u32,
        sender_name: &TextComponent,
        target_name: Option<&TextComponent>,
    ) {
        self.client
            .send_packet(&CDisguisedChatMessage::new(
                message,
                (chat_type + 1).into(),
                sender_name,
                target_name,
            ))
            .await;
    }

    pub async fn drop_item(&self, server: &Server, stack: ItemStack) {
        let entity = server.add_entity(
            self.living_entity.entity.pos.load(),
            EntityType::ITEM,
            &self.world().await,
        );
        let item_entity = Arc::new(ItemEntity::new(entity, stack));
        self.world().await.spawn_entity(item_entity.clone()).await;
        item_entity.send_meta_packet().await;
    }

    pub async fn drop_held_item(&self, server: &Server, drop_stack: bool) {
        let mut inv = self.inventory.lock().await;
        if let Some(item) = inv.held_item_mut() {
            let drop_amount = if drop_stack { item.item_count } else { 1 };
            self.drop_item(server, ItemStack::new(drop_amount, item.item))
                .await;
            inv.decrease_current_stack(drop_amount);
        }
    }

    pub async fn send_system_message(&self, text: &TextComponent) {
        self.send_system_message_raw(text, false).await;
    }

    pub async fn send_system_message_raw(&self, text: &TextComponent, overlay: bool) {
        self.client
            .send_packet(&CSystemChatMessage::new(text, overlay))
            .await;
    }

    /// Sets the player's experience level and updates the client
    pub async fn set_experience(&self, level: i32, progress: f32, points: i32) {
        self.experience_level.store(level, Ordering::Relaxed);
        self.experience_progress.store(progress.clamp(0.0, 1.0));
        self.experience_points.store(points, Ordering::Relaxed);

        self.client
            .send_packet(&CSetExperience::new(
                progress.clamp(0.0, 1.0),
                level.into(),
                points.into(),
            ))
            .await;
    }

    /// Sets the player's experience level directly
    pub async fn set_experience_level(&self, new_level: i32, keep_progress: bool) {
        let progress = self.experience_progress.load();
        let mut points = self.experience_points.load(Ordering::Relaxed);

        // If keep progress is true then calculate the number of points needed to keep the same progress scaled
        if keep_progress {
            // Get our current level
            let current_level = self.experience_level.load(Ordering::Relaxed);
            let current_max_points = experience::points_in_level(current_level);
            // Calculate the max value for new level
            let new_max_points = experience::points_in_level(new_level);
            // Calculate the scaling factor
            let scale = new_max_points as f32 / current_max_points as f32;
            // Scale the points (Vanilla doesn't seem to recalculate progress so we won't)
            points = (points as f32 * scale) as i32;
        }

        self.set_experience(new_level, progress, points).await;
    }

    pub async fn add_effect(&self, effect: Effect, keep_fading: bool) {
        let mut flag: i8 = 0;

        if effect.ambient {
            flag |= 1;
        }
        if effect.show_particles {
            flag |= 2;
        }
        if effect.show_icon {
            flag |= 4;
        }
        if keep_fading {
            flag |= 8;
        }
        let effect_id = VarInt(effect.r#type as i32);
        self.client
            .send_packet(&CUpdateMobEffect::new(
                self.entity_id().into(),
                effect_id,
                effect.amplifier.into(),
                effect.duration.into(),
                flag,
            ))
            .await;
        self.living_entity.add_effect(effect).await;
    }

    /// Add experience levels to the player
    pub async fn add_experience_levels(&self, added_levels: i32) {
        let current_level = self.experience_level.load(Ordering::Relaxed);
        let new_level = current_level + added_levels;
        self.set_experience_level(new_level, true).await;
    }

    /// Set the player's experience points directly, Returns true if successful.
    pub async fn set_experience_points(&self, new_points: i32) -> bool {
        let current_points = self.experience_points.load(Ordering::Relaxed);

        if new_points == current_points {
            return true;
        }

        let current_level = self.experience_level.load(Ordering::Relaxed);
        let max_points = experience::points_in_level(current_level);

        if new_points < 0 || new_points > max_points {
            return false;
        }

        let progress = new_points as f32 / max_points as f32;
        self.set_experience(current_level, progress, new_points)
            .await;
        true
    }

    /// Add experience points to the player
    pub async fn add_experience_points(&self, added_points: i32) {
        let current_level = self.experience_level.load(Ordering::Relaxed);
        let current_points = self.experience_points.load(Ordering::Relaxed);
        let total_exp = experience::points_to_level(current_level) + current_points;
        let new_total_exp = total_exp + added_points;
        let (new_level, new_points) = experience::total_to_level_and_points(new_total_exp);
        let progress = experience::progress_in_level(new_level, new_points);
        self.set_experience(new_level, progress, new_points).await;
    }
}

#[async_trait]
impl NBTStorage for Player {
    async fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.living_entity.write_nbt(nbt).await;
        nbt.put_int(
            "SelectedItemSlot",
            self.inventory.lock().await.selected as i32,
        );
        self.abilities.lock().await.write_nbt(nbt).await;

        // Store total XP instead of individual components
        let total_exp = experience::points_to_level(self.experience_level.load(Ordering::Relaxed))
            + self.experience_points.load(Ordering::Relaxed);
        nbt.put_int("XpTotal", total_exp);
    }

    async fn read_nbt(&mut self, nbt: &mut NbtCompound) {
        self.living_entity.read_nbt(nbt).await;
        self.inventory.lock().await.selected = nbt.get_int("SelectedItemSlot").unwrap_or(0) as u32;
        self.abilities.lock().await.read_nbt(nbt).await;

        // Load from total XP
        let total_exp = nbt.get_int("XpTotal").unwrap_or(0);
        let (level, points) = experience::total_to_level_and_points(total_exp);
        let progress = experience::progress_in_level(level, points);
        self.experience_level.store(level, Ordering::Relaxed);
        self.experience_progress.store(progress);
        self.experience_points.store(points, Ordering::Relaxed);
    }
}

#[async_trait]
impl EntityBase for Player {
    fn get_entity(&self) -> &Entity {
        &self.living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.living_entity)
    }
}

impl Player {
    pub async fn process_packets(self: &Arc<Self>, server: &Arc<Server>) {
        let mut packets = self.client.client_packets_queue.lock().await;
        while let Some(mut packet) = packets.pop_back() {
            tokio::select! {
                () = self.await_cancel() => {
                    log::debug!("Canceling player packet processing");
                    return;
                },
                packet_result = self.handle_play_packet(server, &mut packet) => {
                    match packet_result {
                        Ok(()) => {}
                        Err(e) => {
                            if e.is_kick() {
                                if let Some(kick_reason) = e.client_kick_reason() {
                                    self.kick(TextComponent::text(kick_reason)).await;
                                } else {
                                    self.kick(TextComponent::text(format!(
                                        "Error while reading incoming packet {e}"
                                    )))
                                    .await;
                                }
                            }
                            e.log();
                        }
                    };
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub async fn handle_play_packet(
        self: &Arc<Self>,
        server: &Arc<Server>,
        packet: &mut RawPacket,
    ) -> Result<(), Box<dyn PumpkinError>> {
        let bytebuf = &mut packet.bytebuf;
        match packet.id.0 {
            SConfirmTeleport::PACKET_ID => {
                self.handle_confirm_teleport(SConfirmTeleport::read(bytebuf)?)
                    .await;
            }
            SChatCommand::PACKET_ID => {
                self.handle_chat_command(server, &(SChatCommand::read(bytebuf)?));
            }
            SChatMessage::PACKET_ID => {
                self.handle_chat_message(SChatMessage::read(bytebuf)?).await;
            }
            SClientInformationPlay::PACKET_ID => {
                self.handle_client_information(SClientInformationPlay::read(bytebuf)?)
                    .await;
            }
            SClientCommand::PACKET_ID => {
                self.handle_client_status(SClientCommand::read(bytebuf)?)
                    .await;
            }
            SPlayerInput::PACKET_ID => {
                // TODO
            }
            SInteract::PACKET_ID => {
                self.handle_interact(SInteract::read(bytebuf)?).await;
            }
            SKeepAlive::PACKET_ID => {
                self.handle_keep_alive(SKeepAlive::read(bytebuf)?).await;
            }
            SClientTickEnd::PACKET_ID => {
                // TODO
            }
            SPlayerPosition::PACKET_ID => {
                self.handle_position(SPlayerPosition::read(bytebuf)?).await;
            }
            SPlayerPositionRotation::PACKET_ID => {
                self.handle_position_rotation(SPlayerPositionRotation::read(bytebuf)?)
                    .await;
            }
            SPlayerRotation::PACKET_ID => {
                self.handle_rotation(SPlayerRotation::read(bytebuf)?).await;
            }
            SSetPlayerGround::PACKET_ID => {
                self.handle_player_ground(&SSetPlayerGround::read(bytebuf)?);
            }
            SPickItemFromBlock::PACKET_ID => {
                self.handle_pick_item_from_block(SPickItemFromBlock::read(bytebuf)?)
                    .await;
            }
            SPlayerAbilities::PACKET_ID => {
                self.handle_player_abilities(SPlayerAbilities::read(bytebuf)?)
                    .await;
            }
            SPlayerAction::PACKET_ID => {
                self.clone()
                    .handle_player_action(SPlayerAction::read(bytebuf)?, server)
                    .await;
            }
            SPlayerCommand::PACKET_ID => {
                self.handle_player_command(SPlayerCommand::read(bytebuf)?)
                    .await;
            }
            SPlayerLoaded::PACKET_ID => self.handle_player_loaded(),
            SPlayPingRequest::PACKET_ID => {
                self.handle_play_ping_request(SPlayPingRequest::read(bytebuf)?)
                    .await;
            }
            SClickContainer::PACKET_ID => {
                self.handle_click_container(server, SClickContainer::read(bytebuf)?)
                    .await?;
            }
            SSetHeldItem::PACKET_ID => {
                self.handle_set_held_item(SSetHeldItem::read(bytebuf)?)
                    .await;
            }
            SSetCreativeSlot::PACKET_ID => {
                self.handle_set_creative_slot(server, SSetCreativeSlot::read(bytebuf)?)
                    .await?;
            }
            SSwingArm::PACKET_ID => {
                self.handle_swing_arm(SSwingArm::read(bytebuf)?).await;
            }
            SUpdateSign::PACKET_ID => {
                self.handle_sign_update(SUpdateSign::read(bytebuf)?).await;
            }
            SUseItemOn::PACKET_ID => {
                self.handle_use_item_on(SUseItemOn::read(bytebuf)?, server)
                    .await?;
            }
            SUseItem::PACKET_ID => {
                self.handle_use_item(&SUseItem::read(bytebuf)?, server)
                    .await;
            }
            SCommandSuggestion::PACKET_ID => {
                self.handle_command_suggestion(SCommandSuggestion::read(bytebuf)?, server)
                    .await;
            }
            SPCookieResponse::PACKET_ID => {
                self.handle_cookie_response(SPCookieResponse::read(bytebuf)?);
            }
            SCloseContainer::PACKET_ID => {
                self.handle_close_container(server, SCloseContainer::read(bytebuf)?)
                    .await;
            }
            _ => {
                log::warn!("Failed to handle player packet id {}", packet.id.0);
                // TODO: We give an error if all play packets are implemented
                //  return Err(Box::new(DeserializerError::UnknownPacket));
            }
        };
        Ok(())
    }
}

#[derive(Debug)]
pub enum TitleMode {
    Title,
    SubTitle,
    ActionBar,
}

/// Represents a player's abilities and special powers.
///
/// This struct contains information about the player's current abilities, such as flight, invulnerability, and creative mode.
pub struct Abilities {
    /// Indicates whether the player is invulnerable to damage.
    pub invulnerable: bool,
    /// Indicates whether the player is currently flying.
    pub flying: bool,
    /// Indicates whether the player is allowed to fly (if enabled).
    pub allow_flying: bool,
    /// Indicates whether the player is in creative mode.
    pub creative: bool,
    /// Indicates whether the player is allowed to modify the world.
    pub allow_modify_world: bool,
    /// The player's flying speed.
    pub fly_speed: f32,
    /// The field of view adjustment when the player is walking or sprinting.
    pub walk_speed: f32,
}

#[async_trait]
impl NBTStorage for Abilities {
    async fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        let mut component = NbtCompound::new();
        component.put_bool("invulnerable", self.invulnerable);
        component.put_bool("flying", self.flying);
        component.put_bool("mayfly", self.allow_flying);
        component.put_bool("instabuild", self.creative);
        component.put_bool("mayBuild", self.allow_modify_world);
        component.put_float("flySpeed", self.fly_speed);
        component.put_float("walkSpeed", self.walk_speed);
        nbt.put_component("abilities", component);
    }

    async fn read_nbt(&mut self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        if let Some(component) = nbt.get_compound("abilities") {
            self.invulnerable = component.get_bool("invulnerable").unwrap_or(false);
            self.flying = component.get_bool("flying").unwrap_or(false);
            self.allow_flying = component.get_bool("mayfly").unwrap_or(false);
            self.creative = component.get_bool("instabuild").unwrap_or(false);
            self.allow_modify_world = component.get_bool("mayBuild").unwrap_or(false);
            self.fly_speed = component.get_float("flySpeed").unwrap_or(0.0);
            self.walk_speed = component.get_float("walk_speed").unwrap_or(0.0);
        }
    }
}

impl Default for Abilities {
    fn default() -> Self {
        Self {
            invulnerable: false,
            flying: false,
            allow_flying: false,
            creative: false,
            allow_modify_world: true,
            fly_speed: 0.05,
            walk_speed: 0.1,
        }
    }
}

impl Abilities {
    pub fn set_for_gamemode(&mut self, gamemode: GameMode) {
        match gamemode {
            GameMode::Creative => {
                // self.flying = false; // Start not flying
                self.allow_flying = true;
                self.creative = true;
                self.invulnerable = true;
            }
            GameMode::Spectator => {
                self.flying = true;
                self.allow_flying = true;
                self.creative = false;
                self.invulnerable = true;
            }
            _ => {
                self.flying = false;
                self.allow_flying = false;
                self.creative = false;
                self.invulnerable = false;
            }
        }
    }
}

/// Represents the player's dominant hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    /// Usually the player's off-hand.
    Left,
    /// Usually the player's primary hand.
    Right,
}

pub struct InvalidHand;

impl TryFrom<i32> for Hand {
    type Error = InvalidHand;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Left),
            1 => Ok(Self::Right),
            _ => Err(InvalidHand),
        }
    }
}

/// Represents the player's chat mode settings.
#[derive(Debug, Clone)]
pub enum ChatMode {
    /// Chat is enabled for the player.
    Enabled,
    /// The player should only see chat messages from commands
    CommandsOnly,
    /// All messages should be hidden
    Hidden,
}

pub struct InvalidChatMode;

impl TryFrom<i32> for ChatMode {
    type Error = InvalidChatMode;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Enabled),
            1 => Ok(Self::CommandsOnly),
            2 => Ok(Self::Hidden),
            _ => Err(InvalidChatMode),
        }
    }
}
