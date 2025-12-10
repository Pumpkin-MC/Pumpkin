use core::f32;
use std::collections::{BinaryHeap, HashSet, VecDeque};
use std::f64::consts::TAU;
use std::num::NonZeroU8;
use std::ops::AddAssign;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, AtomicU8, AtomicU32, Ordering};
use std::time::{Duration, Instant};

use crossbeam::atomic::AtomicCell;
use crossbeam::channel::Receiver;
use log::warn;
use pumpkin_inventory::player::ender_chest_inventory::EnderChestInventory;
use pumpkin_protocol::bedrock::client::level_chunk::CLevelChunk;
use pumpkin_protocol::bedrock::client::set_time::CSetTime;
use pumpkin_protocol::bedrock::client::update_abilities::{
    Ability, AbilityLayer, CUpdateAbilities,
};
use pumpkin_protocol::bedrock::server::text::SText;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_world::chunk::{ChunkData, ChunkEntityData};
use pumpkin_world::inventory::Inventory;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use uuid::Uuid;

use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::{AttributeModifiersImpl, Operation};
use pumpkin_data::data_component_impl::{EquipmentSlot, EquippableImpl};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::{EntityPose, EntityStatus, EntityType};
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockState, tag};
use pumpkin_inventory::player::{
    player_inventory::PlayerInventory, player_screen_handler::PlayerScreenHandler,
};
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, PlayerFuture, ScreenHandler, ScreenHandlerFactory, ScreenHandlerListener,
};
use pumpkin_inventory::sync_handler::SyncHandler;
use pumpkin_macros::send_cancellable;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::IdOr;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::{
    Animation, CAcknowledgeBlockChange, CActionBar, CChangeDifficulty, CChunkBatchEnd,
    CChunkBatchStart, CChunkData, CCloseContainer, CCombatDeath, CDisguisedChatMessage,
    CEntityAnimation, CEntityPositionSync, CGameEvent, CKeepAlive, COpenScreen, CParticle,
    CPlayerAbilities, CPlayerInfoUpdate, CPlayerPosition, CPlayerSpawnPosition, CRespawn,
    CSetContainerContent, CSetContainerProperty, CSetContainerSlot, CSetCursorItem, CSetEquipment,
    CSetExperience, CSetHealth, CSetPlayerInventory, CSetSelectedSlot, CSoundEffect, CStopSound,
    CSubtitle, CSystemChatMessage, CTitleText, CUnloadChunk, CUpdateMobEffect, CUpdateTime,
    GameEvent, MetaDataType, Metadata, PlayerAction, PlayerInfoFlags, PreviousMessage,
};
use pumpkin_protocol::java::server::play::SClickSlot;
use pumpkin_registry::VanillaDimensionType;
use pumpkin_util::math::{
    boundingbox::BoundingBox, experience, position::BlockPos, vector2::Vector2, vector3::Vector3,
};
use pumpkin_util::permission::PermissionLvl;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_util::{GameMode, Hand};
use pumpkin_world::biome;
use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;
use pumpkin_world::entity::entity_data_flags::SLEEPING_POS_ID;
use pumpkin_world::item::ItemStack;
use pumpkin_world::level::{Level, SyncChunk, SyncEntityChunk};

use crate::block::blocks::bed::BedBlock;
use crate::command::client_suggestions;
use crate::command::dispatcher::CommandDispatcher;
use crate::data::op_data::OPERATOR_CONFIG;
use crate::entity::{EntityBaseFuture, NbtFuture, TeleportFuture};
use crate::net::{ClientPlatform, GameProfile};
use crate::net::{DisconnectReason, PlayerConfig};
use crate::plugin::player::player_change_world::PlayerChangeWorldEvent;
use crate::plugin::player::player_gamemode_change::PlayerGamemodeChangeEvent;
use crate::plugin::player::player_teleport::PlayerTeleportEvent;
use crate::server::Server;
use crate::world::World;
use crate::{PERMISSION_MANAGER, block};

use super::combat::{self, AttackType, player_attack_sound};
use super::hunger::HungerManager;
use super::item::ItemEntity;
use super::living::LivingEntity;
use super::{Entity, EntityBase, NBTStorage, NBTStorageInit};
use pumpkin_data::potion::Effect;
use pumpkin_world::chunk_system::ChunkLoading;

const MAX_CACHED_SIGNATURES: u8 = 128; // Vanilla: 128
const MAX_PREVIOUS_MESSAGES: u8 = 20; // Vanilla: 20

pub const DATA_VERSION: i32 = 4556; // 1.21.10

enum BatchState {
    Initial,
    Waiting,
    Count(u8),
}

struct HeapNode(i32, Vector2<i32>, SyncChunk);

impl Eq for HeapNode {}

impl PartialEq<Self> for HeapNode {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd<Self> for HeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

pub struct ChunkManager {
    chunks_per_tick: usize,
    center: Vector2<i32>,
    view_distance: u8,
    chunk_listener: Receiver<(Vector2<i32>, SyncChunk)>,
    chunk_sent: HashSet<Vector2<i32>>,
    chunk_queue: BinaryHeap<HeapNode>,
    entity_chunk_queue: VecDeque<(Vector2<i32>, SyncEntityChunk)>,
    batches_sent_since_ack: BatchState,
}

impl ChunkManager {
    pub const NOTCHIAN_BATCHES_WITHOUT_ACK_UNTIL_PAUSE: u8 = 10;

    #[must_use]
    pub fn new(
        chunks_per_tick: usize,
        chunk_listener: Receiver<(Vector2<i32>, SyncChunk)>,
    ) -> Self {
        Self {
            chunks_per_tick,
            center: Vector2::<i32>::new(0, 0),
            view_distance: 0,
            chunk_listener,
            chunk_sent: HashSet::new(),
            chunk_queue: BinaryHeap::new(),
            entity_chunk_queue: VecDeque::new(),
            batches_sent_since_ack: BatchState::Initial,
        }
    }

    pub fn pull_new_chunks(&mut self) {
        // log::debug!("pull new chunks");
        while let Ok((pos, chunk)) = self.chunk_listener.try_recv() {
            let dst = (pos.x - self.center.x)
                .abs()
                .max((pos.y - self.center.y).abs());
            if dst > i32::from(self.view_distance) {
                continue;
            }
            if self.chunk_sent.insert(pos) {
                // log::debug!("receive new chunk {pos:?}");
                self.chunk_queue.push(HeapNode(dst, pos, chunk));
            }
        }
        // log::debug!("chunk_queue size {}", self.chunk_queue.len());
        // log::debug!("chunk_sent size {}", self.chunk_sent.len());
    }

    pub fn update_center_and_view_distance(
        &mut self,
        center: Vector2<i32>,
        mut view_distance: u8,
        level: &Arc<Level>,
    ) {
        view_distance += 1;
        let mut lock = level.chunk_loading.lock().unwrap();
        lock.add_ticket(
            center,
            ChunkLoading::get_level_from_view_distance(view_distance),
        );
        lock.remove_ticket(
            self.center,
            ChunkLoading::get_level_from_view_distance(self.view_distance),
        );
        lock.send_change();
        drop(lock);
        let view_distance = i32::from(view_distance);
        self.chunk_sent
            .retain(|pos| (pos.x - center.x).abs().max((pos.y - center.y).abs()) <= view_distance);
        let mut new_queue = BinaryHeap::with_capacity(self.chunk_queue.len());
        for node in &self.chunk_queue {
            let dst = (node.1.x - center.x).abs().max((node.1.y - center.y).abs());
            if dst <= view_distance {
                new_queue.push(HeapNode(dst, node.1, node.2.clone()));
            }
        }
        self.chunk_queue = new_queue;
        self.center = center;
        self.view_distance = view_distance as u8;
        for dx in (-view_distance)..=view_distance {
            for dy in (-view_distance)..=view_distance {
                let new_pos = center.add_raw(dx, dy);
                if !self.chunk_sent.contains(&new_pos)
                    && let Some(chunk) = level.loaded_chunks.get(&new_pos)
                {
                    self.push_chunk(new_pos, chunk.value().clone());
                }
            }
        }
    }

    pub fn clean_up(&mut self, level: &Arc<Level>) {
        let mut lock = level.chunk_loading.lock().unwrap();
        lock.remove_ticket(
            self.center,
            ChunkLoading::get_level_from_view_distance(self.view_distance),
        );
        let (_rx, tx) = crossbeam::channel::unbounded();
        // drop old channel
        self.chunk_listener = tx;
    }

    pub fn change_world(&mut self, old_level: &Arc<Level>, new_level: &Arc<Level>) {
        let mut lock = old_level.chunk_loading.lock().unwrap();
        lock.remove_ticket(
            self.center,
            ChunkLoading::get_level_from_view_distance(self.view_distance),
        );
        drop(lock);
        self.chunk_listener = new_level.chunk_listener.add_global_chunk_listener();
        self.chunk_sent.clear();
        self.chunk_queue.clear();
    }

    pub fn handle_acknowledge(&mut self, chunks_per_tick: f32) {
        self.batches_sent_since_ack = BatchState::Count(0);
        self.chunks_per_tick = chunks_per_tick.ceil() as usize;
    }

    pub fn push_chunk(&mut self, position: Vector2<i32>, chunk: SyncChunk) {
        self.chunk_sent.insert(position);
        let dst = (position.x - self.center.x)
            .abs()
            .max((position.y - self.center.y).abs());
        self.chunk_queue.push(HeapNode(dst, position, chunk));
    }

    pub fn push_entity(&mut self, position: Vector2<i32>, chunk: SyncEntityChunk) {
        self.entity_chunk_queue.push_back((position, chunk));
    }

    #[must_use]
    pub fn can_send_chunk(&self) -> bool {
        let state_available = match self.batches_sent_since_ack {
            BatchState::Count(count) => count < Self::NOTCHIAN_BATCHES_WITHOUT_ACK_UNTIL_PAUSE,
            BatchState::Initial => true,
            BatchState::Waiting => false,
        };

        state_available && !self.chunk_queue.is_empty()
    }

    pub fn next_chunk(&mut self) -> Box<[SyncChunk]> {
        let mut chunk_size = self.chunk_queue.len().min(self.chunks_per_tick);
        let mut chunks = Vec::<Arc<RwLock<ChunkData>>>::with_capacity(chunk_size);
        while chunk_size > 0 {
            chunks.push(self.chunk_queue.pop().unwrap().2);
            chunk_size -= 1;
        }
        match &mut self.batches_sent_since_ack {
            BatchState::Count(count) => {
                count.add_assign(1);
            }
            state @ BatchState::Initial => *state = BatchState::Waiting,
            BatchState::Waiting => (),
        }

        chunks.into_boxed_slice()
    }

    pub fn next_entity(&mut self) -> Box<[SyncEntityChunk]> {
        let chunk_size = self.entity_chunk_queue.len().min(self.chunks_per_tick);
        let chunks: Vec<Arc<RwLock<ChunkEntityData>>> = self
            .entity_chunk_queue
            .drain(0..chunk_size)
            .map(|(_, chunk)| chunk)
            .collect();

        match &mut self.batches_sent_since_ack {
            BatchState::Count(count) => {
                count.add_assign(1);
            }
            state @ BatchState::Initial => *state = BatchState::Waiting,
            BatchState::Waiting => unreachable!(),
        }

        chunks.into_boxed_slice()
    }
}

/// Represents a Minecraft player entity.
///
/// A `Player` is a special type of entity that represents a human player connected to the server.
pub struct Player {
    /// The underlying living entity object that represents the player.
    pub living_entity: LivingEntity,
    /// The player's game profile information, including their username and UUID.
    pub gameprofile: GameProfile,
    /// The client connection associated with the player.
    pub client: ClientPlatform,
    /// The player's inventory.
    pub inventory: Arc<PlayerInventory>,
    /// The player's `EnderChest` inventory.
    pub ender_chest_inventory: Arc<EnderChestInventory>,
    /// The player's configuration settings. Changes when the player changes their settings.
    pub config: RwLock<PlayerConfig>,
    /// The player's current gamemode (e.g., Survival, Creative, Adventure).
    pub gamemode: AtomicCell<GameMode>,
    /// The player's previous gamemode
    pub previous_gamemode: AtomicCell<Option<GameMode>>,
    /// The player's spawnpoint
    pub respawn_point: AtomicCell<Option<RespawnPoint>>,
    /// The player's sleep status
    pub sleeping_since: AtomicCell<Option<u8>>,
    /// Manages the player's hunger level.
    pub hunger_manager: HungerManager,
    /// The ID of the currently open container (if any).
    pub open_container: AtomicCell<Option<u64>>,
    /// The item currently being held by the player.
    pub carried_item: Mutex<Option<ItemStack>>,
    /// The player's abilities and special powers.
    ///
    /// This field represents the various abilities that the player possesses, such as flight, invulnerability, and other special effects.
    ///
    /// **Note:** When the `abilities` field is updated, the server should send a `send_abilities_update` packet to the client to notify them of the changes.
    pub abilities: Mutex<Abilities>,
    /// The current stage of block destruction of the block the player is breaking.
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
    /// Whether we are waiting for a response after sending a keep alive packet.
    pub wait_for_keep_alive: AtomicBool,
    /// The keep alive packet payload we send. The client should respond with the same id.
    pub keep_alive_id: AtomicI64,
    /// The last time we sent a keep alive packet.
    pub last_keep_alive_time: AtomicCell<Instant>,
    /// The amount of ticks since the player's last attack.
    pub last_attacked_ticks: AtomicU32,
    /// The player's last known experience level.
    pub last_sent_xp: AtomicI32,
    pub last_sent_health: AtomicI32,
    pub last_sent_food: AtomicU8,
    pub last_food_saturation: AtomicBool,
    /// The player's permission level.
    pub permission_lvl: AtomicCell<PermissionLvl>,
    /// Whether the client has reported that it has loaded.
    pub client_loaded: AtomicBool,
    /// The amount of time (in ticks) the client has to report having finished loading before being timed out.
    pub client_loaded_timeout: AtomicU32,
    /// The player's experience level.
    pub experience_level: AtomicI32,
    /// The player's experience progress (`0.0` to `1.0`)
    pub experience_progress: AtomicCell<f32>,
    /// The player's total experience points.
    pub experience_points: AtomicI32,
    pub experience_pick_up_delay: Mutex<u32>,
    pub chunk_manager: Mutex<ChunkManager>,
    pub has_played_before: AtomicBool,
    pub chat_session: Arc<Mutex<ChatSession>>,
    pub signature_cache: Mutex<MessageCache>,
    pub player_screen_handler: Arc<Mutex<PlayerScreenHandler>>,
    pub current_screen_handler: Mutex<Arc<Mutex<dyn ScreenHandler>>>,
    pub screen_handler_sync_id: AtomicU8,
    pub screen_handler_listener: Arc<dyn ScreenHandlerListener>,
    pub screen_handler_sync_handler: Arc<SyncHandler>,
}

impl Player {
    pub async fn new(
        client: ClientPlatform,
        gameprofile: GameProfile,
        config: PlayerConfig,
        world: Arc<World>,
        gamemode: GameMode,
    ) -> Self {
        struct ScreenListener;

        impl ScreenHandlerListener for ScreenListener {}

        let server = world.server.upgrade().unwrap();

        let player_uuid = gameprofile.id;

        let living_entity = LivingEntity::new(Entity::new(
            player_uuid,
            world.clone(),
            Vector3::new(0.0, 100.0, 0.0),
            &EntityType::PLAYER,
            matches!(gamemode, GameMode::Creative | GameMode::Spectator),
        ));

        let inventory = Arc::new(PlayerInventory::new(
            living_entity.entity_equipment.clone(),
            living_entity.equipment_slots.clone(),
        ));

        let ender_chest_inventory = Arc::new(EnderChestInventory::new());

        let player_screen_handler = Arc::new(Mutex::new(
            PlayerScreenHandler::new(&inventory, None, 0).await,
        ));

        Self {
            living_entity,
            config: RwLock::new(config),
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
            carried_item: Mutex::new(None),
            experience_pick_up_delay: Mutex::new(0),
            teleport_id_count: AtomicI32::new(0),
            mining: AtomicBool::new(false),
            mining_pos: Mutex::new(BlockPos::ZERO),
            abilities: Mutex::new(Abilities::default()),
            gamemode: AtomicCell::new(gamemode),
            previous_gamemode: AtomicCell::new(None),
            // TODO: Send the CPlayerSpawnPosition packet when the client connects with proper values
            respawn_point: AtomicCell::new(None),
            sleeping_since: AtomicCell::new(None),
            // We want this to be an impossible watched section so that `chunker::update_position`
            // will mark chunks as watched for a new join rather than a respawn.
            // (We left shift by one so we can search around that chunk)
            watched_section: AtomicCell::new(Cylindrical::new(
                Vector2::new(0, 0),
                // Since 1 is not possible in vanilla it is used as uninit
                NonZeroU8::new(1).unwrap(),
            )),
            wait_for_keep_alive: AtomicBool::new(false),
            keep_alive_id: AtomicI64::new(0),
            last_keep_alive_time: AtomicCell::new(std::time::Instant::now()),
            last_attacked_ticks: AtomicU32::new(0),
            client_loaded: AtomicBool::new(false),
            client_loaded_timeout: AtomicU32::new(60),
            // Minecraft has no way to change the default permission level of new players.
            // Minecraft's default permission level is 0.
            permission_lvl: OPERATOR_CONFIG.read().await.get_entry(&player_uuid).map_or(
                AtomicCell::new(server.advanced_config.commands.default_op_level),
                |op| AtomicCell::new(op.level),
            ),
            inventory,
            ender_chest_inventory,
            experience_level: AtomicI32::new(0),
            experience_progress: AtomicCell::new(0.0),
            experience_points: AtomicI32::new(0),
            // Default to sending 16 chunks per tick.
            chunk_manager: Mutex::new(ChunkManager::new(
                16,
                world.level.chunk_listener.add_global_chunk_listener(),
            )),
            last_sent_xp: AtomicI32::new(-1),
            last_sent_health: AtomicI32::new(-1),
            last_sent_food: AtomicU8::new(0),
            last_food_saturation: AtomicBool::new(true),
            has_played_before: AtomicBool::new(false),
            chat_session: Arc::new(Mutex::new(ChatSession::default())), // Placeholder value until the player actually sets their session id
            signature_cache: Mutex::new(MessageCache::default()),
            player_screen_handler: player_screen_handler.clone(),
            current_screen_handler: Mutex::new(player_screen_handler),
            screen_handler_sync_id: AtomicU8::new(0),
            screen_handler_listener: Arc::new(ScreenListener {}),
            screen_handler_sync_handler: Arc::new(SyncHandler::new()),
        }
    }

    /// Spawns a task associated with this player-client. All tasks spawned with this method are awaited
    /// when the client. This means tasks should complete in a reasonable amount of time or select
    /// on `Self::await_close_interrupt` to cancel the task when the client is closed
    ///
    /// Returns an `Option<JoinHandle<F::Output>>`. If the client is closed, this returns `None`.
    pub fn spawn_task<F>(&self, task: F) -> Option<JoinHandle<F::Output>>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.client.spawn_task(task)
    }

    pub fn inventory(&self) -> &Arc<PlayerInventory> {
        &self.inventory
    }

    pub fn ender_chest_inventory(&self) -> &Arc<EnderChestInventory> {
        &self.ender_chest_inventory
    }

    /// Removes the [`Player`] out of the current [`World`].
    pub async fn remove(self: &Arc<Self>) {
        let world = self.world();
        world.remove_player(self, true).await;

        let cylindrical = self.watched_section.load();
        self.chunk_manager.lock().await.clean_up(&world.level);

        // Radial chunks are all of the chunks the player is theoretically viewing.
        // Given enough time, all of these chunks will be in memory.
        let radial_chunks = cylindrical.all_chunks_within();

        log::debug!(
            "Removing player {}, unwatching {} chunks",
            self.gameprofile.name,
            radial_chunks.len()
        );

        let level = &world.level;

        // Decrement the value of watched chunks
        let chunks_to_clean = level.mark_chunks_as_not_watched(&radial_chunks).await;
        // Remove chunks with no watchers from the cache
        level.clean_entity_chunks(&chunks_to_clean).await;
        // Remove left over entries from all possiblily loaded chunks
        level.clean_memory();

        log::debug!(
            "Removed player id {} from world {} ({} chunks remain cached)",
            self.gameprofile.name,
            "world", // TODO: Add world names
            level.loaded_chunk_count(),
        );

        level.clean_up_log().await;

        //self.world().level.list_cached();
    }

    pub async fn attack(&self, victim: Arc<dyn EntityBase>) {
        let world = self.world();
        let server = world.server.upgrade().unwrap();
        let victim_entity = victim.get_entity();
        let attacker_entity = &self.living_entity.entity;
        let config = &server.advanced_config.pvp;

        let inventory = self.inventory();
        let item_stack = inventory.held_item();

        let base_damage = 1.0;
        let base_attack_speed = 4.0;

        let mut damage_multiplier = 1.0;
        let mut add_damage = 0.0;
        let mut add_speed = 0.0;

        // Get the attack damage
        // TODO: this should be cached in memory, we shouldn't just use default here either
        if let Some(modifiers) = item_stack
            .lock()
            .await
            .get_data_component::<AttributeModifiersImpl>()
        {
            for item_mod in modifiers.attribute_modifiers.iter() {
                if item_mod.operation == Operation::AddValue {
                    if item_mod.id == "minecraft:base_attack_damage" {
                        add_damage = item_mod.amount;
                    } else if item_mod.id == "minecraft:base_attack_speed" {
                        add_speed = item_mod.amount;
                    }
                }
            }
        }

        let attack_speed = base_attack_speed + add_speed;

        let attack_cooldown_progress = self.get_attack_cooldown_progress(
            f64::from(server.basic_config.tps),
            0.5,
            attack_speed,
        );
        self.last_attacked_ticks.store(0, Ordering::Relaxed);

        // Only reduce attack damage if in cooldown
        // TODO: Enchantments are reduced in the same way, just without the square.
        if attack_cooldown_progress < 1.0 {
            damage_multiplier = 0.2 + attack_cooldown_progress.powi(2) * 0.8;
        }
        // Modify the added damage based on the multiplier.
        let mut damage = base_damage + add_damage * damage_multiplier;

        let pos = victim_entity.pos.load();

        let attack_type = AttackType::new(self, attack_cooldown_progress as f32).await;

        if matches!(attack_type, AttackType::Critical) {
            damage *= 1.5;
        }

        if !victim
            .damage_with_context(
                victim.clone(),
                damage as f32,
                DamageType::PLAYER_ATTACK,
                None,
                Some(&self.living_entity.entity),
                Some(&self.living_entity.entity),
            )
            .await
        {
            world
                .play_sound(
                    Sound::EntityPlayerAttackNodamage,
                    SoundCategory::Players,
                    &self.living_entity.entity.pos.load(),
                )
                .await;
            return;
        }

        if victim.get_living_entity().is_some() {
            let mut knockback_strength = 1.0;
            player_attack_sound(&pos, world, attack_type).await;
            match attack_type {
                AttackType::Knockback => knockback_strength += 1.0,
                AttackType::Sweeping => {
                    combat::spawn_sweep_particle(attacker_entity, world, &pos).await;
                }
                _ => {}
            }
            if config.knockback {
                combat::handle_knockback(attacker_entity, world, victim_entity, knockback_strength)
                    .await;
            }
        }

        if config.swing {}
    }

    pub async fn set_respawn_point(
        &self,
        dimension: VanillaDimensionType,
        block_pos: BlockPos,
        yaw: f32,
    ) -> bool {
        if let Some(respawn_point) = self.respawn_point.load()
            && dimension == respawn_point.dimension
            && block_pos == respawn_point.position
        {
            return false;
        }

        self.respawn_point.store(Some(RespawnPoint {
            dimension,
            position: block_pos,
            yaw,
            force: false,
        }));

        self.client
            .send_packet_now(&CPlayerSpawnPosition::new(block_pos, yaw))
            .await;
        true
    }

    pub async fn get_respawn_point(&self) -> Option<(Vector3<f64>, f32)> {
        let respawn_point = self.respawn_point.load()?;

        let block = self.world().get_block(&respawn_point.position).await;

        if respawn_point.dimension == VanillaDimensionType::Overworld
            && block.has_tag(&tag::Block::MINECRAFT_BEDS)
        {
            // TODO: calculate respawn position
            Some((respawn_point.position.to_f64(), respawn_point.yaw))
        } else if respawn_point.dimension == VanillaDimensionType::TheNether
            && block == &Block::RESPAWN_ANCHOR
        {
            // TODO: calculate respawn position
            // TODO: check if there is fuel for respawn
            Some((respawn_point.position.to_f64(), respawn_point.yaw))
        } else {
            self.client
                .send_packet_now(&CGameEvent::new(GameEvent::NoRespawnBlockAvailable, 0.0))
                .await;

            None
        }
    }

    pub async fn sleep(&self, bed_head_pos: BlockPos) {
        // TODO: Stop riding

        self.get_entity().set_pose(EntityPose::Sleeping).await;
        self.living_entity
            .entity
            .set_pos(bed_head_pos.to_f64().add_raw(0.5, 0.6875, 0.5));
        self.get_entity()
            .send_meta_data(&[Metadata::new(
                SLEEPING_POS_ID,
                MetaDataType::OptionalBlockPos,
                Some(bed_head_pos),
            )])
            .await;
        self.get_entity().set_velocity(Vector3::default()).await;

        self.sleeping_since.store(Some(0));
    }

    pub async fn get_off_ground_speed(&self) -> f64 {
        let sprinting = self.get_entity().sprinting.load(Ordering::Relaxed);

        if !self.get_entity().has_vehicle().await {
            let fly_speed = {
                let abilities = self.abilities.lock().await;

                abilities.flying.then_some(f64::from(abilities.fly_speed))
            };

            if let Some(flying) = fly_speed {
                return if sprinting { flying * 2.0 } else { flying };
            }
        }

        if sprinting { 0.025_999_999 } else { 0.02 }
    }

    pub async fn is_flying(&self) -> bool {
        let abilities = self.abilities.lock().await;
        abilities.flying
    }

    pub async fn wake_up(&self) {
        let world = self.world();
        let respawn_point = self
            .respawn_point
            .load()
            .expect("Player waking up should have it's respawn point set on the bed.");

        let (bed, bed_state) = world.get_block_and_state_id(&respawn_point.position).await;
        BedBlock::set_occupied(false, world, bed, &respawn_point.position, bed_state).await;

        self.living_entity
            .entity
            .set_pose(EntityPose::Standing)
            .await;
        self.living_entity.entity.set_pos(self.position());
        self.living_entity
            .entity
            .send_meta_data(&[Metadata::new(
                SLEEPING_POS_ID,
                MetaDataType::OptionalBlockPos,
                None::<BlockPos>,
            )])
            .await;

        world
            .broadcast_packet_all(&CEntityAnimation::new(
                self.entity_id().into(),
                Animation::LeaveBed,
            ))
            .await;

        self.sleeping_since.store(None);
    }

    pub async fn show_title(&self, text: &TextComponent, mode: &TitleMode) {
        match mode {
            TitleMode::Title => self.client.enqueue_packet(&CTitleText::new(text)).await,
            TitleMode::SubTitle => self.client.enqueue_packet(&CSubtitle::new(text)).await,
            TitleMode::ActionBar => self.client.enqueue_packet(&CActionBar::new(text)).await,
        }
    }

    pub async fn spawn_particle(
        &self,
        position: Vector3<f64>,
        offset: Vector3<f32>,
        max_speed: f32,
        particle_count: i32,
        particle: Particle,
    ) {
        self.client
            .enqueue_packet(&CParticle::new(
                false,
                false,
                position,
                offset,
                max_speed,
                particle_count,
                VarInt(particle as i32),
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
            .enqueue_packet(&CSoundEffect::new(
                IdOr::Id(sound_id),
                category,
                position,
                volume,
                pitch,
                seed,
            ))
            .await;
    }

    /// Stops a sound playing on the client.
    ///
    /// # Arguments
    ///
    /// * `sound_id`: An optional [`ResourceLocation`] specifying the sound to stop. If [`None`], all sounds in the specified category (if any) will be stopped.
    /// * `category`: An optional [`SoundCategory`] specifying the sound category to stop. If [`None`], all sounds with the specified resource location (if any) will be stopped.
    pub async fn stop_sound(
        &self,
        sound_id: Option<ResourceLocation>,
        category: Option<SoundCategory>,
    ) {
        self.client
            .enqueue_packet(&CStopSound::new(sound_id, category))
            .await;
    }

    // TODO Abstract the chunk sending
    #[allow(clippy::too_many_lines)]
    pub async fn tick(self: &Arc<Self>, server: &Server) {
        self.current_screen_handler
            .lock()
            .await
            .lock()
            .await
            .send_content_updates()
            .await;

        // if self.client.closed.load(Ordering::Relaxed) {
        //     return;
        // }

        if self.packet_sequence.load(Ordering::Relaxed) > -1 {
            self.client
                .enqueue_packet(&CAcknowledgeBlockChange::new(
                    self.packet_sequence.swap(-1, Ordering::Relaxed).into(),
                ))
                .await;
        }
        {
            let mut xp = self.experience_pick_up_delay.lock().await;
            if *xp > 0 {
                *xp -= 1;
            }
        }

        let chunk_of_chunks = {
            let mut chunk_manager = self.chunk_manager.lock().await;
            chunk_manager.pull_new_chunks();
            if let ClientPlatform::Java(_) = self.client {
                // Java clients can only send a limited amount of chunks per tick.
                // If we have sent too many chunks without receiving an ack, we stop sending chunks.
                chunk_manager
                    .can_send_chunk()
                    .then(|| chunk_manager.next_chunk())
            } else {
                Some(chunk_manager.next_chunk())
            }
        };

        if let Some(chunk_of_chunks) = chunk_of_chunks {
            let chunk_count = chunk_of_chunks.len();
            match &self.client {
                ClientPlatform::Java(java_client) => {
                    java_client.send_packet_now(&CChunkBatchStart).await;
                    for chunk in chunk_of_chunks {
                        let chunk = chunk.read().await;
                        // log::debug!("send chunk {:?}", chunk.position);
                        // TODO: Can we check if we still need to send the chunk? Like if it's a fast moving
                        // player or something.
                        java_client.send_packet_now(&CChunkData(&chunk)).await;
                    }
                    java_client
                        .send_packet_now(&CChunkBatchEnd::new(chunk_count as u16))
                        .await;
                }
                ClientPlatform::Bedrock(bedrock_client) => {
                    for chunk in chunk_of_chunks {
                        let chunk = chunk.read().await;

                        bedrock_client
                            .send_game_packet(&CLevelChunk {
                                dimension: 0,
                                cache_enabled: false,
                                chunk: &chunk,
                            })
                            .await;
                    }
                }
            }
        }

        self.tick_counter.fetch_add(1, Ordering::Relaxed);
        if let Some(sleeping_since) = self.sleeping_since.load()
            && sleeping_since < 101
        {
            self.sleeping_since.store(Some(sleeping_since + 1));
        }

        if self.mining.load(Ordering::Relaxed) {
            let pos = self.mining_pos.lock().await;
            let world = self.world();
            let state = world.get_block_state(&pos).await;
            // Is the block broken?
            if state.is_air() {
                world
                    .set_block_breaking(&self.living_entity.entity, *pos, -1)
                    .await;
                self.current_block_destroy_stage
                    .store(-1, Ordering::Relaxed);
                self.mining.store(false, Ordering::Relaxed);
            } else {
                self.continue_mining(
                    *pos,
                    world,
                    state,
                    self.start_mining_time.load(Ordering::Relaxed),
                )
                .await;
            }
        }

        self.last_attacked_ticks.fetch_add(1, Ordering::Relaxed);

        self.living_entity.tick(self.clone(), server).await;
        self.hunger_manager.tick(self).await;

        // experience handling
        self.tick_experience().await;
        self.tick_health().await;

        // Timeout/keep alive handling
        self.tick_client_load_timeout();

        // TODO This should only be handled by the ClientPlatform
        let now = Instant::now();
        if now.duration_since(self.last_keep_alive_time.load()) >= Duration::from_secs(15) {
            if matches!(self.client, ClientPlatform::Bedrock(_)) {
                return;
            }
            // We never got a response from the last keep alive we sent.
            if self.wait_for_keep_alive.load(Ordering::Relaxed) {
                self.kick(
                    DisconnectReason::Timeout,
                    TextComponent::translate("disconnect.timeout", []),
                )
                .await;
                return;
            }
            self.wait_for_keep_alive.store(true, Ordering::Relaxed);
            self.last_keep_alive_time.store(now);
            let id = now.elapsed().as_millis() as i64;
            self.keep_alive_id.store(id, Ordering::Relaxed);
            self.client.enqueue_packet(&CKeepAlive::new(id)).await;
        }
    }

    async fn continue_mining(
        &self,
        location: BlockPos,
        world: &World,
        state: &BlockState,
        starting_time: i32,
    ) {
        let time = self.tick_counter.load(Ordering::Relaxed) - starting_time;
        let speed = block::calc_block_breaking(self, state, Block::from_state_id(state.id)).await
            * (time + 1) as f32;
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
        // TODO: Swimming, gliding...
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

    pub fn get_attack_cooldown_progress(&self, tps: f64, base_time: f64, attack_speed: f64) -> f64 {
        let x = f64::from(self.last_attacked_ticks.load(Ordering::Acquire)) + base_time;

        let progress_per_tick = tps / attack_speed;
        let progress = x / progress_per_tick;
        progress.clamp(0.0, 1.0)
    }

    pub const fn entity_id(&self) -> i32 {
        self.living_entity.entity.entity_id
    }

    pub fn world(&self) -> &Arc<World> {
        &self.living_entity.entity.world
    }

    pub fn position(&self) -> Vector3<f64> {
        self.living_entity.entity.pos.load()
    }

    pub fn eye_position(&self) -> Vector3<f64> {
        let eye_height = if self.living_entity.entity.pose.load() == EntityPose::Crouching {
            1.27
        } else {
            f64::from(self.living_entity.entity.standing_eye_height)
        };
        Vector3::new(
            self.living_entity.entity.pos.load().x,
            self.living_entity.entity.pos.load().y + eye_height,
            self.living_entity.entity.pos.load().z,
        )
    }

    pub fn rotation(&self) -> (f32, f32) {
        (
            self.living_entity.entity.yaw.load(),
            self.living_entity.entity.pitch.load(),
        )
    }

    /// Updates the current abilities the player has.
    pub async fn send_abilities_update(&self) {
        match &self.client {
            ClientPlatform::Java(java) => {
                let mut b = 0;
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
                java.enqueue_packet(&CPlayerAbilities::new(
                    b,
                    abilities.fly_speed,
                    abilities.walk_speed,
                ))
                .await;
            }
            ClientPlatform::Bedrock(bedrock) => {
                let mut ability_value = 0;
                let abilities = &self.abilities.lock().await;

                if abilities.invulnerable {
                    ability_value |= 1 << Ability::Invulnerable as u32;
                }

                if abilities.flying {
                    ability_value |= 1 << Ability::Flying as u32;
                }

                if abilities.allow_flying {
                    ability_value |= 1 << Ability::MayFly as u32;
                }

                if abilities.creative {
                    ability_value |= 1 << Ability::OperatorCommands as u32;
                    ability_value |= 1 << Ability::Teleport as u32;
                    ability_value |= 1 << Ability::Invulnerable as u32;
                }

                // Todo: Integrate this into the system
                ability_value |= 1 << Ability::AttackMobs as u32;
                ability_value |= 1 << Ability::AttackPlayers as u32;
                ability_value |= 1 << Ability::Build as u32;
                ability_value |= 1 << Ability::DoorsAndSwitches as u32;
                ability_value |= 1 << Ability::Instabuild as u32;
                ability_value |= 1 << Ability::Mine as u32;

                let packet = CUpdateAbilities {
                    target_player_raw_id: self.entity_id().into(),
                    player_permission: 2,
                    command_permission: 4,
                    layers: vec![AbilityLayer {
                        serialized_layer: 1,
                        abilities_set: (1 << Ability::AbilityCount as u32) - 1,
                        ability_value,
                        fly_speed: 0.05,
                        vertical_fly_speed: 1.0,
                        walk_speed: 0.1,
                    }],
                };

                bedrock.send_game_packet(&packet).await;
            }
        }
    }

    /// Updates the client of the player's current permission level.
    pub async fn send_permission_lvl_update(&self) {
        let status = match self.permission_lvl.load() {
            PermissionLvl::Zero => EntityStatus::SetOpLevel0,
            PermissionLvl::One => EntityStatus::SetOpLevel1,
            PermissionLvl::Two => EntityStatus::SetOpLevel2,
            PermissionLvl::Three => EntityStatus::SetOpLevel3,
            PermissionLvl::Four => EntityStatus::SetOpLevel4,
        };
        self.world()
            .send_entity_status(&self.living_entity.entity, status)
            .await;
    }

    /// Sets the player's difficulty level.
    pub async fn send_difficulty_update(&self) {
        let world = self.world();
        let level_info = world.level_info.read().await;
        self.client
            .enqueue_packet(&CChangeDifficulty::new(
                level_info.difficulty as u8,
                level_info.difficulty_locked,
            ))
            .await;
    }

    /// Sets the player's permission level and notifies the client.
    pub async fn set_permission_lvl(
        self: &Arc<Self>,
        lvl: PermissionLvl,
        command_dispatcher: &CommandDispatcher,
    ) {
        self.permission_lvl.store(lvl);
        self.send_permission_lvl_update().await;
        client_suggestions::send_c_commands_packet(self, command_dispatcher).await;
    }

    /// Sends the world time to only this player.
    pub async fn send_time(&self, world: &World) {
        let l_world = world.level_time.lock().await;
        match &self.client {
            ClientPlatform::Java(java_client) => {
                java_client
                    .enqueue_packet(&CUpdateTime::new(
                        l_world.world_age,
                        l_world.time_of_day,
                        true,
                    ))
                    .await;
            }
            ClientPlatform::Bedrock(bedrock_client) => {
                bedrock_client
                    .send_game_packet(&CSetTime {
                        time: VarInt(l_world.query_daytime() as _),
                    })
                    .await;
            }
        }
    }

    async fn unload_watched_chunks(&self, world: &World) {
        let radial_chunks = self.watched_section.load().all_chunks_within();
        let level = &world.level;
        let chunks_to_clean = level.mark_chunks_as_not_watched(&radial_chunks).await;
        // level.clean_chunks(&chunks_to_clean).await;
        for chunk in chunks_to_clean {
            self.client
                .enqueue_packet(&CUnloadChunk::new(chunk.x, chunk.y))
                .await;
        }

        self.watched_section.store(Cylindrical::new(
            Vector2::new(0, 0),
            NonZeroU8::new(1).unwrap(),
        ));
    }

    /// Teleports the player to a different world or dimension with an optional position, yaw, and pitch.
    pub async fn teleport_world(
        self: &Arc<Self>,
        new_world: Arc<World>,
        position: Vector3<f64>,
        yaw: Option<f32>,
        pitch: Option<f32>,
    ) {
        let current_world = self.living_entity.entity.world.clone();
        let yaw = yaw.unwrap_or(new_world.level_info.read().await.spawn_angle);
        let pitch = pitch.unwrap_or(10.0);

        send_cancellable! {{
            PlayerChangeWorldEvent {
                player: self.clone(),
                previous_world: current_world.clone(),
                new_world: new_world.clone(),
                position,
                yaw,
                pitch,
                cancelled: false,
            };

            'after: {
                // TODO: this is duplicate code from world
                let position = event.position;
                let yaw = event.yaw;
                let pitch = event.pitch;
                let new_world = event.new_world;

                self.set_client_loaded(false);
                let uuid = self.gameprofile.id;
                current_world.remove_player(self, false).await;
                new_world.players.write().await.insert(uuid, Arc::new(Self::new(
                            self.client.clone(),
                            self.gameprofile.clone(),
                            self.config.read().await.clone(),
                            new_world.clone(),
                            self.gamemode.load(),
                        )
                        .await));
                self.unload_watched_chunks(&current_world).await;

                self.chunk_manager.lock().await.change_world(&current_world.level, &new_world.level);

                let last_pos = self.living_entity.entity.last_pos.load();
                let death_dimension = self.world().dimension_type.resource_location();
                let death_location = BlockPos(Vector3::new(
                    last_pos.x.round() as i32,
                    last_pos.y.round() as i32,
                    last_pos.z.round() as i32,
                ));
                self.client
                    .send_packet_now(&CRespawn::new(
                        (new_world.dimension_type as u8).into(),
                        new_world.dimension_type.resource_location(),
                        biome::hash_seed(new_world.level.seed.0), // seed
                        self.gamemode.load() as u8,
                        self.gamemode.load() as i8,
                        false,
                        false,
                        Some((death_dimension, death_location)),
                        VarInt(self.get_entity().portal_cooldown.load(Ordering::Relaxed) as i32),
                        new_world.sea_level.into(),
                        1,
                    )).await
                    ;
                self.send_permission_lvl_update().await;
                self.clone().request_teleport(position, yaw, pitch).await;
                self.living_entity.entity.last_pos.store(position);
                self.send_abilities_update().await;
                self.enqueue_set_held_item_packet(&CSetSelectedSlot::new(
                   self.get_inventory().get_selected_slot() as i8,
                )).await;
                self.on_screen_handler_opened(self.player_screen_handler.clone()).await;
                self.send_health().await;

                new_world.send_world_info(self, position, yaw, pitch).await;
            }
        }}
    }

    /// `yaw` and `pitch` are in degrees.
    /// Rarly used, for example when waking up the player from a bed or their first time spawn. Otherwise, the `teleport` method should be used.
    /// The player should respond with the `SConfirmTeleport` packet.
    pub async fn request_teleport(self: &Arc<Self>, position: Vector3<f64>, yaw: f32, pitch: f32) {
        // This is the ultra special magic code used to create the teleport id
        // This returns the old value
        // This operation wraps around on overflow.

        send_cancellable! {{
            PlayerTeleportEvent {
                player: self.clone(),
                from: self.living_entity.entity.pos.load(),
                to: position,
                cancelled: false,
            };

            'after: {
                let position = event.to;
                let i = self
                    .teleport_id_count
                    .fetch_add(1, Ordering::Relaxed);
                let teleport_id = i + 1;
                self.living_entity.entity.set_pos(position);
                let entity = &self.living_entity.entity;
                entity.set_rotation(yaw, pitch);
                *self.awaiting_teleport.lock().await = Some((teleport_id.into(), position));
                self.client
                    .send_packet_now(&CPlayerPosition::new(
                        teleport_id.into(),
                        position,
                        Vector3::new(0.0, 0.0, 0.0),
                        yaw,
                        pitch,
                        // TODO
                        Vec::new(),
                    )).await;
            }
        }}
    }

    pub fn block_interaction_range(&self) -> f64 {
        if self.gamemode.load() == GameMode::Creative {
            5.0
        } else {
            4.5
        }
    }

    pub fn can_interact_with_block_at(&self, position: &BlockPos, additional_range: f64) -> bool {
        let d = self.block_interaction_range() + additional_range;
        let box_pos = BoundingBox::from_block(position);
        let entity_pos = self.living_entity.entity.pos.load();
        let standing_eye_height = self.living_entity.entity.standing_eye_height;
        box_pos.squared_magnitude(Vector3 {
            x: entity_pos.x,
            y: entity_pos.y + f64::from(standing_eye_height),
            z: entity_pos.z,
        }) < d * d
    }

    pub async fn kick(&self, reason: DisconnectReason, message: TextComponent) {
        self.client.kick(reason, message).await;
    }

    pub fn can_food_heal(&self) -> bool {
        let health = self.living_entity.health.load();
        let max_health = 20.0; // TODO
        health > 0.0 && health < max_health
    }

    pub async fn add_exhaustion(&self, exhaustion: f32) {
        if self.abilities.lock().await.invulnerable {
            return;
        }
        self.hunger_manager.add_exhaustion(exhaustion);
    }

    pub async fn heal(&self, additional_health: f32) {
        self.living_entity.heal(additional_health).await;
        self.send_health().await;
    }

    pub async fn send_health(&self) {
        self.client
            .enqueue_packet(&CSetHealth::new(
                self.living_entity.health.load(),
                self.hunger_manager.level.load().into(),
                self.hunger_manager.saturation.load(),
            ))
            .await;
    }

    pub async fn tick_health(&self) {
        let health = self.living_entity.health.load() as i32;
        let food = self.hunger_manager.level.load();
        let saturation = self.hunger_manager.saturation.load();

        let last_health = self.last_sent_health.load(Ordering::Relaxed);
        let last_food = self.last_sent_food.load(Ordering::Relaxed);
        let last_saturation = self.last_food_saturation.load(Ordering::Relaxed);

        if health != last_health || food != last_food || (saturation == 0.0) != last_saturation {
            self.last_sent_health.store(health, Ordering::Relaxed);
            self.last_sent_food.store(food, Ordering::Relaxed);
            self.last_food_saturation
                .store(saturation == 0.0, Ordering::Relaxed);
            self.send_health().await;
        }
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

    async fn handle_killed(&self, death_msg: TextComponent) {
        self.set_client_loaded(false);
        self.client
            .send_packet_now(&CCombatDeath::new(self.entity_id().into(), &death_msg))
            .await;
    }

    pub async fn set_gamemode(self: &Arc<Self>, gamemode: GameMode) {
        // We could send the same gamemode without any problems. But why waste bandwidth?
        assert_ne!(
            self.gamemode.load(),
            gamemode,
            "Attempt to set the gamemode to the already current gamemode"
        );
        send_cancellable! {{
            PlayerGamemodeChangeEvent {
                player: self.clone(),
                new_gamemode: gamemode,
                previous_gamemode: self.gamemode.load(),
                cancelled: false,
            };

            'after: {
                let gamemode = event.new_gamemode;
                self.gamemode.store(gamemode);
                // TODO: Fix this when mojang fixes it
                // This is intentional to keep the pure vanilla mojang experience
                // self.previous_gamemode.store(self.previous_gamemode.load());
                {
                    // Use another scope so that we instantly unlock `abilities`.
                    let mut abilities = self.abilities.lock().await;
                    abilities.set_for_gamemode(gamemode);
                };
                self.send_abilities_update().await;

                self.living_entity.entity.invulnerable.store(
                    matches!(gamemode, GameMode::Creative | GameMode::Spectator),
                    Ordering::Relaxed,
                );
                self.living_entity
                    .entity
                    .world
                    .broadcast_packet_all(&CPlayerInfoUpdate::new(
                        PlayerInfoFlags::UPDATE_GAME_MODE.bits(),
                        &[pumpkin_protocol::java::client::play::Player {
                            uuid: self.gameprofile.id,
                            actions: &[PlayerAction::UpdateGameMode((gamemode as i32).into())],
                        }],
                    ))
                    .await;

                self.client
                    .enqueue_packet(&CGameEvent::new(
                        GameEvent::ChangeGameMode,
                        gamemode as i32 as f32,
                    )).await;
            }
        }}
    }

    /// Send the player's skin layers and used hand to all players.
    pub fn send_client_information(&self) {
        //let config = self.config.read().await;
        // TODO
        // self.living_entity
        //     .entity
        //     .send_meta_data(&[
        //         Metadata::new(
        //             DATA_PLAYER_MODE_CUSTOMISATION,
        //             MetaDataType::Byte,
        //             config.skin_parts,
        //         ),
        //         Metadata::new(
        //             DATA_PLAYER_MAIN_HAND,
        //             MetaDataType::Byte,
        //             config.main_hand as u8,
        //         ),
        //     ])
        //     .await;
    }

    pub async fn can_harvest(&self, state: &BlockState, block: &'static Block) -> bool {
        !state.tool_required()
            || self
                .inventory
                .held_item()
                .lock()
                .await
                .is_correct_for_drops(block)
    }

    pub async fn get_mining_speed(&self, block: &'static Block) -> f32 {
        let mut speed = self.inventory.held_item().lock().await.get_speed(block);
        // Haste
        if self.living_entity.has_effect(&StatusEffect::HASTE).await
            || self
                .living_entity
                .has_effect(&StatusEffect::CONDUIT_POWER)
                .await
        {
            speed *= 1.0 + (self.get_haste_amplifier().await + 1) as f32 * 0.2;
        }
        // Fatigue
        if let Some(fatigue) = self
            .living_entity
            .get_effect(&StatusEffect::MINING_FATIGUE)
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
        // TODO: Handle when in water
        if !self.living_entity.entity.on_ground.load(Ordering::Relaxed) {
            speed /= 5.0;
        }
        speed
    }

    async fn get_haste_amplifier(&self) -> u32 {
        let mut i = 0;
        let mut j = 0;
        if let Some(effect) = self.living_entity.get_effect(&StatusEffect::HASTE).await {
            i = effect.amplifier;
        }
        if let Some(effect) = self
            .living_entity
            .get_effect(&StatusEffect::CONDUIT_POWER)
            .await
        {
            j = effect.amplifier;
        }
        u32::from(i.max(j))
    }

    pub async fn send_message(
        &self,
        message: &TextComponent,
        chat_type: u8,
        sender_name: &TextComponent,
        target_name: Option<&TextComponent>,
    ) {
        self.client
            .enqueue_packet(&CDisguisedChatMessage::new(
                message,
                (chat_type + 1).into(),
                sender_name,
                target_name,
            ))
            .await;
    }

    pub async fn drop_item(&self, item_stack: ItemStack) {
        let item_pos = self.living_entity.entity.pos.load()
            + Vector3::new(0.0, f64::from(EntityType::PLAYER.eye_height) - 0.3, 0.0);
        let entity = Entity::new(
            Uuid::new_v4(),
            self.world().clone(),
            item_pos,
            &EntityType::ITEM,
            false,
        );

        let pitch = f64::from(self.living_entity.entity.pitch.load()).to_radians();
        let yaw = f64::from(self.living_entity.entity.yaw.load()).to_radians();
        let pitch_sin = pitch.sin();
        let pitch_cos = pitch.cos();
        let yaw_sin = yaw.sin();
        let yaw_cos = yaw.cos();
        let horizontal_offset = rand::random::<f64>() * TAU;
        let l = 0.02 * rand::random::<f64>();

        let velocity = Vector3::new(
            -yaw_sin * pitch_cos * 0.3 + horizontal_offset.cos() * l,
            -pitch_sin * 0.3 + 0.1 + (rand::random::<f64>() - rand::random::<f64>()) * 0.1,
            yaw_cos * pitch_cos * 0.3 + horizontal_offset.sin() * l,
        );

        // TODO: Merge stacks together
        let item_entity =
            Arc::new(ItemEntity::new_with_velocity(entity, item_stack, velocity, 40).await);
        self.world().spawn_entity(item_entity).await;
    }

    pub async fn drop_held_item(&self, drop_stack: bool) {
        // should be locked first otherwise cause deadlock in tick() (this thread lock stack, that thread lock screen_handler)

        let binding = self.inventory.held_item();
        let mut item_stack = binding.lock().await;

        if !item_stack.is_empty() {
            let drop_amount = if drop_stack { item_stack.item_count } else { 1 };
            self.drop_item(item_stack.copy_with_count(drop_amount))
                .await;
            item_stack.decrement(drop_amount);
            let selected_slot = self.inventory.get_selected_slot();
            let inv: Arc<dyn Inventory> = self.inventory.clone();
            let screen_binding = self.current_screen_handler.lock().await;
            let mut screen_handler = screen_binding.lock().await;
            let slot_index = screen_handler
                .get_slot_index(&inv, selected_slot as usize)
                .await;

            if let Some(slot_index) = slot_index {
                screen_handler.set_received_stack(slot_index, item_stack.clone());
            }
        }
    }

    pub async fn swap_item(&self) {
        let (main_hand_item, off_hand_item) = self.inventory.swap_item().await;
        let equipment = &[
            (EquipmentSlot::MAIN_HAND, main_hand_item),
            (EquipmentSlot::OFF_HAND, off_hand_item),
        ];
        self.living_entity.send_equipment_changes(equipment).await;
        // todo this.player.stopUsingItem();
    }

    pub async fn send_system_message(&self, text: &TextComponent) {
        match &self.client {
            ClientPlatform::Java(client) => {
                client
                    .enqueue_packet(&CSystemChatMessage::new(text, false))
                    .await;
            }
            ClientPlatform::Bedrock(client) => {
                client
                    .send_game_packet(&SText::system_message(text.clone().get_text()))
                    .await;
            }
        }
    }

    pub async fn send_system_message_raw(&self, text: &TextComponent, overlay: bool) {
        match &self.client {
            ClientPlatform::Java(client) => {
                client
                    .enqueue_packet(&CSystemChatMessage::new(text, overlay))
                    .await;
            }
            ClientPlatform::Bedrock(client) => {
                client
                    .send_game_packet(&SText::system_message(text.clone().get_text()))
                    .await;
            }
        }
    }

    pub async fn tick_experience(&self) {
        let level = self.experience_level.load(Ordering::Relaxed);
        if self.last_sent_xp.load(Ordering::Relaxed) != level {
            let progress = self.experience_progress.load();
            let points = self.experience_points.load(Ordering::Relaxed);

            self.last_sent_xp.store(level, Ordering::Relaxed);

            self.client
                .send_packet_now(&CSetExperience::new(
                    progress.clamp(0.0, 1.0),
                    points.into(),
                    level.into(),
                ))
                .await;
        }
    }

    /// Sets the player's experience level and notifies the client.
    pub async fn set_experience(&self, level: i32, progress: f32, points: i32) {
        // TODO: These should be atomic together, not isolated; make a struct containing these. can cause ABA issues
        self.experience_level.store(level, Ordering::Relaxed);
        self.experience_progress.store(progress.clamp(0.0, 1.0));
        self.experience_points.store(points, Ordering::Relaxed);
        self.last_sent_xp.store(-1, Ordering::Relaxed);
        self.tick_experience().await;

        self.client
            .enqueue_packet(&CSetExperience::new(
                progress.clamp(0.0, 1.0),
                points.into(),
                level.into(),
            ))
            .await;
    }

    /// Sets the player's experience level directly.
    pub async fn set_experience_level(&self, new_level: i32, keep_progress: bool) {
        let progress = self.experience_progress.load();
        let mut points = self.experience_points.load(Ordering::Relaxed);

        // If `keep_progress` is `true` then calculate the number of points needed to keep the same progress scaled.
        if keep_progress {
            // Get our current level
            let current_level = self.experience_level.load(Ordering::Relaxed);
            let current_max_points = experience::points_in_level(current_level);
            // Calculate the max value for the new level
            let new_max_points = experience::points_in_level(new_level);
            // Calculate the scaling factor
            let scale = new_max_points as f32 / current_max_points as f32;
            // Scale the points (Vanilla doesn't seem to recalculate progress so we won't)
            points = (points as f32 * scale) as i32;
        }

        self.set_experience(new_level, progress, points).await;
    }

    pub async fn add_effect(&self, effect: Effect) {
        self.send_effect(effect.clone()).await;
        self.living_entity.add_effect(effect).await;
    }

    pub async fn send_active_effects(&self) {
        let effects = self.living_entity.active_effects.lock().await;
        for effect in effects.values() {
            self.send_effect(effect.clone()).await;
        }
    }

    pub async fn send_effect(&self, effect: Effect) {
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
        if effect.blend {
            flag |= 8;
        }

        let effect_id = VarInt(i32::from(effect.effect_type.id));
        self.client
            .enqueue_packet(&CUpdateMobEffect::new(
                self.entity_id().into(),
                effect_id,
                effect.amplifier.into(),
                effect.duration.into(),
                flag,
            ))
            .await;
    }

    pub async fn remove_effect(&self, effect_type: &'static StatusEffect) {
        let effect_id = VarInt(i32::from(effect_type.id));
        self.client
            .enqueue_packet(
                &pumpkin_protocol::java::client::play::CRemoveMobEffect::new(
                    self.entity_id().into(),
                    effect_id,
                ),
            )
            .await;
        self.living_entity.remove_effect(effect_type).await;

        // TODO broadcast metadata
    }

    pub async fn remove_all_effect(&self) -> u8 {
        let mut count = 0;
        let mut effect_list = vec![];
        for effect in self.living_entity.active_effects.lock().await.keys() {
            effect_list.push(*effect);
            let effect_id = VarInt(i32::from(effect.id));
            self.client
                .enqueue_packet(
                    &pumpkin_protocol::java::client::play::CRemoveMobEffect::new(
                        self.entity_id().into(),
                        effect_id,
                    ),
                )
                .await;
            count += 1;
        }
        //Need to remove effect after because the player effect are lock in the for before
        for effect in effect_list {
            self.living_entity.remove_effect(effect).await;
        }

        count
    }

    /// Add experience levels to the player.
    pub async fn add_experience_levels(&self, added_levels: i32) {
        let current_level = self.experience_level.load(Ordering::Relaxed);
        let new_level = current_level + added_levels;
        self.set_experience_level(new_level, true).await;
    }

    /// Set the player's experience points directly. Returns `true` if successful.
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

    /// Add experience points to the player.
    pub async fn add_experience_points(&self, added_points: i32) {
        let current_level = self.experience_level.load(Ordering::Relaxed);
        let current_points = self.experience_points.load(Ordering::Relaxed);
        let total_exp = experience::points_to_level(current_level) + current_points;
        let new_total_exp = total_exp + added_points;
        let (new_level, new_points) = experience::total_to_level_and_points(new_total_exp);
        let progress = experience::progress_in_level(new_points, new_level);
        self.set_experience(new_level, progress, new_points).await;
    }

    pub fn increment_screen_handler_sync_id(&self) {
        let current_id = self.screen_handler_sync_id.load(Ordering::Relaxed);
        self.screen_handler_sync_id
            .store(current_id % 100 + 1, Ordering::Relaxed);
    }

    pub async fn close_handled_screen(&self) {
        self.client
            .enqueue_packet(&CCloseContainer::new(
                self.current_screen_handler
                    .lock()
                    .await
                    .lock()
                    .await
                    .sync_id()
                    .into(),
            ))
            .await;
        self.on_handled_screen_closed().await;
    }

    pub async fn on_handled_screen_closed(&self) {
        self.current_screen_handler
            .lock()
            .await
            .lock()
            .await
            .on_closed(self)
            .await;

        let player_screen_handler: Arc<Mutex<dyn ScreenHandler>> =
            self.player_screen_handler.clone();
        let current_screen_handler: Arc<Mutex<dyn ScreenHandler>> =
            self.current_screen_handler.lock().await.clone();

        if !Arc::ptr_eq(&player_screen_handler, &current_screen_handler) {
            player_screen_handler
                .lock()
                .await
                .copy_shared_slots(current_screen_handler)
                .await;
        }

        *self.current_screen_handler.lock().await = self.player_screen_handler.clone();
    }

    pub async fn on_screen_handler_opened(&self, screen_handler: Arc<Mutex<dyn ScreenHandler>>) {
        let mut screen_handler = screen_handler.lock().await;

        screen_handler
            .add_listener(self.screen_handler_listener.clone())
            .await;

        screen_handler
            .update_sync_handler(self.screen_handler_sync_handler.clone())
            .await;
    }

    pub async fn open_handled_screen(
        &self,
        screen_handler_factory: &dyn ScreenHandlerFactory,
    ) -> Option<u8> {
        if !self
            .current_screen_handler
            .lock()
            .await
            .lock()
            .await
            .as_any()
            .is::<PlayerScreenHandler>()
        {
            self.close_handled_screen().await;
        }

        self.increment_screen_handler_sync_id();

        if let Some(screen_handler) = screen_handler_factory
            .create_screen_handler(
                self.screen_handler_sync_id.load(Ordering::Relaxed),
                &self.inventory,
                self,
            )
            .await
        {
            let screen_handler_temp = screen_handler.lock().await;
            self.client
                .enqueue_packet(&COpenScreen::new(
                    screen_handler_temp.sync_id().into(),
                    (screen_handler_temp
                        .window_type()
                        .expect("Can't open PlayerScreenHandler") as i32)
                        .into(),
                    &screen_handler_factory.get_display_name(),
                ))
                .await;
            drop(screen_handler_temp);
            self.on_screen_handler_opened(screen_handler.clone()).await;
            *self.current_screen_handler.lock().await = screen_handler;
            Some(self.screen_handler_sync_id.load(Ordering::Relaxed))
        } else {
            //TODO: Send message if spectator

            None
        }
    }

    pub async fn on_slot_click(&self, packet: SClickSlot) {
        let screen_handler = self.current_screen_handler.lock().await;
        let mut screen_handler = screen_handler.lock().await;
        let behaviour = screen_handler.get_behaviour();

        // behaviour is dropped here
        if i32::from(behaviour.sync_id) != packet.sync_id.0 {
            return;
        }

        if self.gamemode.load() == GameMode::Spectator {
            screen_handler.sync_state().await;
            return;
        }

        if !screen_handler.can_use(self) {
            warn!(
                "Player {} interacted with invalid menu {:?}",
                self.gameprofile.name,
                screen_handler.window_type()
            );
            return;
        }

        let slot = packet.slot;

        if !screen_handler.is_slot_valid(i32::from(slot)).await {
            warn!(
                "Player {} clicked invalid slot index: {}, available slots: {}",
                self.gameprofile.name,
                slot,
                screen_handler.get_behaviour().slots.len()
            );
            return;
        }

        let not_in_sync = packet.revision.0 != (behaviour.revision.load(Ordering::Relaxed) as i32);

        screen_handler.disable_sync();
        screen_handler
            .on_slot_click(
                i32::from(slot),
                i32::from(packet.button),
                packet.mode.clone(),
                self,
            )
            .await;

        for (key, value) in packet.array_of_changed_slots {
            screen_handler.set_received_hash(key as usize, value);
        }

        screen_handler.set_received_cursor_hash(packet.carried_item);
        screen_handler.enable_sync();

        if not_in_sync {
            screen_handler.update_to_client().await;
        } else {
            screen_handler.send_content_updates().await;
            drop(screen_handler);
        }
    }

    /// Check if the player has a specific permission
    pub async fn has_permission(&self, node: &str) -> bool {
        let perm_manager = PERMISSION_MANAGER.read().await;
        perm_manager
            .has_permission(&self.gameprofile.id, node, self.permission_lvl.load())
            .await
    }

    pub fn is_creative(&self) -> bool {
        self.gamemode.load() == GameMode::Creative
    }

    /// Swing the hand of the player
    pub async fn swing_hand(&self, hand: Hand, all: bool) {
        let world = self.world();
        let entity_id = VarInt(self.entity_id());

        let animation = match hand {
            Hand::Left => Animation::SwingMainArm,
            Hand::Right => Animation::SwingOffhand,
        };

        let packet = CEntityAnimation::new(entity_id, animation);
        if all {
            world.broadcast_packet_all(&packet).await;
        } else {
            world
                .broadcast_packet_except(&[self.gameprofile.id], &packet)
                .await;
        }
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.gameprofile.id == other.gameprofile.id
    }
}

impl NBTStorage for Player {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            nbt.put_int("DataVersion", DATA_VERSION);
            self.living_entity.write_nbt(nbt).await;
            self.inventory.write_nbt(nbt).await;
            self.ender_chest_inventory.write_nbt(nbt).await;

            self.abilities.lock().await.write_nbt(nbt).await;

            // Store total XP instead of individual components
            let total_exp =
                experience::points_to_level(self.experience_level.load(Ordering::Relaxed))
                    + self.experience_points.load(Ordering::Relaxed);
            nbt.put_int("XpTotal", total_exp);
            nbt.put_byte("playerGameType", self.gamemode.load() as i8);
            if let Some(previous_gamemode) = self.previous_gamemode.load() {
                nbt.put_byte("previousPlayerGameType", previous_gamemode as i8);
            }

            nbt.put_bool(
                "HasPlayedBefore",
                self.has_played_before.load(Ordering::Relaxed),
            );

            // Store food level, saturation, exhaustion, and tick timer
            self.hunger_manager.write_nbt(nbt).await;

            nbt.put_string(
                "Dimension",
                self.world().dimension_type.resource_location().to_string(),
            );
        })
    }

    fn read_nbt<'a>(&'a mut self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.living_entity.read_nbt(nbt).await;
            self.inventory.read_nbt_non_mut(nbt).await;
            self.ender_chest_inventory.read_nbt_non_mut(nbt).await;
            self.abilities.lock().await.read_nbt(nbt).await;

            self.gamemode.store(
                GameMode::try_from(nbt.get_byte("playerGameType").unwrap_or(0))
                    .unwrap_or(GameMode::Survival),
            );

            self.previous_gamemode.store(
                nbt.get_byte("previousPlayerGameType")
                    .and_then(|byte| GameMode::try_from(byte).ok()),
            );

            self.has_played_before.store(
                nbt.get_bool("HasPlayedBefore").unwrap_or(false),
                Ordering::Relaxed,
            );

            // Load food level, saturation, exhaustion, and tick timer
            self.hunger_manager.read_nbt(nbt).await;

            // Load from total XP
            let total_exp = nbt.get_int("XpTotal").unwrap_or(0);
            let (level, points) = experience::total_to_level_and_points(total_exp);
            let progress = experience::progress_in_level(level, points);
            self.experience_level.store(level, Ordering::Relaxed);
            self.experience_progress.store(progress);
            self.experience_points.store(points, Ordering::Relaxed);
        })
    }
}

impl NBTStorageInit for Player {}

impl NBTStorage for PlayerInventory {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            // Save the selected slot (hotbar)
            nbt.put_int("SelectedItemSlot", i32::from(self.get_selected_slot()));

            // Create inventory list with the correct capacity (inventory size)
            let mut items: Vec<NbtTag> = Vec::with_capacity(41);
            for (i, item) in self.main_inventory.iter().enumerate() {
                let stack = item.lock().await;
                if !stack.is_empty() {
                    let mut item_compound = NbtCompound::new();
                    item_compound.put_byte("Slot", i as i8);
                    stack.write_item_stack(&mut item_compound);
                    drop(stack);
                    items.push(NbtTag::Compound(item_compound));
                }
            }

            let mut equipment_compound = NbtCompound::new();
            for slot in self.equipment_slots.values() {
                let stack_binding = self.entity_equipment.lock().await.get(slot);
                let stack = stack_binding.lock().await;
                if !stack.is_empty() {
                    let mut item_compound = NbtCompound::new();
                    stack.write_item_stack(&mut item_compound);
                    drop(stack);
                    match slot {
                        EquipmentSlot::OffHand(_) => {
                            equipment_compound.put_component("offhand", item_compound);
                        }
                        EquipmentSlot::Feet(_) => {
                            equipment_compound.put_component("feet", item_compound);
                        }
                        EquipmentSlot::Legs(_) => {
                            equipment_compound.put_component("legs", item_compound);
                        }
                        EquipmentSlot::Chest(_) => {
                            equipment_compound.put_component("chest", item_compound);
                        }
                        EquipmentSlot::Head(_) => {
                            equipment_compound.put_component("head", item_compound);
                        }
                        _ => {
                            warn!("Invalid equipment slot for a player");
                        }
                    }
                }
            }
            nbt.put_component("equipment", equipment_compound);
            nbt.put("Inventory", NbtTag::List(items));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            // Read selected hotbar slot
            self.set_selected_slot(nbt.get_int("SelectedItemSlot").unwrap_or(0) as u8);
            // Process inventory list
            if let Some(inventory_list) = nbt.get_list("Inventory") {
                for tag in inventory_list {
                    if let Some(item_compound) = tag.extract_compound()
                        && let Some(slot_byte) = item_compound.get_byte("Slot")
                    {
                        let slot = slot_byte as usize;
                        if let Some(item_stack) = ItemStack::read_item_stack(item_compound) {
                            self.set_stack(slot, item_stack).await;
                        }
                    }
                }
            }

            if let Some(equipment) = nbt.get_compound("equipment") {
                if let Some(offhand) = equipment.get_compound("offhand")
                    && let Some(item_stack) = ItemStack::read_item_stack(offhand)
                {
                    self.set_stack(40, item_stack).await;
                }

                if let Some(head) = equipment.get_compound("head")
                    && let Some(item_stack) = ItemStack::read_item_stack(head)
                {
                    self.set_stack(39, item_stack).await;
                }

                if let Some(chest) = equipment.get_compound("chest")
                    && let Some(item_stack) = ItemStack::read_item_stack(chest)
                {
                    self.set_stack(38, item_stack).await;
                }

                if let Some(legs) = equipment.get_compound("legs")
                    && let Some(item_stack) = ItemStack::read_item_stack(legs)
                {
                    self.set_stack(37, item_stack).await;
                }

                if let Some(feet) = equipment.get_compound("feet")
                    && let Some(item_stack) = ItemStack::read_item_stack(feet)
                {
                    self.set_stack(36, item_stack).await;
                }
            }
        })
    }
}

impl NBTStorageInit for PlayerInventory {}

impl NBTStorage for EnderChestInventory {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            // Create item list with the correct capacity (inventory size)
            let mut items: Vec<NbtTag> = Vec::with_capacity(Self::INVENTORY_SIZE);
            for (i, item) in self.items.iter().enumerate() {
                let stack = item.lock().await;
                if !stack.is_empty() {
                    let mut item_compound = NbtCompound::new();
                    item_compound.put_byte("Slot", i as i8);
                    stack.write_item_stack(&mut item_compound);
                    drop(stack);
                    items.push(NbtTag::Compound(item_compound));
                }
            }

            nbt.put("EnderItems", NbtTag::List(items));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            // Process item list
            if let Some(item_list) = nbt.get_list("EnderItems") {
                for tag in item_list {
                    if let Some(item_compound) = tag.extract_compound()
                        && let Some(slot_byte) = item_compound.get_byte("Slot")
                    {
                        let slot = slot_byte as usize;
                        if let Some(item_stack) = ItemStack::read_item_stack(item_compound) {
                            self.set_stack(slot, item_stack).await;
                        }
                    }
                }
            }
        })
    }
}

impl NBTStorageInit for EnderChestInventory {}

impl EntityBase for Player {
    fn damage_with_context<'a>(
        &'a self,
        caller: Arc<dyn EntityBase>,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if self.abilities.lock().await.invulnerable && damage_type != DamageType::GENERIC_KILL {
                return false;
            }
            let dyn_self = self
                .living_entity
                .entity
                .world
                .get_entity_by_id(self.living_entity.entity.entity_id)
                .await
                .expect("Entity not found in world");
            let result = self
                .living_entity
                .damage_with_context(caller, amount, damage_type, position, source, cause)
                .await;
            if result {
                let health = self.living_entity.health.load();
                if health <= 0.0 {
                    let death_message =
                        LivingEntity::get_death_message(&*dyn_self, damage_type, source, cause)
                            .await;
                    self.handle_killed(death_message).await;
                }
            }
            result
        })
    }

    fn teleport(
        self: Arc<Self>,
        position: Vector3<f64>,
        yaw: Option<f32>,
        pitch: Option<f32>,
        world: Arc<World>,
    ) -> TeleportFuture {
        Box::pin(async move {
            if Arc::ptr_eq(&world, self.world()) {
                // Same world
                let yaw = yaw.unwrap_or(self.living_entity.entity.yaw.load());
                let pitch = pitch.unwrap_or(self.living_entity.entity.pitch.load());
                send_cancellable! {{
                    PlayerTeleportEvent {
                        player: self.clone(),
                        from: self.living_entity.entity.pos.load(),
                        to: position,
                        cancelled: false,
                    };
                    'after: {
                        let position = event.to;
                        let entity = self.get_entity();
                        self.request_teleport(position, yaw, pitch).await;
                        entity
                            .world
                            .broadcast_packet_except(&[self.gameprofile.id], &CEntityPositionSync::new(
                                self.living_entity.entity.entity_id.into(),
                                position,
                                Vector3::new(0.0, 0.0, 0.0),
                                yaw,
                                pitch,
                                entity.on_ground.load(Ordering::SeqCst),
                            ))
                            .await;
                    }
                }}
            } else {
                self.teleport_world(world, position, yaw, pitch).await;
            }
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.living_entity.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        Some(&self.living_entity)
    }

    fn get_player(&self) -> Option<&Player> {
        Some(self)
    }

    fn is_spectator(&self) -> bool {
        self.gamemode.load() == GameMode::Spectator
    }

    fn get_name(&self) -> TextComponent {
        //TODO: team color
        TextComponent::text(self.gameprofile.name.clone())
    }

    fn get_display_name(&self) -> EntityBaseFuture<'_, TextComponent> {
        let name = self.get_name();
        let name_clone = name.clone();
        let mut name = name.click_event(ClickEvent::SuggestCommand {
            command: format!("/tell {} ", self.gameprofile.name.clone()).into(),
        });
        name = name.hover_event(HoverEvent::show_entity(
            self.living_entity.entity.entity_uuid.to_string(),
            self.living_entity.entity.entity_type.resource_name.into(),
            Some(name_clone),
        ));
        Box::pin(async move { name.insertion(self.gameprofile.name.clone()) })
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
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

impl NBTStorage for Abilities {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            let mut component = NbtCompound::new();
            component.put_bool("invulnerable", self.invulnerable);
            component.put_bool("flying", self.flying);
            component.put_bool("mayfly", self.allow_flying);
            component.put_bool("instabuild", self.creative);
            component.put_bool("mayBuild", self.allow_modify_world);
            component.put_float("flySpeed", self.fly_speed);
            component.put_float("walkSpeed", self.walk_speed);
            nbt.put_component("abilities", component);
        })
    }

    fn read_nbt<'a>(&'a mut self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            if let Some(component) = nbt.get_compound("abilities") {
                self.invulnerable = component.get_bool("invulnerable").unwrap_or(false);
                self.flying = component.get_bool("flying").unwrap_or(false);
                self.allow_flying = component.get_bool("mayfly").unwrap_or(false);
                self.creative = component.get_bool("instabuild").unwrap_or(false);
                self.allow_modify_world = component.get_bool("mayBuild").unwrap_or(false);
                self.fly_speed = component.get_float("flySpeed").unwrap_or(0.05);
                self.walk_speed = component.get_float("walkSpeed").unwrap_or(0.1);
            }
        })
    }
}

impl NBTStorageInit for Abilities {}

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

/// Represents the player's respawn point.
#[derive(Copy, Debug, Clone, PartialEq)]
pub struct RespawnPoint {
    pub dimension: VanillaDimensionType,
    pub position: BlockPos,
    pub yaw: f32,
    pub force: bool,
}

/// Represents the player's chat mode settings.
#[derive(Debug, Clone)]
pub enum ChatMode {
    /// Chat is enabled for the player.
    Enabled,
    /// The player should only see chat messages from commands.
    CommandsOnly,
    /// All messages should be hidden.
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

/// Player's current chat session
pub struct ChatSession {
    pub session_id: uuid::Uuid,
    pub expires_at: i64,
    pub public_key: Box<[u8]>,
    pub signature: Box<[u8]>,
    pub messages_sent: i32,
    pub messages_received: i32,
    pub signature_cache: Vec<Box<[u8]>>,
}

impl Default for ChatSession {
    fn default() -> Self {
        Self::new(Uuid::nil(), 0, Box::new([]), Box::new([]))
    }
}

impl ChatSession {
    #[must_use]
    pub fn new(
        session_id: Uuid,
        expires_at: i64,
        public_key: Box<[u8]>,
        key_signature: Box<[u8]>,
    ) -> Self {
        Self {
            session_id,
            expires_at,
            public_key,
            signature: key_signature,
            messages_sent: 0,
            messages_received: 0,
            signature_cache: Vec::new(),
        }
    }
}

#[derive(Clone, Default)]
pub struct LastSeen(Vec<Box<[u8]>>);

impl From<LastSeen> for Vec<Box<[u8]>> {
    fn from(seen: LastSeen) -> Self {
        seen.0
    }
}

impl AsRef<[Box<[u8]>]> for LastSeen {
    fn as_ref(&self) -> &[Box<[u8]>] {
        &self.0
    }
}

impl LastSeen {
    /// The sender's `last_seen` signatures are sent as ID's if the recipient has them in their cache.
    /// Otherwise, the full signature is sent. (ID:0 indicates full signature is being sent)
    pub async fn indexed_for(&self, recipient: &Arc<Player>) -> Box<[PreviousMessage]> {
        let mut indexed = Vec::new();
        for signature in &self.0 {
            let index = recipient
                .signature_cache
                .lock()
                .await
                .full_cache
                .iter()
                .position(|s| s == signature);
            if let Some(index) = index {
                indexed.push(PreviousMessage {
                    // Send ID reference to recipient's cache (index + 1 because 0 is reserved for full signature)
                    id: VarInt(1 + index as i32),
                    signature: None,
                });
            } else {
                indexed.push(PreviousMessage {
                    // Send ID as 0 for full signature
                    id: VarInt(0),
                    signature: Some(signature.clone()),
                });
            }
        }
        indexed.into_boxed_slice()
    }
}

pub struct MessageCache {
    /// max 128 cached message signatures. Most recent FIRST.
    /// Server should (when possible) reference indexes in this (recipient's) cache instead of sending full signatures in last seen.
    /// Must be 1:1 with client's signature cache.
    full_cache: VecDeque<Box<[u8]>>,
    /// max 20 last seen messages by the sender. Most Recent LAST
    pub last_seen: LastSeen,
}

impl Default for MessageCache {
    fn default() -> Self {
        Self {
            full_cache: VecDeque::with_capacity(MAX_CACHED_SIGNATURES as usize),
            last_seen: LastSeen::default(),
        }
    }
}

impl MessageCache {
    /// Not used for caching seen messages. Only for non-indexed signatures from senders.
    pub fn cache_signatures(&mut self, signatures: &[Box<[u8]>]) {
        for sig in signatures.iter().rev() {
            if self.full_cache.contains(sig) {
                continue;
            }
            // If the cache is maxed, and someone sends a signature older than the oldest in cache, ignore it
            if self.full_cache.len() < MAX_CACHED_SIGNATURES as usize {
                self.full_cache.push_back(sig.clone()); // Recipient never saw this message so it must be older than the oldest in cache
            }
        }
    }

    /// Adds a seen signature to `last_seen` and `full_cache`.
    pub fn add_seen_signature(&mut self, signature: &[u8]) {
        if self.last_seen.0.len() >= MAX_PREVIOUS_MESSAGES as usize {
            self.last_seen.0.remove(0);
        }
        self.last_seen.0.push(signature.into());
        // This probably doesn't need to be a loop, but better safe than sorry
        while self.full_cache.len() >= MAX_CACHED_SIGNATURES as usize {
            self.full_cache.pop_back();
        }
        self.full_cache.push_front(signature.into()); // Since recipient saw this message it will be most recent in cache
    }
}

impl InventoryPlayer for Player {
    fn drop_item(&self, item: ItemStack, _retain_ownership: bool) -> PlayerFuture<'_, ()> {
        Box::pin(async move {
            self.drop_item(item).await;
        })
    }

    // Synchronous methods remain unchanged
    fn has_infinite_materials(&self) -> bool {
        self.gamemode.load() == GameMode::Creative
    }

    fn get_inventory(&self) -> Arc<PlayerInventory> {
        self.inventory.clone()
    }

    fn enqueue_inventory_packet<'a>(
        &'a self,
        packet: &'a CSetContainerContent,
    ) -> PlayerFuture<'a, ()> {
        Box::pin(async move {
            self.client.enqueue_packet(packet).await;
        })
    }

    fn enqueue_slot_packet<'a>(&'a self, packet: &'a CSetContainerSlot) -> PlayerFuture<'a, ()> {
        Box::pin(async move {
            self.client.enqueue_packet(packet).await;
        })
    }

    fn enqueue_cursor_packet<'a>(&'a self, packet: &'a CSetCursorItem) -> PlayerFuture<'a, ()> {
        Box::pin(async move {
            self.client.enqueue_packet(packet).await;
        })
    }

    fn enqueue_property_packet<'a>(
        &'a self,
        packet: &'a CSetContainerProperty,
    ) -> PlayerFuture<'a, ()> {
        Box::pin(async move {
            self.client.enqueue_packet(packet).await;
        })
    }

    fn enqueue_slot_set_packet<'a>(
        &'a self,
        packet: &'a CSetPlayerInventory,
    ) -> PlayerFuture<'a, ()> {
        Box::pin(async move {
            self.client.enqueue_packet(packet).await;
        })
    }

    fn enqueue_set_held_item_packet<'a>(
        &'a self,
        packet: &'a CSetSelectedSlot,
    ) -> PlayerFuture<'a, ()> {
        Box::pin(async move {
            self.client.enqueue_packet(packet).await;
        })
    }

    fn enqueue_equipment_change<'a>(
        &'a self,
        slot: &'a EquipmentSlot,
        stack: &'a ItemStack,
    ) -> PlayerFuture<'a, ()> {
        Box::pin(async move {
            self.world()
                .broadcast_packet_except(
                    &[self.get_entity().entity_uuid],
                    &CSetEquipment::new(
                        self.entity_id().into(),
                        vec![(
                            slot.discriminant(),
                            ItemStackSerializer::from(stack.clone()),
                        )],
                    ),
                )
                .await;

            if let Some(equippable) = stack.get_data_component::<EquippableImpl>()
                && let Some(sound) = Sound::from_name(
                    equippable
                        .equip_sound
                        .strip_prefix("minecraft:")
                        .unwrap_or(equippable.equip_sound),
                )
            {
                self.world()
                    .play_sound(sound, SoundCategory::Players, &self.position())
                    .await;
            }
        })
    }
}
