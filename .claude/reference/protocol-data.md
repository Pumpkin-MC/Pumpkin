# Protocol Data Reference (1.21.4)

> Extracted 2026-02-07 from Pumpkin spec data + codebase analysis.
> Sources: `registries.json`, `commands.json`, `pumpkin-protocol/src/`

---

## Registry Names + Entry Counts

| Registry | Entries |
|---|---|
| activity | 26 |
| advancement | 1481 |
| atlas | 14 |
| attribute | 32 |
| banner_pattern | 43 |
| block | 1095 |
| block_definition | 1097 |
| block_entity_type | 45 |
| block_predicate_type | 13 |
| block_type | 245 |
| cat_variant | 11 |
| chat_type | 7 |
| chunk_status | 12 |
| command_argument_type | 54 |
| consume_effect_type | 5 |
| creative_mode_tab | 14 |
| custom_stat | 75 |
| damage_type | 49 |
| data_component_type | 67 |
| datapack | 3 |
| decorated_pot_pattern | 24 |
| dimension | 3 |
| dimension_type | 4 |
| enchantment | 42 |
| enchantment_effect_component_type | 30 |
| enchantment_entity_effect_type | 13 |
| enchantment_level_based_value_type | 5 |
| enchantment_location_based_effect_type | 14 |
| enchantment_provider | 7 |
| enchantment_provider_type | 3 |
| enchantment_value_effect_type | 5 |
| entity_sub_predicate_type | 20 |
| entity_type | 149 |
| equipment | 26 |
| float_provider_type | 4 |
| fluid | 5 |
| font | 7 |
| frog_variant | 3 |
| game_event | 60 |
| height_provider_type | 6 |
| instrument | 8 |
| int_provider_type | 6 |
| item | 1385 |
| item_definition | 1385 |
| item_sub_predicate_type | 14 |
| jukebox_song | 19 |
| lang | 133 |
| loot_condition_type | 19 |
| loot_function_type | 40 |
| loot_nbt_provider_type | 2 |
| loot_number_provider_type | 6 |
| loot_pool_entry_type | 8 |
| loot_score_provider_type | 2 |
| loot_table | 1237 |
| map_decoration_type | 35 |
| memory_module_type | 106 |
| menu | 25 |
| mob_effect | 39 |
| model | 3289 |
| number_format_type | 3 |
| painting_variant | 50 |
| particle_type | 112 |
| point_of_interest_type | 20 |
| pos_rule_test | 3 |
| position_source_type | 2 |
| post_effect | 6 |
| potion | 46 |
| recipe | 1370 |
| recipe_book_category | 13 |
| recipe_display | 5 |
| recipe_serializer | 22 |
| recipe_type | 7 |
| resourcepack | 2 |
| rule_block_entity_modifier | 4 |
| rule_test | 6 |
| schedule | 4 |
| sensor_type | 26 |
| slot_display | 8 |
| sound | 3855 |
| sound_event | 1651 |
| stat_type | 9 |
| structure | 1201 |
| trim_material | 11 |
| trim_pattern | 18 |
| trigger_type | 56 |
| trial_spawner | 28 |
| villager_profession | 15 |
| villager_type | 7 |
| wolf_variant | 9 |

**Tag registries** (selected):

| Tag Registry | Tags |
|---|---|
| tag/block | 186 |
| tag/item | 173 |
| tag/entity_type | 35 |
| tag/damage_type | 33 |
| tag/enchantment | 22 |
| tag/banner_pattern | 11 |
| tag/worldgen/biome | 70 |
| tag/worldgen/structure | 13 |
| tag/fluid | 2 |
| tag/game_event | 5 |

**Worldgen registries** (selected):

| Worldgen Registry | Entries |
|---|---|
| worldgen/biome | 65 |
| worldgen/configured_feature | 204 |
| worldgen/placed_feature | 238 |
| worldgen/template_pool | 188 |
| worldgen/structure | 34 |
| worldgen/structure_set | 20 |
| worldgen/noise_settings | 7 |
| worldgen/noise | 60 |
| worldgen/density_function | 35 |
| worldgen/processor_list | 40 |
| worldgen/feature | 62 |

**Key numbers**: 1095 blocks, 1385 items, 149 entity types, 42 enchantments, 39 mob effects, 3855 sounds, 1651 sound events, 1237 loot tables, 1370 recipes.

---

## Command Tree (Top-Level Vanilla Commands)

All 83 commands from `commands.json`:

```
advancement    attribute      ban            ban-ip         banlist
bossbar        clear          clone          damage         data
datapack       debug          defaultgamemode deop          difficulty
effect         enchant        execute        experience     fill
fillbiome      forceload      function       gamemode       gamerule
give           help           item           jfr            kick
kill           list           locate         loot           me
msg            op             pardon         pardon-ip      particle
perf           place          playsound      publish        random
recipe         reload         return         ride           rotate
save-all       save-off       save-on        say            schedule
scoreboard     seed           setblock       setidletimeout setworldspawn
spawnpoint     spectate       spreadplayers  stop           stopsound
summon         tag            team           teammsg        teleport
tell           tellraw        tick           time           title
tm             tp             transfer       trigger        w
weather        whitelist      worldborder    xp
```

Note: `tell`, `w`, `msg` are aliases. `tp` and `teleport` are aliases. `tm` and `teammsg` are aliases. `xp` and `experience` are aliases.

---

## Packet Coverage

### Largest Protocol Source Files (by line count)

| File | Lines |
|---|---|
| **total** | **14,722** |
| ser/mod.rs | 705 |
| java/packet_encoder.rs | 669 |
| lib.rs | 565 |
| ser/serializer.rs | 533 |
| java/packet_decoder.rs | 408 |
| bedrock/client/set_actor_data.rs | 398 |
| query.rs | 362 |
| codec/var_int.rs | 351 |
| java/client/play/commands.rs | 342 |
| ser/deserializer.rs | 316 |
| codec/item_stack_seralizer.rs | 307 |
| codec/data_component.rs | 289 |
| bedrock/server/text.rs | 249 |
| codec/var_long.rs | 244 |
| codec/var_uint.rs | 228 |
| serial/deserializer.rs | 219 |
| codec/var_ulong.rs | 197 |
| java/client/play/chunk_data.rs | 195 |
| serial/serializer.rs | 188 |

### Packet File Counts by Phase (Java Edition)

| Direction | Phase | Files (incl. mod.rs) |
|---|---|---|
| Client (S->C) | play | 91 |
| Client (S->C) | config | 12 |
| Client (S->C) | login | 7 |
| Client (S->C) | status | 3 |
| Server (C->S) | play | 35 |
| Server (C->S) | config | 7 |
| Server (C->S) | login | 6 |
| Server (C->S) | status | 3 |
| Server (C->S) | handshake | 1 |

**Total Java packet files**: ~165 (including mod.rs per directory)

### Client-bound Play Packets (91 files)

Includes: acknowledge_block, actionbar, block_destroy_stage, block_entity_data, block_event,
block_update, boss_event, center_chunk, change_difficulty, chunk_batch_end/start, chunk_data,
clear_title, close_container, combat_death, command_suggestions, commands, cookie_request,
damage_event, disconnect, disguised_chat_message, display_objective, entity_animation,
entity_metadata, entity_position_sync, entity_sound_effect, entity_status, entity_velocity,
explode, game_event, head_rot, hurt_animation, initialize_world_border, keep_alive,
level_event, login, multi_block_update, open_screen, open_sign_editor, particle,
ping_response, player_abilities, player_action, player_chat_message, player_info_update,
player_position, player_remove, player_spawn_position, remove_entities, remove_mob_effect,
reset_score, respawn, server_links, set_border_*, set_container_*, set_cursor_slot,
set_equipment, set_experience, set_health, set_held_item, set_player_inventory, set_time,
set_title, set_title_animation, sound_effect, spawn_entity, stop_sound, store_cookie,
subtitle, system_chat_message, take_item, teleport_entity, ticking_state, ticking_step,
transfer, unload_chunk, update_entity_pos/rot, update_mob_effect, update_objectives,
update_score, worldevent

### Server-bound Play Packets (35 files)

Includes: change_game_mode, chat_command, chat_message, chunk_batch, click_container,
client_command, client_information, client_tick_end, close_container, command_suggestion,
confirm_teleport, cookie_response, custom_payload, interact, keep_alive, pick_item,
ping_request, player_abilities, player_action, player_command, player_ground, player_input,
player_loaded, player_position, player_position_rotation, player_rotation, player_session,
set_command_block, set_creative_slot, set_held_item, swing_arm, update_sign, use_item,
use_item_on

---

## Gap Analysis

### Packets Not Yet Implemented (Notable Missing from Vanilla Protocol)

Comparing the ~91 client-bound + ~35 server-bound play packets against the full 1.21.4 vanilla protocol:

**Missing client-bound (S->C) packets** (estimated, based on wiki.vg for 1.21.4):
- Map data packets (map rendering)
- Tab-complete response variants
- Merchant offers / trade list
- Player combat begin/end (non-death events)
- Scoreboard team advanced operations
- Recipe-related packets (recipe book, unlocked recipes)
- Set passengers
- Set camera
- NBT query response
- Select advancements tab
- Resource pack push/status
- Vibration signal
- Bundle delimiter

**Missing server-bound (C->S) packets** (estimated):
- Resource pack status
- Advancement tab
- Entity NBT query
- Spectate teleport
- Lock difficulty
- Recipe book seen/displayed
- Steer vehicle / boat input variants
- Edit book
- Jigsaw/structure block update
- Bundle interactions

**Coverage estimate**: ~65-70% of vanilla play-phase packets are present. Login, status, config, and handshake phases appear reasonably complete.

### Registry Coverage

Pumpkin loads and serves registries via data-driven JSON. The key registries (block, item, entity_type, dimension_type, biome, damage_type, chat_type) are wired. Many worldgen registries exist in spec data but worldgen is partially delegated to pumpkin-world.

### Overall Protocol Health

- Total protocol crate: 14,722 lines across all .rs files
- Both Java and Bedrock protocol paths exist (Bedrock is early/experimental)
- Serialization framework (ser/ + serial/ + codec/) is substantial (~3,000+ lines)
- Packet encoder/decoder pipeline is mature (1,077 lines combined)
