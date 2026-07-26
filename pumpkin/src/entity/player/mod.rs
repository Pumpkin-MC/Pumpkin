pub mod advancement;
pub mod statistics;

mod abilities;
mod chat;
mod chunk_manager;
mod combat;
mod display;
mod experience;
mod health;
mod inventory;
mod movement;
mod nbt;
mod network;
mod respawn;
mod screen;
mod tick;

pub use abilities::Abilities;
pub use chat::{ChatMode, ChatSession, InvalidChatMode, LastSeen, MessageCache};
pub use chunk_manager::ChunkManager;
pub use display::TitleMode;
pub use respawn::{CalculatedRespawnPoint, RespawnPoint};

use super::Entity;
use super::EntityBase;
use super::NBTStorage;
use super::breath::BreathManager;
use super::hunger::HungerManager;
use super::living::LivingEntity;
use crate::command::CommandSender;
use crate::command::context::command_source::CommandSource;
use crate::entity::EntityBaseFuture;
use crate::entity::TeleportFuture;
use crate::net::ClientPlatform;
use crate::net::GameProfile;
use crate::net::PlayerConfig;
use crate::plugin::player::player_teleport::PlayerTeleportEvent;
use crate::server::Server;
use crate::world::World;
use advancement::PlayerAdvancement;
use arc_swap::ArcSwap;
use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_inventory::player::ender_chest_inventory::EnderChestInventory;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::player::player_screen_handler::PlayerScreenHandler;
use pumpkin_inventory::screen_handler::ScreenHandler;
use pumpkin_inventory::screen_handler::ScreenHandlerListener;
use pumpkin_inventory::sync_handler::SyncHandler;
use pumpkin_macros::send_cancellable;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CCombatDeath;
use pumpkin_protocol::java::client::play::CEntityPositionSync;
use pumpkin_util::GameMode;
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::permission::PermissionLvl;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::click::ClickEvent;
use pumpkin_util::text::hover::HoverEvent;
use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;
use std::collections::HashMap;
use std::mem;
use std::num::NonZeroU8;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicI8;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::AtomicU8;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::debug;

pub const DATA_VERSION: i32 = 4903; // 26.2

/// Represents a Minecraft player entity.
///
/// A `Player` is a special type of entity that represents a human player connected to the server.
#[derive(Clone, Copy, Debug)]
pub struct ItemCooldown {
    pub start_tick: i32,
    pub duration: i32,
}

pub struct Player {
    /// The underlying living entity object that represents the player.
    pub living_entity: LivingEntity,
    /// The player's game profile information, including their username and UUID.
    pub gameprofile: GameProfile,
    /// The client connection associated with the player.
    pub client: Arc<ClientPlatform>,
    /// The player's inventory.
    pub inventory: Arc<PlayerInventory>,
    /// The player's `EnderChest` inventory.
    pub ender_chest_inventory: Arc<EnderChestInventory>,
    /// The player's configuration settings. Changes when the player changes their settings.
    pub config: ArcSwap<PlayerConfig>,
    /// The player's current gamemode (e.g., Survival, Creative, Adventure).
    pub gamemode: AtomicCell<GameMode>,
    /// The player's previous gamemode
    pub previous_gamemode: AtomicCell<Option<GameMode>>,
    /// The entity ID of the entity that the player is currently spectating/camera targeting.
    pub camera_target_id: AtomicCell<Option<i32>>,
    /// The player's spawnpoint
    pub respawn_point: Mutex<Option<RespawnPoint>>,
    /// The player's sleep status
    pub sleeping_since: AtomicCell<Option<u8>>,
    /// Manages the player's breath level
    pub breath_manager: BreathManager,
    /// Manages the player's hunger level.
    pub hunger_manager: HungerManager,
    /// The ID of the currently open container (if any).
    pub open_container: AtomicCell<Option<u64>>,
    /// The block position of the currently open container screen (if any).
    pub open_container_pos: AtomicCell<Option<BlockPos>>,
    /// The item currently being held by the player.
    pub carried_item: Mutex<Option<ItemStack>>,
    /// The player's abilities and special powers.
    ///
    /// This field represents the various abilities that the player possesses, such as flight, invulnerability, and other special effects.
    ///
    /// **Note:** When the `abilities` field is updated, the server should send a `send_abilities_update` packet to the client to notify them of the changes.
    pub abilities: Mutex<Abilities>,
    /// Player statistics
    pub stats: Mutex<statistics::Statistics>,
    /// The current stage of block destruction of the block the player is breaking.
    pub current_block_destroy_stage: AtomicI32,
    /// Indicates if the player is currently mining a block.
    pub mining: AtomicBool,
    pub start_mining_time: AtomicI32,
    pub tick_counter: AtomicI32,
    pub mining_pos: Mutex<BlockPos>,
    pub last_input: AtomicI8,
    /// A counter for teleport IDs used to track pending teleports.
    pub teleport_id_count: AtomicI32,
    /// The pending teleport information, including the teleport ID and target location.
    pub awaiting_teleport: Mutex<Option<(VarInt, Vector3<f64>)>>,
    /// The coordinates of the chunk section the player is currently watching.
    pub watched_section: AtomicCell<Cylindrical>,
    /// The last time the player performed an action (for idle timeout).
    pub last_action_time: AtomicCell<Instant>,
    /// The ping in millis.
    pub ping: AtomicU32,
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
    pub bedrock_spawned: AtomicBool,
    /// The amount of time (in ticks) the client has to report having finished loading before being timed out.
    pub client_loaded_timeout: AtomicU32,
    /// Item usage tracking for bows, crossbows, etc.
    pub using_item: AtomicBool,
    pub item_use_start_time: AtomicI32,
    pub using_hand: AtomicCell<Option<Hand>>,
    /// The player's experience level.
    pub experience_level: AtomicI32,
    /// The player's experience progress (`0.0` to `1.0`)
    pub experience_progress: AtomicCell<f32>,
    /// The player's total experience points.
    pub experience_points: AtomicI32,
    pub item_cooldowns: Mutex<HashMap<String, ItemCooldown>>,
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
    pub tab_list_header: Mutex<TextComponent>,
    pub tab_list_footer: Mutex<TextComponent>,
    pub display_name: Mutex<Option<TextComponent>>,
    pub tab_list_name: Mutex<Option<TextComponent>>,
    pub tab_list_order: AtomicI32,
    pub tab_list_latency: AtomicI32,
    pub tab_list_listed: AtomicBool,
    pub advancements: Arc<Mutex<PlayerAdvancement>>,
    pub enchantment_seed: AtomicI32,
    pub fishing_bobber: AtomicI32,
    pub bedrock_skin: arc_swap::ArcSwap<pumpkin_protocol::bedrock::client::Skin>,
}

use base64::prelude::*;
use pumpkin_protocol::Property;
use serde::Deserialize;
use std::io::Read;

#[derive(Deserialize)]
struct TexturesProperty {
    textures: Textures,
}

#[derive(Deserialize)]
struct Textures {
    #[serde(rename = "SKIN")]
    skin: Option<SkinTexture>,
}

#[derive(Deserialize)]
struct SkinTexture {
    url: String,
}

impl Player {
    #[must_use]
    pub fn fetch_skin(properties: &[Property]) -> Option<pumpkin_protocol::bedrock::client::Skin> {
        let textures_prop = properties.iter().find(|p| &*p.name == "textures")?;
        let decoded = BASE64_STANDARD
            .decode(textures_prop.value.as_bytes())
            .ok()?;
        let textures: TexturesProperty = serde_json::from_slice(&decoded).ok()?;
        let url = textures.textures.skin?.url;

        let resp = ureq::get(&url).call().ok()?;
        let mut buf = Vec::new();
        resp.into_body().into_reader().read_to_end(&mut buf).ok()?;
        let img = image::load_from_memory(&buf).ok()?;

        let width = img.width();
        let mut height = img.height();

        if width != 64 || (height != 32 && height != 64) {
            return None;
        }

        let mut rgba = img.into_rgba8().into_raw();

        if height == 32 {
            rgba.resize(64 * 64 * 4, 0);
            height = 64;
        }

        let mut skin = pumpkin_protocol::bedrock::client::Skin::steve();
        skin.image_width = width;
        skin.image_height = height;
        skin.skin_data = rgba;
        Some(skin)
    }

    #[expect(clippy::too_many_lines)]
    pub async fn new(
        client: Arc<ClientPlatform>,
        gameprofile: GameProfile,
        config: PlayerConfig,
        world: Arc<World>,
        gamemode: GameMode,
    ) -> Self {
        struct ScreenListener;

        impl ScreenHandlerListener for ScreenListener {}

        let server = world.server.upgrade().unwrap();

        let player_uuid = gameprofile.id;

        let living_entity = LivingEntity::new(Entity::from_uuid(
            player_uuid,
            world.clone(),
            Vector3::new(0.0, 100.0, 0.0),
            &EntityType::PLAYER,
        ));
        living_entity.entity.invulnerable.store(
            matches!(gamemode, GameMode::Creative | GameMode::Spectator),
            Ordering::Relaxed,
        );

        let inventory = Arc::new(PlayerInventory::new(
            living_entity.entity_equipment.clone(),
            living_entity.equipment_slots.clone(),
        ));

        let ender_chest_inventory = Arc::new(EnderChestInventory::new());

        let player_screen_handler = Arc::new(Mutex::new(
            PlayerScreenHandler::new(
                &inventory,
                None,
                0,
                Some(world.server.upgrade().unwrap().recipe_manager.clone()),
            )
            .await,
        ));

        // Initialize abilities based on gamemode (like vanilla's GameMode.setAbilities())
        let mut abilities = Abilities::default();
        abilities.set_for_gamemode(gamemode);

        let properties = gameprofile.properties.load().clone();
        let bedrock_skin = tokio::task::spawn_blocking(move || {
            Self::fetch_skin(&properties)
                .unwrap_or_else(pumpkin_protocol::bedrock::client::Skin::steve)
        })
        .await
        .unwrap_or_else(|_| pumpkin_protocol::bedrock::client::Skin::steve());

        Self {
            living_entity,
            config: ArcSwap::new(Arc::new(config)),
            advancements: Arc::new(Mutex::new(
                server
                    .advancement_manager
                    .clone()
                    .new_player_advancement(gameprofile.id),
            )),
            gameprofile,
            client,
            awaiting_teleport: Mutex::new(None),
            breath_manager: BreathManager::default(),
            // TODO: Load this from previous instance
            hunger_manager: HungerManager::default(),
            current_block_destroy_stage: AtomicI32::new(-1),
            enchantment_seed: AtomicI32::new(rand::random()),
            open_container: AtomicCell::new(None),
            open_container_pos: AtomicCell::new(None),
            tick_counter: AtomicI32::new(0),
            start_mining_time: AtomicI32::new(0),
            last_input: AtomicI8::new(0),
            carried_item: Mutex::new(None),
            experience_pick_up_delay: Mutex::new(0),
            teleport_id_count: AtomicI32::new(0),
            mining: AtomicBool::new(false),
            mining_pos: Mutex::new(BlockPos::ZERO),
            abilities: Mutex::new(abilities),
            stats: Mutex::new(statistics::Statistics::default()),
            gamemode: AtomicCell::new(gamemode),
            previous_gamemode: AtomicCell::new(None),
            camera_target_id: AtomicCell::new(None),
            // TODO: Send the CPlayerSpawnPosition packet when the client connects with proper values
            respawn_point: Mutex::new(None),
            sleeping_since: AtomicCell::new(None),
            // We want this to be an impossible watched section so that `chunker::update_position`
            // will mark chunks as watched for a new join rather than a respawn.
            // (We left shift by one so we can search around that chunk)
            watched_section: AtomicCell::new(Cylindrical::new(
                Vector2::new(0, 0),
                // Since 1 is not possible in vanilla it is used as uninit
                NonZeroU8::new(1).unwrap(),
            )),
            last_action_time: AtomicCell::new(std::time::Instant::now()),
            ping: AtomicU32::new(0),
            last_attacked_ticks: AtomicU32::new(0),
            client_loaded: AtomicBool::new(false),
            bedrock_spawned: AtomicBool::new(false),
            client_loaded_timeout: AtomicU32::new(60),
            // Item usage tracking
            using_item: AtomicBool::new(false),
            item_use_start_time: AtomicI32::new(0),
            using_hand: AtomicCell::new(None),
            // Minecraft has no way to change the default permission level of new players.
            // Minecraft's default permission level is 0.
            permission_lvl: server
                .data
                .operator_config
                .read()
                .await
                .get_entry(&player_uuid)
                .map_or(
                    AtomicCell::new(server.advanced_config.commands.default_op_level),
                    |op| AtomicCell::new(op.level),
                ),
            inventory,
            ender_chest_inventory,
            experience_level: AtomicI32::new(0),
            experience_progress: AtomicCell::new(0.0),
            experience_points: AtomicI32::new(0),
            item_cooldowns: Mutex::new(HashMap::new()),
            // Default to sending 16 chunks per tick.
            chunk_manager: Mutex::new(ChunkManager::new(
                16,
                world.level.chunk_listener.add_global_chunk_listener(),
                world.clone(),
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
            screen_handler_listener: Arc::new(ScreenListener),
            screen_handler_sync_handler: Arc::new(SyncHandler::new()),
            tab_list_header: Mutex::new(TextComponent::text("")),
            tab_list_footer: Mutex::new(TextComponent::text("")),
            display_name: Mutex::new(None),
            tab_list_name: Mutex::new(None),
            tab_list_order: AtomicI32::new(0),
            tab_list_latency: AtomicI32::new(0),
            tab_list_listed: AtomicBool::new(true),
            fishing_bobber: AtomicI32::new(-1),
            bedrock_skin: ArcSwap::new(Arc::new(bedrock_skin)),
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

    /// Removes the [`Player`] out of the current [`World`].
    pub async fn remove(self: &Arc<Self>) {
        self.stats
            .lock()
            .await
            .increment_custom(statistics::CustomStatistic::LeaveGame, 1);
        let world = self.world();
        world.remove_player(self, true).await;

        let cylindrical = self.watched_section.load();
        self.chunk_manager.lock().await.clean_up(&world.level);

        // Radial chunks are all of the chunks the player is theoretically viewing.
        // Given enough time, all of these chunks will be in memory.
        let radial_chunks = cylindrical.all_chunks_within();

        debug!(
            "Removing player {}, unwatching {} chunks",
            self.gameprofile.name,
            radial_chunks.len()
        );

        let level = &world.level;

        // Decrement the value of watched chunks
        let chunks_to_clean = level.mark_chunks_as_not_watched(&radial_chunks).await;
        // Remove chunks with no watchers from the cache
        if !chunks_to_clean.is_empty() {
            world.remove_entities_in_chunks(&chunks_to_clean).await;
            level.clean_entity_chunks(&chunks_to_clean);
        }
        // Remove left over entries from all possiblily loaded chunks
        let cleaned_chunks = level.clean_memory();
        if !cleaned_chunks.is_empty() {
            world.remove_entities_in_chunks(&cleaned_chunks).await;
            level.clean_entity_chunks(&cleaned_chunks);
        }

        debug!(
            "Removed player id {} from world {} ({} chunks remain cached)",
            self.gameprofile.name,
            self.world().get_world_name(),
            level.loaded_chunk_count(),
        );

        //self.world().level.list_cached();
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

    pub const fn entity_id(&self) -> i32 {
        self.living_entity.entity.entity_id
    }

    pub fn world(&self) -> Arc<World> {
        self.living_entity.entity.world.load_full()
    }

    pub fn position(&self) -> Vector3<f64> {
        self.living_entity.entity.pos.load()
    }

    pub fn eye_position(&self) -> Vector3<f64> {
        let eye_height = self.living_entity.entity.get_eye_height();
        Vector3::new(
            self.living_entity.entity.pos.load().x,
            self.living_entity.entity.pos.load().y + eye_height,
            self.living_entity.entity.pos.load().z,
        )
    }

    /// Returns the player's rotation.
    /// Yaw then Pitch
    pub fn rotation(&self) -> (f32, f32) {
        (
            self.living_entity.entity.yaw.load(),
            self.living_entity.entity.pitch.load(),
        )
    }

    /// Updates the last action time to now. Call this on player actions like movement, chat, etc.
    pub fn update_last_action_time(&self) {
        self.last_action_time.store(std::time::Instant::now());
    }

    pub async fn get_ip(&self) -> String {
        self.client.address().await.to_string()
    }

    async fn handle_killed(&self, death_msg: TextComponent) {
        self.trigger_advancement(
            crate::entity::player::advancement::trigger::AdvancementTrigger::PlayerKilled,
        )
        .await;
        self.set_client_loaded(false);
        let block_pos = self.position().to_block_pos();

        let keep_inventory = { self.world().level_info.load().game_rules.keep_inventory };

        if !keep_inventory {
            for item in &self.inventory().main_inventory {
                let mut lock = item.lock().await;
                self.world()
                    .drop_stack(
                        &block_pos,
                        mem::replace(&mut *lock, ItemStack::EMPTY.clone()),
                    )
                    .await;
            }
        }

        // Reset air supply & drowning ticks on death
        self.breath_manager.reset(self);

        self.client
            .send_packet_now(&CCombatDeath::new(self.entity_id().into(), &death_msg))
            .await;
    }

    pub async fn get_command_source(self: &Arc<Self>, server: &Arc<Server>) -> CommandSource {
        CommandSender::Player(self.clone())
            .into_source(server)
            .await
    }

    pub async fn has_advancement(
        &self,
        advancement: &'static pumpkin_data::advancement::Advancement,
    ) -> bool {
        let advancements = self.advancements.lock().await;
        advancements
            .progress
            .map
            .get(advancement)
            .is_some_and(crate::entity::player::advancement::AdvancementProgress::is_done)
    }

    pub async fn trigger_advancement_criterion(
        &self,
        advancement: &'static pumpkin_data::advancement::Advancement,
        criterion: &str,
    ) {
        let mut advancements = self.advancements.lock().await;
        advancements.award(advancement, criterion);
    }

    pub async fn check_inventory_advancements(&self) {
        self.trigger_advancement(
            crate::entity::player::advancement::trigger::AdvancementTrigger::InventoryChanged,
        )
        .await;
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.gameprofile.id == other.gameprofile.id
    }
}

impl EntityBase for Player {
    fn damage_with_context<'a>(
        &'a self,
        caller: &'a dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if self.abilities.lock().await.invulnerable
                && damage_type != DamageType::GENERIC_KILL
                && damage_type != DamageType::OUT_OF_WORLD
            {
                return false;
            }
            // TODO: Implement shield blocking durability.
            let result = self
                .living_entity
                .damage_with_context(caller, amount, damage_type, position, source, cause)
                .await;
            if result {
                let health = self.living_entity.health.load();
                if health <= 0.0 {
                    let death_message =
                        LivingEntity::get_death_message(caller, damage_type, source, cause).await;
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
            if Arc::ptr_eq(&world, &self.world()) {
                // Same world
                let yaw = yaw.unwrap_or(self.living_entity.entity.yaw.load());
                let pitch = pitch.unwrap_or(self.living_entity.entity.pitch.load());
                let server = self.world().server.upgrade().unwrap();
                send_cancellable! {{
                    server;
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
                        let chunk_pos = entity.chunk_pos.load();
                        entity
                            .world
                            .load()
                            .broadcast_to_chunk_except(
                                chunk_pos,
                                &[self.living_entity.entity.entity_uuid],
                                &CEntityPositionSync::new(
                                    self.living_entity.entity.entity_id.into(),
                                    position,
                                    Vector3::new(0.0, 0.0, 0.0),
                                    yaw,
                                    pitch,
                                    entity.on_ground.load(Ordering::SeqCst),
                                )
                            )
                            ;
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

    fn set_on_fire_for_ticks(&self, ticks: u32) {
        let entity = self.get_entity();
        let ticks = if entity.invulnerable.load(Ordering::Relaxed) {
            1
        } else {
            ticks
        };
        if entity.fire_ticks.load(Ordering::Relaxed) < ticks as i32 {
            entity.fire_ticks.store(ticks as i32, Ordering::Relaxed);
        }
    }

    fn is_pushable(&self) -> bool {
        self.gamemode.load() != GameMode::Spectator && self.gamemode.load() != GameMode::Creative
    }

    fn get_name(&self) -> TextComponent {
        //TODO: team color
        TextComponent::text(self.gameprofile.name.clone())
    }

    fn get_display_name(&self) -> EntityBaseFuture<'_, TextComponent> {
        Box::pin(async move {
            if let Some(display_name) = self.display_name.lock().await.as_ref() {
                return display_name.clone();
            }
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
            name.insertion(self.gameprofile.name.clone())
        })
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn get_experience_reward(&self, _killer: Option<&dyn EntityBase>) -> u32 {
        // vanilla: min(level * 7, 100)
        let level = self.experience_level.load(Ordering::Relaxed);
        (level * 7).min(100) as u32
    }

    fn tick_in_void<'a>(&'a self, dyn_self: &'a dyn EntityBase) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.living_entity.tick_in_void(dyn_self).await;
        })
    }
}
