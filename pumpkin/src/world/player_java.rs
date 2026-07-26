use crate::command::client_suggestions;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::plugin::player::player_join::PlayerJoinEvent;
use crate::server::Server;
use crate::world::World;
use crate::world::chunker;
use bytes::BufMut;
use pumpkin_config::BasicConfiguration;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_data::translation;
use pumpkin_inventory::crafting::recipe_provider::RecipeProvider;
use pumpkin_inventory::screen_handler::InventoryPlayer;
use pumpkin_protocol::bedrock::client::EntityProperties;
use pumpkin_protocol::bedrock::client::add_player::CAddPlayer;
use pumpkin_protocol::bedrock::client::player_list::{CPlayerList, PlayerListEntry};
use pumpkin_protocol::bedrock::client::set_actor_data::{CSetActorData, PropertySyncData};
use pumpkin_protocol::bedrock::network_item::NetworkItemDescriptor;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::java;
use pumpkin_protocol::java::client::play::{
    CChunkBatchEnd, CChunkBatchStart, CChunkData, CGameEvent, CLogin, CPlayerInfoUpdate,
    CPlayerSpawnPosition, CRecipeBookAdd, CRecipeBookSettings, CSetEntityMetadata, CSetEquipment,
    CSetSelectedSlot, CSpawnEntity, GameEvent, InitChat, Metadata, PlayerAction, PlayerInfoFlags,
    PlayerSpawnData,
};
use pumpkin_util::GameMode;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::biome;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tracing::{debug, info};

impl World {
    #[expect(clippy::too_many_lines)]
    pub async fn spawn_java_player(
        &self,
        base_config: &BasicConfiguration,
        player: &Arc<Player>,
        server: &Arc<Server>,
    ) {
        let dimensions: Vec<ResourceLocation> = server
            .dimensions
            .iter()
            .map(|d| ResourceLocation::from(d.minecraft_name))
            .collect();

        // This code follows the vanilla packet order
        let entity_id = player.entity_id();
        let gamemode = player.gamemode.load();
        debug!(
            "spawning player {}, entity id {}",
            player.gameprofile.name, entity_id
        );

        let Some(client) = player.client.java() else {
            return;
        };
        // Send the login packet for our new player
        client
            .send_packet_now(&CLogin::new(
                entity_id,
                base_config.hardcore,
                &dimensions,
                server
                    .advanced_config
                    .networking
                    .java
                    .max_players
                    .try_into()
                    .unwrap(),
                server
                    .advanced_config
                    .networking
                    .java
                    .view_distance
                    .get()
                    .into(), //  TODO: view distance
                server
                    .advanced_config
                    .networking
                    .java
                    .simulation_distance
                    .get()
                    .into(), // TODO: sim view dinstance
                false,
                true,
                false,
                PlayerSpawnData::new(
                    self.dimension.clone(),
                    biome::hash_seed(self.level.seed.0), // seed
                    gamemode as u8,
                    player
                        .previous_gamemode
                        .load()
                        .map_or(-1, |gamemode| gamemode as i8),
                    false,
                    false,
                    None,
                    VarInt(player.get_entity().portal_cooldown.load(Ordering::Relaxed) as i32),
                    self.sea_level.into(),
                ),
                server.advanced_config.networking.java.online_mode,
                // This should stay true even when reports are disabled.
                // It prevents the annoying popup when joining the server.
                true,
            ))
            .await;

        // Send the current ticking state to the new player so they are in sync.
        server.tick_rate_manager.update_joining_player(player).await;

        // Permissions, i.e. the commands a player may use.
        player.send_permission_lvl_update();

        // Difficulty of the world
        player.send_difficulty_update().await;
        {
            let command_dispatcher = server.command_dispatcher.read().await;

            client_suggestions::send_c_commands_packet(player, server, &command_dispatcher).await;
        };

        let (position, yaw, pitch) = if player.has_played_before.load(Ordering::Relaxed) {
            let position = player.position();
            let yaw = player.get_entity().yaw.load(); //info.spawn_angle;
            let pitch = player.get_entity().pitch.load();

            (position, yaw, pitch)
        } else {
            let info = &self.level_info.load();
            let spawn_position = Vector2::new(info.spawn_x, info.spawn_z);
            let chunk_pos = Vector2::new(info.spawn_x >> 4, info.spawn_z >> 4);
            self.level.get_or_fetch_chunk(chunk_pos, |_| ()).await;
            let pos_y = self.get_top_block(spawn_position) + 1; // +1 to spawn on top of the block

            let position = Vector3::new(
                f64::from(info.spawn_x) + 0.5,
                f64::from(pos_y),
                f64::from(info.spawn_z) + 0.5,
            );
            (position, info.spawn_yaw, info.spawn_pitch)
        };

        // Load chunks around the real spawn position before teleporting the client there.
        player.living_entity.entity.set_pos(position);
        player.living_entity.entity.set_rotation(yaw, pitch);
        player.living_entity.entity.last_pos.store(position);
        chunker::update_position(player).await;

        let center_chunk = player.living_entity.entity.chunk_pos.load();
        let chunk = self
            .level
            .get_or_fetch_chunk(center_chunk, std::clone::Clone::clone)
            .await;
        client.send_packet_now(&CChunkBatchStart).await;
        client.send_packet_now(&CChunkData(&chunk)).await;
        client.send_packet_now(&CChunkBatchEnd::new(1u16)).await;

        let velocity = player.living_entity.entity.velocity.load();

        debug!("Sending player teleport to {}", player.gameprofile.name);
        player.request_teleport(position, yaw, pitch).await;

        let gameprofile = &player.gameprofile;
        let bedrock_player_list = CPlayerList {
            action: CPlayerList::ACTION_ADD,
            entries: vec![PlayerListEntry {
                uuid: gameprofile.id,
                entity_unique_id: VarLong(entity_id as i64),
                username: gameprofile.name.clone(),
                xuid: String::new(),
                platform_chat_id: String::new(),
                build_platform: 0,
                skin: (**player.bedrock_skin.load()).clone(),
                is_teacher: false,
                is_host: false,
                is_sub_client: false,
                player_color: [0, 0, 0, 0],
            }],
        };

        let player_actions = [
            PlayerAction::AddPlayer {
                name: &gameprofile.name,
                properties: &gameprofile.properties.load(),
            },
            PlayerAction::UpdateGameMode(VarInt(gamemode as i32)),
            PlayerAction::UpdateListed(true),
            PlayerAction::UpdateLatency(VarInt(0)),
            PlayerAction::UpdateListOrder(VarInt(0)),
        ];
        let java_player = [pumpkin_protocol::java::client::play::Player {
            uuid: gameprofile.id,
            actions: &player_actions,
        }];
        let player_info_update = CPlayerInfoUpdate::new(
            (PlayerInfoFlags::ADD_PLAYER
                | PlayerInfoFlags::UPDATE_GAME_MODE
                | PlayerInfoFlags::UPDATE_LISTED
                | PlayerInfoFlags::UPDATE_LATENCY
                | PlayerInfoFlags::UPDATE_LIST_PRIORITY)
                .bits(),
            &java_player,
        );

        self.broadcast_editioned(&player_info_update, &bedrock_player_list)
            .await;

        // If the player has a custom tab_list_name, send an update for it
        if let Some(tab_list_name) = player.get_tab_list_name().await {
            let actions = [PlayerAction::UpdateDisplayName(Some(&tab_list_name))];
            let java_player = [pumpkin_protocol::java::client::play::Player {
                uuid: gameprofile.id,
                actions: &actions,
            }];
            self.broadcast_packet_all(&CPlayerInfoUpdate::new(
                PlayerInfoFlags::UPDATE_DISPLAY_NAME.bits(),
                &java_player,
            ));
        }

        // Here, we send all the infos of players who already joined.
        let mut players_tab_list_names = Vec::new();
        {
            let players = self.players.load();
            let mut data_to_process = Vec::new();
            for p in players
                .iter()
                .filter(|p| p.gameprofile.id != player.gameprofile.id)
            {
                let props_guard = p.gameprofile.properties.load();
                data_to_process.push((props_guard, p));
            }

            let mut current_player_data = Vec::new();
            for (properties, player) in &data_to_process {
                let chat_session = player.chat_session.lock().await;
                let tab_list_name = player.get_tab_list_name().await;

                let mut player_actions = vec![
                    PlayerAction::AddPlayer {
                        name: &player.gameprofile.name,
                        properties,
                    },
                    PlayerAction::UpdateGameMode(VarInt(player.gamemode.load() as i32)),
                    PlayerAction::UpdateListed(player.tab_list_listed.load(Ordering::Relaxed)),
                    PlayerAction::UpdateLatency(VarInt(
                        player.tab_list_latency.load(Ordering::Relaxed),
                    )),
                    PlayerAction::UpdateListOrder(VarInt(
                        player.tab_list_order.load(Ordering::Relaxed),
                    )),
                ];

                if base_config.allow_chat_reports {
                    player_actions.push(PlayerAction::InitializeChat(Some(InitChat {
                        session_id: chat_session.session_id,
                        expires_at: chat_session.expires_at,
                        public_key: chat_session.public_key.clone(),
                        signature: chat_session.signature.clone(),
                    })));
                }
                drop(chat_session);

                current_player_data.push((&player.gameprofile.id, player_actions));

                // Collect tab_list_names for sending later
                if tab_list_name.is_some() {
                    players_tab_list_names.push((player.gameprofile.id, tab_list_name));
                }
            }

            let mut action_flags = PlayerInfoFlags::ADD_PLAYER
                | PlayerInfoFlags::UPDATE_LISTED
                | PlayerInfoFlags::UPDATE_LATENCY
                | PlayerInfoFlags::UPDATE_LIST_PRIORITY
                | PlayerInfoFlags::UPDATE_GAME_MODE;
            if base_config.allow_chat_reports {
                action_flags |= PlayerInfoFlags::INITIALIZE_CHAT;
            }

            let entries = current_player_data
                .iter()
                .map(|(id, actions)| java::client::play::Player {
                    uuid: **id,
                    actions,
                })
                .collect::<Vec<_>>();

            debug!("Sending player info to {}", player.gameprofile.name);
            client
                .enqueue_packet(&CPlayerInfoUpdate::new(action_flags.bits(), &entries))
                .await;

            // Send tab_list_names for existing players with custom names
            for (player_id, tab_list_name) in &players_tab_list_names {
                if let Some(name) = tab_list_name {
                    let actions = [PlayerAction::UpdateDisplayName(Some(name))];
                    let java_player = [pumpkin_protocol::java::client::play::Player {
                        uuid: *player_id,
                        actions: &actions,
                    }];
                    client
                        .enqueue_packet(&CPlayerInfoUpdate::new(
                            PlayerInfoFlags::UPDATE_DISPLAY_NAME.bits(),
                            &java_player,
                        ))
                        .await;
                }
            }
        };

        let gameprofile = &player.gameprofile;

        let bedrock_add_player = CAddPlayer {
            uuid: gameprofile.id,
            username: gameprofile.name.clone(),
            entity_runtime_id: VarULong(entity_id as u64),
            platform_chat_id: String::new(),
            position: Vector3::new(position.x as f32, position.y as f32, position.z as f32),
            velocity: Vector3::new(velocity.x as f32, velocity.y as f32, velocity.z as f32),
            pitch,
            yaw,
            head_yaw: yaw,
            held_item: NetworkItemDescriptor::default(),
            game_mode: VarInt(match player.gamemode.load() {
                GameMode::Survival => 0,
                GameMode::Creative => 1,
                GameMode::Adventure => 2,
                GameMode::Spectator => 6,
            }),
            metadata: player.get_entity().bedrock_metadata(),
            properties: EntityProperties::default(),
            ability_data: pumpkin_protocol::bedrock::client::add_player::AbilityData {
                entity_unique_id: entity_id as i64,
                player_permissions: 0,
                command_permissions: 0,
                layers: vec![pumpkin_protocol::bedrock::client::AbilityLayer {
                    serialized_layer: 0,
                    abilities_set: 0,
                    ability_value: 0,
                    fly_speed: 0.05,
                    vertical_fly_speed: 0.05,
                    walk_speed: 0.1,
                }],
            },
            links: Vec::new(),
            device_id: String::new(),
            build_platform: 0,
        };

        // Spawn the player for every client.
        let spawn_entity = CSpawnEntity::new(
            entity_id.into(),
            gameprofile.id,
            i32::from(EntityType::PLAYER.id).into(),
            position,
            pitch,
            yaw,
            yaw,
            0.into(),
            velocity,
        );

        self.broadcast_packet_except_editioned_sync(
            &[player.gameprofile.id],
            &spawn_entity,
            &bedrock_add_player,
        );

        // Broadcast metadata to Java players so they can correctly interact with the new player
        let config = player.config.load();
        let mut java_meta_buf = Vec::new();
        {
            let meta = Metadata::new(
                TrackedData::PLAYER_MODE_CUSTOMISATION,
                MetaDataType::BYTE,
                config.skin_parts,
            );
            meta.write(&mut java_meta_buf, &JavaMinecraftVersion::V_1_21_4)
                .unwrap();
        };
        java_meta_buf.put_u8(255);

        self.broadcast_packet_except_editioned_sync(
            &[gameprofile.id],
            &CSetEntityMetadata::new((entity_id).into(), java_meta_buf.into()),
            &CSetActorData {
                actor_runtime_id: VarULong(entity_id as u64),
                metadata: player.get_entity().bedrock_metadata(),
                synced_properties: PropertySyncData {
                    int_properties: HashMap::new(),
                    float_properties: HashMap::new(),
                },
                tick: VarULong(0),
            },
        );

        // Spawn players for our client.
        let id = player.gameprofile.id;
        for existing_player in self
            .players
            .load()
            .iter()
            .filter(|c| c.gameprofile.id != id)
        {
            let entity = &existing_player.get_entity();
            let pos = entity.pos.load();
            let gameprofile = &existing_player.gameprofile;
            let bedrock_add_player = CAddPlayer {
                uuid: gameprofile.id,
                username: gameprofile.name.clone(),
                entity_runtime_id: VarULong(existing_player.entity_id() as u64),
                platform_chat_id: String::new(),
                position: Vector3::new(pos.x as f32, pos.y as f32, pos.z as f32),
                velocity: Vector3::new(
                    entity.velocity.load().x as f32,
                    entity.velocity.load().y as f32,
                    entity.velocity.load().z as f32,
                ),
                pitch: entity.pitch.load(),
                yaw: entity.yaw.load(),
                head_yaw: entity.head_yaw.load(),
                held_item: NetworkItemDescriptor::default(),
                game_mode: VarInt(match existing_player.gamemode.load() {
                    GameMode::Survival => 0,
                    GameMode::Creative => 1,
                    GameMode::Adventure => 2,
                    GameMode::Spectator => 6,
                }),
                metadata: entity.bedrock_metadata(),
                properties: EntityProperties::default(),
                ability_data: pumpkin_protocol::bedrock::client::add_player::AbilityData {
                    entity_unique_id: existing_player.entity_id() as i64,
                    player_permissions: 0,
                    command_permissions: 0,
                    layers: vec![pumpkin_protocol::bedrock::client::AbilityLayer {
                        serialized_layer: 0,
                        abilities_set: 0,
                        ability_value: 0,
                        fly_speed: 0.05,
                        vertical_fly_speed: 0.05,
                        walk_speed: 0.1,
                    }],
                },
                links: Vec::new(),
                device_id: String::new(),
                build_platform: 0,
            };

            let bedrock_player_list = CPlayerList {
                action: CPlayerList::ACTION_ADD,
                entries: vec![PlayerListEntry {
                    uuid: gameprofile.id,
                    entity_unique_id: VarLong(existing_player.entity_id() as i64),
                    username: gameprofile.name.clone(),
                    xuid: String::new(),
                    platform_chat_id: String::new(),
                    build_platform: 0,
                    skin: (**existing_player.bedrock_skin.load()).clone(),
                    is_teacher: false,
                    is_host: false,
                    is_sub_client: false,
                    player_color: [0, 0, 0, 0],
                }],
            };

            let actions = [
                PlayerAction::AddPlayer {
                    name: &gameprofile.name,
                    properties: &gameprofile.properties.load(),
                },
                PlayerAction::UpdateGameMode(VarInt(existing_player.gamemode.load() as i32)),
                PlayerAction::UpdateListed(existing_player.tab_list_listed.load(Ordering::Relaxed)),
                PlayerAction::UpdateLatency(VarInt(
                    existing_player.tab_list_latency.load(Ordering::Relaxed),
                )),
                PlayerAction::UpdateListOrder(VarInt(
                    existing_player.tab_list_order.load(Ordering::Relaxed),
                )),
            ];
            let java_player = [pumpkin_protocol::java::client::play::Player {
                uuid: gameprofile.id,
                actions: &actions,
            }];
            player
                .client
                .enqueue_packet_editioned(
                    &CPlayerInfoUpdate::new(
                        (PlayerInfoFlags::ADD_PLAYER
                            | PlayerInfoFlags::UPDATE_LISTED
                            | PlayerInfoFlags::UPDATE_GAME_MODE
                            | PlayerInfoFlags::UPDATE_LATENCY
                            | PlayerInfoFlags::UPDATE_LIST_PRIORITY)
                            .bits(),
                        &java_player,
                    ),
                    &bedrock_player_list,
                )
                .await;

            player
                .client
                .enqueue_packet_editioned(
                    &CSpawnEntity::new(
                        existing_player.entity_id().into(),
                        gameprofile.id,
                        i32::from(EntityType::PLAYER.id).into(),
                        pos,
                        entity.pitch.load(),
                        entity.yaw.load(),
                        entity.head_yaw.load(),
                        0.into(),
                        entity.velocity.load(),
                    ),
                    &bedrock_add_player,
                )
                .await;

            {
                let config = existing_player.config.load();
                let mut buf = Vec::new();
                {
                    let meta = Metadata::new(
                        TrackedData::PLAYER_MODE_CUSTOMISATION,
                        MetaDataType::BYTE,
                        config.skin_parts,
                    );
                    meta.write(&mut buf, &client.version.load()).unwrap();
                };
                drop(config);
                // END
                buf.put_u8(255);
                client
                    .enqueue_packet(&CSetEntityMetadata::new(
                        existing_player.get_entity().entity_id.into(),
                        buf.into(),
                    ))
                    .await;
            };

            {
                let mut equipment_list = Vec::new();

                equipment_list.push((
                    EquipmentSlot::MAIN_HAND.discriminant(),
                    existing_player.inventory.held_item().lock().await.clone(),
                ));

                for (slot, item_arc_mutex) in &existing_player
                    .inventory
                    .entity_equipment
                    .lock()
                    .await
                    .equipment
                {
                    let item_stack = item_arc_mutex.lock().await.clone();
                    equipment_list.push((slot.discriminant(), item_stack));
                }

                let equipment: Vec<(i8, ItemStackSerializer)> = equipment_list
                    .iter()
                    .map(|(slot, stack)| (*slot, ItemStackSerializer::from(stack.clone())))
                    .collect();

                client
                    .enqueue_packet(&CSetEquipment::new(
                        existing_player.entity_id().into(),
                        equipment,
                    ))
                    .await;
            }
        }
        player.send_client_information();

        player.send_abilities_update().await;

        // Sync selected slot
        player
            .enqueue_set_held_item_packet(&CSetSelectedSlot::new(
                player.get_inventory().get_selected_slot() as i8,
            ))
            .await;

        // Start waiting for level chunks. Sets the "Loading Terrain" screen
        debug!("Sending waiting chunks to {}", player.gameprofile.name);
        client
            .send_packet_now(&CGameEvent::new(GameEvent::StartWaitingChunks, 0.0))
            .await;

        self.worldborder.lock().await.init_client(client).await;

        // Sends initial time
        player.send_time(self).await;

        let (spawn_block_pos, yaw, pitch) = {
            let level_info_lock = self.level_info.load();
            (
                BlockPos::new(
                    level_info_lock.spawn_x,
                    level_info_lock.spawn_y,
                    level_info_lock.spawn_z,
                ),
                level_info_lock.spawn_yaw,
                level_info_lock.spawn_pitch,
            )
        };

        client
            .send_packet_now(&CPlayerSpawnPosition::new(
                spawn_block_pos,
                yaw,
                pitch,
                self.dimension.minecraft_name.to_owned(),
            ))
            .await;

        // Send initial weather state
        let weather = self.weather.lock().await;
        if weather.raining {
            client
                .enqueue_packet(&CGameEvent::new(GameEvent::BeginRaining, 0.0))
                .await;

            // Calculate rain and thunder levels directly from public fields
            let rain_level = weather.rain_level.clamp(0.0, 1.0);
            let thunder_level = weather.thunder_level.clamp(0.0, 1.0);
            drop(weather);

            client
                .enqueue_packet(&CGameEvent::new(GameEvent::RainLevelChange, rain_level))
                .await;
            client
                .enqueue_packet(&CGameEvent::new(
                    GameEvent::ThunderLevelChange,
                    thunder_level,
                ))
                .await;
        }

        // if let Some(bossbars) = self..lock().get_player_bars(&player.gameprofile.id) {
        //     for bossbar in bossbars {
        //         player.send_bossbar(bossbar);
        //     }
        // }

        player.has_played_before.store(true, Ordering::Relaxed);
        player
            .on_screen_handler_opened(player.player_screen_handler.clone())
            .await;

        player.send_active_effects().await;
        self.send_player_equipment(player).await;

        if let crate::net::ClientPlatform::Java(java_client) = player.client.as_ref()
            && server.advanced_config.recipe.send_recipes
        {
            java_client
                .send_packet_now(&CRecipeBookSettings::default_closed())
                .await;
            let dynamic_recipes = server.recipe_manager.get_dynamic_recipes().await;
            java_client
                .send_packet_now(&CRecipeBookAdd::new(true, &dynamic_recipes))
                .await;
        }

        let msg_comp = TextComponent::translate_cross(
            translation::java::MULTIPLAYER_PLAYER_JOINED,
            translation::bedrock::MULTIPLAYER_PLAYER_JOINED,
            [TextComponent::text(player.gameprofile.name.clone())],
        )
        .color_named(NamedColor::Yellow);
        let event = PlayerJoinEvent::new(player.clone(), msg_comp);

        let event = server.plugin_manager.fire(event).await;

        if !event.cancelled {
            self.broadcast_system_message(&event.join_message, false)
                .await;
            // TODO: Switch to structured logging, e.g. info!(player = %name, "connected")
            info!("{}", event.join_message.to_pretty_console());
        }
    }

    async fn send_player_equipment(&self, from: &Player) {
        let mut equipment_list = Vec::new();

        equipment_list.push((
            EquipmentSlot::MAIN_HAND.discriminant(),
            from.inventory.held_item().lock().await.clone(),
        ));

        for (slot, item_arc_mutex) in &from.inventory.entity_equipment.lock().await.equipment {
            let item_stack = item_arc_mutex.lock().await.clone();
            equipment_list.push((slot.discriminant(), item_stack));
        }

        let equipment: Vec<(i8, ItemStackSerializer)> = equipment_list
            .iter()
            .map(|(slot, stack)| (*slot, ItemStackSerializer::from(stack.clone())))
            .collect();
        let chunk_pos = from.get_entity().chunk_pos.load();
        self.broadcast_to_chunk_except(
            chunk_pos,
            &[from.get_entity().entity_uuid],
            &CSetEquipment::new(from.entity_id().into(), equipment),
        );
    }
}
