use crate::command::client_suggestions;
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::plugin::player::player_join::PlayerJoinEvent;
use crate::server::Server;
use crate::world::World;
use bytes::BufMut;
use pumpkin_config::BasicConfiguration;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::{BedrockItem, BedrockItemVersion};
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_data::translation;
use pumpkin_protocol::bedrock::client::add_player::CAddPlayer;
use pumpkin_protocol::bedrock::client::creative_content::{
    CCreativeContent, CreativeCategory, Entry, Group,
};
use pumpkin_protocol::bedrock::client::gamerules_changed::GameRules;
use pumpkin_protocol::bedrock::client::item_registry::{CItemRegistry, ItemDefinition};
use pumpkin_protocol::bedrock::client::player_list::{CPlayerList, PlayerListEntry};
use pumpkin_protocol::bedrock::client::set_actor_data::{CSetActorData, PropertySyncData};
use pumpkin_protocol::bedrock::client::start_game::{
    CStartGame, Experiments, GamePublishSetting, LevelSettings, ServerTelemetryData,
};
use pumpkin_protocol::bedrock::client::update_attributes::{Attribute, CUpdateAttributes};
use pumpkin_protocol::bedrock::client::{CInventoryContent, EntityProperties};
use pumpkin_protocol::bedrock::network_item::{
    ContainerName, FullContainerName, NetworkItemDescriptor, NetworkItemStackDescriptor,
};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::codec::var_uint::VarUInt;
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::java::client::play::{
    CPlayerInfoUpdate, CSetEntityMetadata, CSpawnEntity, Metadata, PlayerAction, PlayerInfoFlags,
};
use pumpkin_util::GameMode;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::CURRENT_BEDROCK_MC_VERSION;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tracing::info;
use uuid::Uuid;

impl World {
    #[allow(clippy::too_many_lines)]
    pub async fn spawn_bedrock_player(
        &self,
        base_config: &BasicConfiguration,
        player: Arc<Player>,
        server: &Server,
    ) {
        static CREATIVE_CONTENT: std::sync::OnceLock<(Vec<Group>, Vec<Entry>)> =
            std::sync::OnceLock::new();

        static BEDROCK_CRAFTING_DATA: std::sync::OnceLock<
            Vec<pumpkin_protocol::bedrock::client::BedrockRecipe>,
        > = std::sync::OnceLock::new();

        let level_info = server.level_info.load();
        let weather = self.weather.lock().await;
        let runtime_id = player.entity_id() as u64;

        let (position, yaw, pitch) = if player.has_played_before.load(Ordering::Relaxed) {
            let position = player.position();
            let yaw = player.get_entity().yaw.load(); //info.spawn_angle;
            let pitch = player.get_entity().pitch.load();

            (position, yaw, pitch)
        } else {
            let spawn_position = Vector2::new(level_info.spawn_x, level_info.spawn_z);
            let chunk_pos = Vector2::new(level_info.spawn_x >> 4, level_info.spawn_z >> 4);
            self.level.get_or_fetch_chunk(chunk_pos, |_| ()).await;
            let pos_y = self.get_top_block(spawn_position) + 1; // +1 to spawn on top of the block

            let position = Vector3::new(
                f64::from(level_info.spawn_x) + 0.5,
                f64::from(pos_y),
                f64::from(level_info.spawn_z) + 0.5,
            );
            (position, level_info.spawn_yaw, level_info.spawn_pitch)
        };

        // Todo make the data less spread
        let level_settings = LevelSettings {
            seed: self.level.seed.0,
            spawn_biome_type: 0,
            custom_biome_name: String::new(),
            dimension: VarInt(0),
            generator_type: VarInt(1),
            world_gamemode: server.defaultgamemode.lock().await.gamemode,
            hardcore: base_config.hardcore,
            difficulty: VarInt(level_info.difficulty as i32),
            spawn_position: BlockPos::new(
                level_info.spawn_x,
                level_info.spawn_y,
                level_info.spawn_z,
            ),
            has_achievements_disabled: false,
            editor_world_type: VarInt(0),
            is_created_in_editor: false,
            is_exported_from_editor: false,
            day_cycle_stop_time: VarInt(-1),
            education_edition_offer: VarInt(0),
            has_education_features_enabled: false,
            education_product_id: String::new(),
            rain_level: weather.rain_level,
            lightning_level: weather.thunder_level,
            has_confirmed_platform_locked_content: false,
            was_multiplayer_intended: true,
            was_lan_broadcasting_intended: true,
            xbox_live_broadcast_setting: GamePublishSetting::Public,
            platform_broadcast_setting: GamePublishSetting::Public,
            commands_enabled: level_info.allow_commands,
            is_texture_packs_required: false,
            rule_data: GameRules {
                list_size: VarUInt(0),
            },
            experiments: Experiments {
                names_size: 0,
                experiments_ever_toggled: false,
            },
            bonus_chest: false,
            has_start_with_map_enabled: false,
            // TODO Bedrock permission level are different
            permission_level: VarInt(2),
            server_simulation_distance: server
                .advanced_config
                .networking
                .bedrock
                .simulation_distance
                .get()
                .into(),
            has_locked_behavior_pack: false,
            has_locked_resource_pack: false,
            is_from_locked_world_template: false,
            is_using_msa_gamertags_only: false,
            is_from_world_template: false,
            is_world_template_option_locked: false,
            is_only_spawning_v1_villagers: false,
            is_disabling_personas: false,
            is_disabling_custom_skins: false,
            emote_chat_muted: false,
            game_version: CURRENT_BEDROCK_MC_VERSION.into(),
            limited_world_width: 0,
            limited_world_height: 0,
            new_nether: true,
            edu_shared_uri_button_name: String::new(),
            edu_shared_uri_link_uri: String::new(),
            override_force_experimental_gameplay_has_value: false,
            chat_restriction_level: 0,
            disable_player_interactions: false,
            server_editor_connection_policy: VarInt(0),
            allow_anonymous_block_drops_in_editor_worlds: false,
        };
        drop(level_info);
        drop(weather);

        let Some(client) = player.client.bedrock() else {
            return;
        };

        client
            .send_game_packet(&CStartGame {
                entity_id: VarLong(runtime_id as _),
                runtime_entity_id: VarULong(runtime_id),
                player_gamemode: player.gamemode.load(),
                position: Vector3::new(position.x as f32, position.y as f32, position.z as f32),
                pitch,
                yaw,
                level_settings,
                level_id: String::new(),
                level_name: "Pumpkin world".to_string(),
                premium_world_template_id: String::new(),
                is_trial: false,
                rewind_history_size: VarInt(0),
                server_authoritative_block_breaking: true,
                current_level_time: self.level_time.lock().await.world_age as _,
                enchantment_seed: VarInt(0),
                block_properties_size: VarUInt(0),
                // TODO Make this unique
                multiplayer_correlation_id: Uuid::default().to_string(),
                enable_itemstack_net_manager: true,
                server_version: "Pumpkin Rust Server".to_string(),
                compound_id: 10,
                compound_len: VarUInt(0),
                compound_end: 0,
                block_registry_checksum: 0,
                world_template_id: Uuid::nil(),
                enable_clientside_generation: false,
                blocknetwork_ids_are_hashed: false,
                server_auth_sounds: true,
                is_logging_chat: false,
                server_join_information: None,
                telemetry: ServerTelemetryData {
                    server_id: String::new(),
                    scenario_id: String::new(),
                    world_id: String::new(),
                    owner_id: String::new(),
                },
            })
            .await;

        client
            .send_game_packet(&CItemRegistry {
                items: BedrockItem::ALL_BEDROCK_ITEMS
                    .iter()
                    .map(|b| ItemDefinition {
                        name: b.registry_key.into(),
                        id: b.id,
                        component_based: b.component_based,
                        item_version: VarInt::from(match b.version {
                            BedrockItemVersion::Legacy => 0,
                            BedrockItemVersion::DataDriven => 1,
                            BedrockItemVersion::None => 2,
                        }),
                        component_data: b.definition_components.into(),
                    })
                    .collect::<Vec<_>>(),
            })
            .await;

        let (groups, entries) = CREATIVE_CONTENT.get_or_init(|| {
            let groups = pumpkin_data::bedrock_creative::CREATIVE_GROUPS
                .iter()
                .map(|g| {
                    let creative_category = match g.category {
                        1 => CreativeCategory::Construction,
                        2 => CreativeCategory::Nature,
                        3 => CreativeCategory::Equipment,
                        4 => CreativeCategory::Items,
                        5 => CreativeCategory::CommandOnly,
                        _ => CreativeCategory::Undefined,
                    };
                    let icon_item = if g.icon_item_id != 0 {
                        NetworkItemDescriptor {
                            id: VarInt::from(g.icon_item_id),
                            stack_size: 1,
                            aux_value: VarUInt(g.icon_item_aux_value),
                            block_runtime_id: VarInt(0),
                            nbt_data: pumpkin_nbt::Nbt::default(),
                            place_on_blocks: Vec::new(),
                            destroy_blocks: Vec::new(),
                            shield_blocking_tick: 0,
                        }
                    } else {
                        NetworkItemDescriptor::default()
                    };

                    Group {
                        creative_category,
                        name: g.name.to_string(),
                        icon_item,
                    }
                })
                .collect::<Vec<_>>();

            let entries = pumpkin_data::bedrock_creative::CREATIVE_ENTRIES
                .iter()
                .enumerate()
                .map(|(i, e)| Entry {
                    id: VarUInt((i + 1) as u32),
                    item: NetworkItemDescriptor {
                        id: VarInt::from(e.item_id),
                        stack_size: 1,
                        aux_value: VarUInt(e.item_aux_value),
                        block_runtime_id: VarInt(0),
                        nbt_data: pumpkin_nbt::Nbt::default(),
                        place_on_blocks: Vec::new(),
                        destroy_blocks: Vec::new(),
                        shield_blocking_tick: 0,
                    },
                    group_index: VarUInt(e.group_index),
                })
                .collect::<Vec<_>>();

            (groups, entries)
        });

        client
            .send_game_packet(&CCreativeContent { groups, entries })
            .await;

        let bedrock_recipes = BEDROCK_CRAFTING_DATA.get_or_init(|| {
            use pumpkin_data::item::{Item, JavaToBedrockItemMapping};
            use pumpkin_data::recipes::{CraftingRecipeTypes, RecipeIngredientTypes};
            use pumpkin_protocol::bedrock::client::{
                BedrockRecipe, BedrockShapedRecipe, BedrockShapelessRecipe, ItemDescriptorCount,
                RecipeUnlockRequirement,
            };
            use pumpkin_protocol::bedrock::network_item::NetworkItemDescriptor;
            use pumpkin_protocol::codec::{var_int::VarInt, var_uint::VarUInt};

            let mut mapped_recipes = Vec::new();
            let mut network_id_counter = 1u32;

            for recipe in pumpkin_data::recipes::RECIPES_CRAFTING {
                let map_ingredient = |ing: &RecipeIngredientTypes| -> ItemDescriptorCount {
                    let item_key = match ing {
                        RecipeIngredientTypes::Simple(name) => Some(*name),
                        RecipeIngredientTypes::Tagged(tag) => {
                            let tag_name = tag.strip_prefix('#').unwrap_or(tag);
                            pumpkin_data::tag::get_tag_ids(
                                pumpkin_data::tag::RegistryKey::Item,
                                tag_name,
                            )
                            .and_then(|ids| {
                                ids.first().and_then(|&first_id| {
                                    Item::from_id(first_id).map(|item| item.registry_key)
                                })
                            })
                        }
                        RecipeIngredientTypes::OneOf(names) => names.first().copied(),
                    };

                    if let Some(key) = item_key {
                        let registry_key = key.strip_prefix("minecraft:").unwrap_or(key);
                        if let Some(item) = Item::from_registry_key(registry_key)
                            && let Some(mapping) =
                                JavaToBedrockItemMapping::from_java_item_id(item.id)
                        {
                            return ItemDescriptorCount {
                                network_id: mapping.bedrock_item.id,
                                metadata_value: mapping.bedrock_data as i16,
                                count: 1,
                            };
                        }
                    }

                    ItemDescriptorCount {
                        network_id: 0,
                        metadata_value: 0,
                        count: 0,
                    }
                };

                match recipe {
                    CraftingRecipeTypes::CraftingShaped {
                        category: _,
                        group: _,
                        show_notification: _,
                        key,
                        pattern,
                        result,
                    } => {
                        let height = pattern.len() as i32;
                        let width = pattern.iter().map(|s| s.len()).max().unwrap_or(0) as i32;

                        let mut input = Vec::new();
                        for r in 0..height {
                            let pattern_row = pattern[r as usize];
                            for c in 0..width {
                                let ch = pattern_row.chars().nth(c as usize).unwrap_or(' ');
                                if ch == ' ' {
                                    input.push(ItemDescriptorCount {
                                        network_id: 0,
                                        metadata_value: 0,
                                        count: 0,
                                    });
                                } else {
                                    let mut ingredient = None;
                                    for &(key_ch, ref ing) in *key {
                                        if key_ch == ch {
                                            ingredient = Some(ing);
                                            break;
                                        }
                                    }
                                    if let Some(ing) = ingredient {
                                        input.push(map_ingredient(ing));
                                    } else {
                                        input.push(ItemDescriptorCount {
                                            network_id: 0,
                                            metadata_value: 0,
                                            count: 0,
                                        });
                                    }
                                }
                            }
                        }

                        let output_item = Item::from_registry_key(result.id);
                        if let Some(item) = output_item
                            && let Some(mapping) =
                                JavaToBedrockItemMapping::from_java_item_id(item.id)
                        {
                            let output_descriptor = NetworkItemDescriptor {
                                id: VarInt::from(mapping.bedrock_item.id),
                                stack_size: result.count as u16,
                                aux_value: VarUInt(mapping.bedrock_data),
                                block_runtime_id: VarInt::from(mapping.bedrock_block_state),
                                nbt_data: pumpkin_nbt::Nbt::default(),
                                place_on_blocks: Vec::new(),
                                destroy_blocks: Vec::new(),
                                shield_blocking_tick: 0,
                            };

                            mapped_recipes.push(BedrockRecipe::Shaped(BedrockShapedRecipe {
                                recipe_id: format!("pumpkin:recipe_{network_id_counter}"),
                                width: VarInt(width),
                                height: VarInt(height),
                                input,
                                output: vec![output_descriptor],
                                uuid: [0; 16],
                                block: "crafting_table".to_string(),
                                priority: VarInt(1),
                                assume_symmetry: true,
                                unlock_requirement: RecipeUnlockRequirement { context: 1 },
                                recipe_network_id: VarUInt(network_id_counter),
                            }));
                            network_id_counter += 1;
                        }
                    }
                    CraftingRecipeTypes::CraftingShapeless {
                        category: _,
                        group: _,
                        ingredients,
                        result,
                    } => {
                        let input = ingredients.iter().map(map_ingredient).collect::<Vec<_>>();

                        let output_item = Item::from_registry_key(result.id);
                        if let Some(item) = output_item
                            && let Some(mapping) =
                                JavaToBedrockItemMapping::from_java_item_id(item.id)
                        {
                            let output_descriptor = NetworkItemDescriptor {
                                id: VarInt::from(mapping.bedrock_item.id),
                                stack_size: result.count as u16,
                                aux_value: VarUInt(mapping.bedrock_data),
                                block_runtime_id: VarInt::from(mapping.bedrock_block_state),
                                nbt_data: pumpkin_nbt::Nbt::default(),
                                place_on_blocks: Vec::new(),
                                destroy_blocks: Vec::new(),
                                shield_blocking_tick: 0,
                            };

                            mapped_recipes.push(BedrockRecipe::Shapeless(BedrockShapelessRecipe {
                                recipe_id: format!("pumpkin:recipe_{network_id_counter}"),
                                input,
                                output: vec![output_descriptor],
                                uuid: [0; 16],
                                block: "crafting_table".to_string(),
                                priority: VarInt(1),
                                unlock_requirement: RecipeUnlockRequirement { context: 1 },
                                recipe_network_id: VarUInt(network_id_counter),
                            }));
                            network_id_counter += 1;
                        }
                    }
                    _ => {}
                }
            }
            mapped_recipes
        });

        client
            .send_game_packet(&pumpkin_protocol::bedrock::client::CCraftingData {
                recipes: bedrock_recipes.clone(),
                clean_recipes: false,
            })
            .await;

        client
            .send_game_packet(&CInventoryContent {
                container_id: VarUInt(0), // player inventory,
                slots: futures::future::join_all(player.inventory.main_inventory.iter().map(
                    async |s| {
                        let stack = s.lock().await;

                        NetworkItemStackDescriptor::from(&*stack)
                    },
                ))
                .await,
                full_container_name: FullContainerName {
                    container_name: ContainerName::Inventory,
                    dynamic_id: None,
                },
                storage_item: NetworkItemStackDescriptor::default(),
            })
            .await;

        {
            let mut abilities = player.abilities.lock().await;
            abilities.set_for_gamemode(player.gamemode.load());
        };

        let entity = &player.get_entity();
        let metadata = entity.bedrock_metadata();

        let actor_data = CSetActorData {
            actor_runtime_id: VarULong(runtime_id),
            metadata,
            synced_properties: PropertySyncData {
                int_properties: HashMap::new(),
                float_properties: HashMap::new(),
            },
            tick: VarULong(0),
        };
        client.send_game_packet(&actor_data).await;
        player.send_abilities_update().await;

        {
            let command_dispatcher = server.command_dispatcher.read().await;
            client_suggestions::send_bedrock_commands_packet(&player, server, &command_dispatcher)
                .await;
        };

        client
            .enqueue_packet_internal(&CUpdateAttributes {
                runtime_id: VarULong(runtime_id),
                attributes: vec![
                    Attribute {
                        min_value: 0.0,
                        max_value: 3.402_823_5E38,
                        current_value: 0.1,
                        default_min_value: 0.0,
                        default_max_value: 3.402_823_5E38,
                        default_value: 0.1,
                        name: "minecraft:movement".to_string(),
                        modifiers_list_size: VarUInt(0),
                    },
                    Attribute {
                        min_value: 0.0,
                        max_value: 3.402_823_5E38,
                        current_value: 0.02,
                        default_min_value: 0.0,
                        default_max_value: 3.402_823_5E38,
                        default_value: 0.02,
                        name: "minecraft:underwater_movement".to_string(),
                        modifiers_list_size: VarUInt(0),
                    },
                    Attribute {
                        min_value: 0.0,
                        max_value: 1.0,
                        current_value: 0.08,
                        default_min_value: 0.0,
                        default_max_value: 1.0,
                        default_value: 0.08,
                        name: "minecraft:gravity".to_string(),
                        modifiers_list_size: VarUInt(0),
                    },
                    Attribute {
                        min_value: 0.0,
                        max_value: 400.0,
                        current_value: 400.0,
                        default_min_value: 0.0,
                        default_max_value: 400.0,
                        default_value: 400.0,
                        name: "minecraft:air".to_string(),
                        modifiers_list_size: VarUInt(0),
                    },
                    Attribute {
                        min_value: 0.0,
                        max_value: 20.0,
                        current_value: 20.0,
                        default_min_value: 0.0,
                        default_max_value: 20.0,
                        default_value: 20.0,
                        name: "minecraft:health".to_string(),
                        modifiers_list_size: VarUInt(0),
                    },
                    Attribute {
                        min_value: 0.0,
                        max_value: 20.0,
                        current_value: 20.0,
                        default_min_value: 0.0,
                        default_max_value: 20.0,
                        default_value: 20.0,
                        name: "minecraft:player.hunger".to_string(),
                        modifiers_list_size: VarUInt(0),
                    },
                ],
                player_tick: VarULong(0),
            })
            .await;

        // --- MULTIPLAYER BROADCASTING ---

        let gameprofile = &player.gameprofile;
        let velocity = player.get_entity().velocity.load();

        // 1. Broadcast the new Bedrock player to everyone else (Java + Bedrock)
        let bedrock_player_list = CPlayerList {
            action: CPlayerList::ACTION_ADD,
            entries: vec![PlayerListEntry {
                uuid: gameprofile.id,
                entity_unique_id: VarLong(runtime_id as i64),
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

        let gamemode = player.gamemode.load();
        self.broadcast_packet_except_editioned_sync(
            &[gameprofile.id],
            &CPlayerInfoUpdate::new(
                (PlayerInfoFlags::ADD_PLAYER
                    | PlayerInfoFlags::UPDATE_GAME_MODE
                    | PlayerInfoFlags::UPDATE_LISTED
                    | PlayerInfoFlags::UPDATE_LATENCY
                    | PlayerInfoFlags::UPDATE_LIST_PRIORITY)
                    .bits(),
                &[pumpkin_protocol::java::client::play::Player {
                    uuid: gameprofile.id,
                    actions: &[
                        PlayerAction::AddPlayer {
                            name: &gameprofile.name,
                            properties: &gameprofile.properties.load(),
                        },
                        PlayerAction::UpdateGameMode(VarInt(gamemode as i32)),
                        PlayerAction::UpdateListed(true),
                        PlayerAction::UpdateLatency(VarInt(0)),
                        PlayerAction::UpdateListOrder(VarInt(0)),
                    ],
                }],
            ),
            &bedrock_player_list,
        );

        let bedrock_add_player = CAddPlayer {
            uuid: gameprofile.id,
            username: gameprofile.name.clone(),
            entity_runtime_id: VarULong(runtime_id),
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
            metadata: entity.bedrock_metadata(),
            properties: EntityProperties::default(),
            ability_data: pumpkin_protocol::bedrock::client::add_player::AbilityData {
                entity_unique_id: runtime_id as i64,
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

        self.broadcast_packet_except_editioned_sync(
            &[gameprofile.id],
            &CSpawnEntity::new(
                (runtime_id as i32).into(),
                gameprofile.id,
                i32::from(EntityType::PLAYER.id).into(),
                position,
                pitch,
                yaw,
                yaw,
                0.into(),
                velocity,
            ),
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
            &CSetEntityMetadata::new((runtime_id as i32).into(), java_meta_buf.into()),
            &actor_data,
        );

        // 2. Spawn existing players for our new Bedrock client
        let players = self.players.load();

        for existing_player in players
            .iter()
            .filter(|p| p.gameprofile.id != gameprofile.id)
        {
            let ex_profile = &existing_player.gameprofile;
            let ex_entity = &existing_player.get_entity();
            let ex_pos = ex_entity.pos.load();
            let ex_vel = ex_entity.velocity.load();

            let ex_player_list = CPlayerList {
                action: CPlayerList::ACTION_ADD,
                entries: vec![PlayerListEntry {
                    uuid: ex_profile.id,
                    entity_unique_id: VarLong(existing_player.entity_id() as i64),
                    username: ex_profile.name.clone(),
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
            // Send PlayerList FIRST
            client.send_game_packet(&ex_player_list).await;

            let ex_add_player = CAddPlayer {
                uuid: ex_profile.id,
                username: ex_profile.name.clone(),
                entity_runtime_id: VarULong(existing_player.entity_id() as u64),
                platform_chat_id: String::new(),
                position: Vector3::new(ex_pos.x as f32, ex_pos.y as f32, ex_pos.z as f32),
                velocity: Vector3::new(ex_vel.x as f32, ex_vel.y as f32, ex_vel.z as f32),
                pitch: ex_entity.pitch.load(),
                yaw: ex_entity.yaw.load(),
                head_yaw: ex_entity.head_yaw.load(),
                held_item: NetworkItemDescriptor::default(),
                game_mode: VarInt(match existing_player.gamemode.load() {
                    GameMode::Survival => 0,
                    GameMode::Creative => 1,
                    GameMode::Adventure => 2,
                    GameMode::Spectator => 6,
                }),
                metadata: ex_entity.bedrock_metadata(),
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

            client.send_game_packet(&ex_add_player).await;
        }

        // 3. Trigger Join Event and Broadcast Join Message
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
            info!("{}", event.join_message.to_pretty_console());
        }
    }
}
