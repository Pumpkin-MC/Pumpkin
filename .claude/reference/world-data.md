# MC 1.21.4 | World Agent Reference

> Comprehensive data reference for the World agent. Extracted from Pumpkin spec data.
> Generated: 2026-02-07

---

## Block Type Categories

**1,095 total blocks** across **245 unique types** in `block_definitions.json`.

### Multi-Instance Types (83 types, 933 blocks)

| Type | Count | Type | Count |
|------|------:|------|------:|
| block | 172 | slab | 58 |
| rotated_pillar | 56 | stair | 54 |
| flower_pot | 38 | wall | 26 |
| door | 17 | trapdoor | 17 |
| candle | 17 | candle_cake | 17 |
| shulker_box | 17 | banner | 16 |
| bed | 16 | wool_carpet | 16 |
| concrete_powder | 16 | glazed_terracotta | 16 |
| stained_glass | 16 | stained_glass_pane | 16 |
| wall_banner | 16 | drop_experience | 16 |
| button | 14 | pressure_plate | 14 |
| fence | 13 | flower | 13 |
| fence_gate | 12 | ceiling_hanging_sign | 12 |
| standing_sign | 12 | wall_hanging_sign | 12 |
| wall_sign | 12 | weathering_copper_full | 12 |
| leaves | 8 | sapling | 8 |
| infested | 6 | coral_plant | 5 |
| coral | 5 | coral_fan | 5 |
| coral_wall_fan | 5 | skull | 5 |
| base_coral_plant | 5 | base_coral_fan | 5 |
| base_coral_wall_fan | 5 | amethyst_cluster | 4 |
| weathering_copper_bulb | 4 | weathering_copper_door | 4 |
| weathering_copper_grate | 4 | weathering_copper_trap_door | 4 |
| wall_skull | 4 | weathering_copper_slab | 4 |
| weathering_copper_stair | 4 | tall_flower | 4 |
| copper_bulb_block | 4 | waterlogged_transparent | 4 |
| air | 3 | anvil | 3 |
| huge_mushroom | 3 | command | 3 |
| colored_falling | 3 | double_plant | 3 |
| powered_rail | 2 | attached_stem | 2 |
| azalea | 2 | beehive | 2 |
| mushroom | 2 | campfire | 2 |
| jack_o_lantern | 2 | particle_leaves | 2 |
| eyeblossom | 2 | fungus | 2 |
| nylium | 2 | roots | 2 |
| redstone_ore | 2 | tall_grass | 2 |
| iron_bars | 2 | weighted_pressure_plate | 2 |
| lantern | 2 | liquid | 2 |
| stem | 2 | bonemealable_feature_placer | 2 |
| piston_base | 2 | layered_cauldron | 2 |
| torch | 2 | wall_torch | 2 |
| brushable | 2 | | |

### Single-Instance Types (162 types)

<details><summary>Click to expand all 162 singleton types</summary>

amethyst, bamboo_sapling, bamboo_stalk, barrel, barrier, beacon, beetroot, bell, big_dripleaf, big_dripleaf_stem, blast_furnace, brewing_stand, bubble_column, budding_amethyst, cactus, cake, calibrated_sculk_sensor, carpet, carrot, cartography_table, cauldron, cave_vines, cave_vines_plant, chain, chest, chiseled_book_shelf, chorus_flower, chorus_plant, cocoa, comparator, composter, conduit, crafter, crafting_table, creaking_heart, crop, crying_obsidian, daylight_detector, dead_bush, decorated_pot, detector_rail, dirt_path, dispenser, dragon_egg, dropper, enchantment_table, end_gateway, end_portal, end_portal_frame, end_rod, ender_chest, farm, fire, fletching_table, frogspawn, frosted_ice, furnace, glow_lichen, grass, grindstone, half_transparent, hanging_moss, hanging_roots, hay, heavy_core, honey, hopper, ice, infested_rotated_pillar, jigsaw, jukebox, kelp, kelp_plant, ladder, lava_cauldron, lectern, lever, light, lightning_rod, loom, magma, mangrove_leaves, mangrove_propagule, mangrove_roots, mossy_carpet, moving_piston, mud, multiface, mycelium, nether_portal, nether_sprouts, nether_wart, netherrack, note, observer, piglinwallskull, pink_petals, piston_head, pitcher_crop, player_head, player_wall_head, pointed_dripstone, potato, powder_snow, powered, pumpkin, rail, redstone_lamp, redstone_torch, redstone_wall_torch, redstone_wire, repeater, respawn_anchor, rooted_dirt, scaffolding, sculk, sculk_catalyst, sculk_sensor, sculk_shrieker, sculk_vein, sea_pickle, seagrass, slime, small_dripleaf, smithing_table, smoker, sniffer_egg, snow_layer, snowy_dirt, soul_fire, soul_sand, spawner, sponge, spore_blossom, stonecutter, structure, structure_void, sugar_cane, sweet_berry_bush, tall_seagrass, target, tinted_glass, tnt, torchflower_crop, transparent, trapped_chest, trial_spawner, trip_wire_hook, tripwire, turtle_egg, twisting_vines, twisting_vines_plant, vault, vine, waterlily, web, weeping_vines, weeping_vines_plant, wet_sponge, wither_rose, wither_skull, wither_wall_skull

</details>

---

## Mining Speeds

From `prismarine/materials.json`. Item IDs decoded to tool names.

**Tool ID Mapping:**

| ID | Tool | ID | Tool |
|----|------|----|------|
| 850 | wooden_shovel | 851 | wooden_pickaxe |
| 852 | wooden_axe | 853 | wooden_hoe |
| 855 | stone_shovel | 856 | stone_pickaxe |
| 857 | stone_axe | 858 | stone_hoe |
| 860 | golden_shovel | 861 | golden_pickaxe |
| 862 | golden_axe | 863 | golden_hoe |
| 865 | iron_shovel | 866 | iron_pickaxe |
| 867 | iron_axe | 868 | iron_hoe |
| 870 | diamond_shovel | 871 | diamond_pickaxe |
| 872 | diamond_axe | 873 | diamond_hoe |
| 875 | netherite_shovel | 876 | netherite_pickaxe |
| 877 | netherite_axe | 878 | netherite_hoe |
| 1032 | shears | | |

### Base Material Speed Table

| Material | Tool Multipliers |
|----------|-----------------|
| `default` | _(no multipliers -- fist speed)_ |
| `leaves` | shears: 15.0 |
| `coweb` | shears: 15.0 |
| `plant` | _(no multipliers)_ |
| `gourd` | _(no multipliers)_ |
| `vine_or_glow_lichen` | shears: 2.0 |
| `wool` | shears: 5.0 |

### Tool-Tier Speed Table (mineable categories)

Speed multiplier for each tool tier. Applies to `mineable/shovel`, `mineable/pickaxe`, `mineable/axe`, `mineable/hoe` respectively.

| Tier | Shovel | Pickaxe | Axe | Hoe |
|------|-------:|--------:|----:|----:|
| Wooden | 2.0 | 2.0 | 2.0 | 2.0 |
| Stone | 4.0 | 4.0 | 4.0 | 4.0 |
| Golden | 12.0 | 12.0 | 12.0 | 12.0 |
| Iron | 6.0 | 6.0 | 6.0 | 6.0 |
| Diamond | 8.0 | 8.0 | 8.0 | 8.0 |
| Netherite | 9.0 | 9.0 | 9.0 | 9.0 |

### Incorrect Tool Tier Restrictions

Blocks tagged with these materials cannot be effectively mined by the named tier (drops nothing).

| Restriction Tag | Tier Speed Applied |
|-----------------|---:|
| `incorrect_for_wooden_tool` | wooden_{shovel,pickaxe,axe,hoe}: 1.0 |
| `incorrect_for_stone_tool` | stone_{shovel,pickaxe,axe,hoe}: 4.0 |
| `incorrect_for_gold_tool` | golden_{shovel,pickaxe,axe,hoe}: 12.0 |
| `incorrect_for_iron_tool` | iron_{shovel,pickaxe,axe,hoe}: 6.0 |
| `incorrect_for_diamond_tool` | diamond_{shovel,pickaxe,axe,hoe}: 8.0 |
| `incorrect_for_netherite_tool` | netherite_{shovel,pickaxe,axe,hoe}: 9.0 |

### Compound Materials

| Compound Material | Tool Multipliers |
|-------------------|-----------------|
| `plant;mineable/axe` | all axe tiers: 1.0 |
| `gourd;mineable/axe` | all axe tiers: 1.0 |
| `leaves;mineable/hoe` | shears: 15.0, all hoe tiers: 1.0 |
| `leaves;mineable/axe;mineable/hoe` | shears: 15.0, all axe tiers: 1.0, all hoe tiers: 1.0 |
| `vine_or_glow_lichen;plant;mineable/axe` | shears: 2.0, all axe tiers: 1.0 |

---

## Worldgen Data Inventory

File counts per category from `mcdata-1.21.4.zip`:

| Category | File Count | Description |
|----------|----------:|-------------|
| template_pool | 245 | Jigsaw structure pools (villages, bastions, etc.) |
| placed_feature | 239 | Feature placement rules (position, count, conditions) |
| configured_feature | 205 | Feature configurations (trees, ores, patches) |
| biome | 138 | Biome definitions (climate, spawns, features) |
| noise | 61 | Noise parameter definitions |
| structure | 49 | Structure definitions (villages, temples, etc.) |
| density_function | 42 | Terrain shape density functions |
| processor_list | 41 | Block processors for structure gen |
| structure_set | 21 | Structure placement sets |
| flat_level_generator_preset | 12 | Superflat presets |
| world_preset | 10 | World type presets |
| noise_settings | 8 | Noise generator settings (overworld, nether, end, etc.) |
| configured_carver | 5 | Cave/canyon carver configs |
| multi_noise_biome_source_parameter_list | 3 | Multi-noise biome parameter lists |
| dimension | 4 | Dimension definitions |
| **TOTAL** | **1,083** | |

---

## Sample Biome Definition: `plains.json`

```json
{
  "carvers": [
    "minecraft:cave",
    "minecraft:cave_extra_underground",
    "minecraft:canyon"
  ],
  "downfall": 0.4,
  "effects": {
    "fog_color": 12638463,
    "mood_sound": {
      "block_search_extent": 8,
      "offset": 2.0,
      "sound": "minecraft:ambient.cave",
      "tick_delay": 6000
    },
    "music_volume": 1.0,
    "sky_color": 7907327,
    "water_color": 4159204,
    "water_fog_color": 329011
  },
  "features": [
    [],
    ["minecraft:lake_lava_underground", "minecraft:lake_lava_surface"],
    ["minecraft:amethyst_geode"],
    ["minecraft:monster_room", "minecraft:monster_room_deep"],
    [], [],
    [
      "minecraft:ore_dirt", "minecraft:ore_gravel",
      "minecraft:ore_granite_upper", "minecraft:ore_granite_lower",
      "minecraft:ore_diorite_upper", "minecraft:ore_diorite_lower",
      "minecraft:ore_andesite_upper", "minecraft:ore_andesite_lower",
      "minecraft:ore_tuff", "minecraft:ore_coal_upper", "minecraft:ore_coal_lower",
      "minecraft:ore_iron_upper", "minecraft:ore_iron_middle", "minecraft:ore_iron_small",
      "minecraft:ore_gold", "minecraft:ore_gold_lower",
      "minecraft:ore_redstone", "minecraft:ore_redstone_lower",
      "minecraft:ore_diamond", "minecraft:ore_diamond_medium",
      "minecraft:ore_diamond_large", "minecraft:ore_diamond_buried",
      "minecraft:ore_lapis", "minecraft:ore_lapis_buried",
      "minecraft:ore_copper", "minecraft:underwater_magma",
      "minecraft:disk_sand", "minecraft:disk_clay", "minecraft:disk_gravel"
    ],
    [],
    ["minecraft:spring_water", "minecraft:spring_lava"],
    [
      "minecraft:glow_lichen", "minecraft:patch_tall_grass_2",
      "minecraft:trees_plains", "minecraft:flower_plains",
      "minecraft:patch_grass_plain", "minecraft:brown_mushroom_normal",
      "minecraft:red_mushroom_normal", "minecraft:patch_sugar_cane",
      "minecraft:patch_pumpkin"
    ],
    ["minecraft:freeze_top_layer"]
  ],
  "has_precipitation": true,
  "temperature": 0.8,
  "spawners": {
    "creature": [
      { "type": "minecraft:sheep",   "weight": 12, "minCount": 4, "maxCount": 4 },
      { "type": "minecraft:pig",     "weight": 10, "minCount": 4, "maxCount": 4 },
      { "type": "minecraft:chicken", "weight": 10, "minCount": 4, "maxCount": 4 },
      { "type": "minecraft:cow",     "weight": 8,  "minCount": 4, "maxCount": 4 },
      { "type": "minecraft:horse",   "weight": 5,  "minCount": 2, "maxCount": 6 },
      { "type": "minecraft:donkey",  "weight": 1,  "minCount": 1, "maxCount": 3 }
    ],
    "monster": [
      { "type": "minecraft:spider",          "weight": 100, "minCount": 4, "maxCount": 4 },
      { "type": "minecraft:zombie",          "weight": 95,  "minCount": 4, "maxCount": 4 },
      { "type": "minecraft:zombie_villager", "weight": 5,   "minCount": 1, "maxCount": 1 },
      { "type": "minecraft:skeleton",        "weight": 100, "minCount": 4, "maxCount": 4 },
      { "type": "minecraft:creeper",         "weight": 100, "minCount": 4, "maxCount": 4 },
      { "type": "minecraft:slime",           "weight": 100, "minCount": 4, "maxCount": 4 },
      { "type": "minecraft:enderman",        "weight": 10,  "minCount": 1, "maxCount": 4 },
      { "type": "minecraft:witch",           "weight": 5,   "minCount": 1, "maxCount": 1 }
    ],
    "ambient": [
      { "type": "minecraft:bat", "weight": 10, "minCount": 8, "maxCount": 8 }
    ],
    "underground_water_creature": [
      { "type": "minecraft:glow_squid", "weight": 10, "minCount": 4, "maxCount": 6 }
    ],
    "water_creature": [], "water_ambient": [], "axolotls": [], "misc": []
  },
  "spawn_costs": {}
}
```

**Biome Structure Notes:**
- `features` is an array of 11 generation steps (indexed 0-10)
- Steps: raw_generation, lakes, local_modifications, underground_structures, surface_structures, strongholds, underground_ores, underground_decoration, fluid_springs, vegetal_decoration, top_layer_modification
- `spawners` has 9 categories: creature, monster, ambient, axolotls, underground_water_creature, water_creature, water_ambient, misc
- `effects` controls rendering: fog, sky, water colors, mood sounds, music

---

## Current Pumpkin Implementation

### pumpkin-world/src/ (216 files, 38,396 lines)

| File / Module | Lines | Purpose |
|--------------|------:|---------|
| `chunk_system.rs` | 2,189 | Chunk loading/management system |
| `generation/noise/aquifer_sampler.rs` | 2,664 | Aquifer noise sampling |
| `chunk/format/anvil.rs` | 1,324 | Anvil format read/write |
| `generation/proto_chunk.rs` | 1,247 | Proto-chunk during generation |
| `level.rs` | 810 | Level management |
| `generation/proto_chunk_test.rs` | 680 | Proto-chunk tests |
| `generation/noise/router/density_function/test.rs` | 827 | Density function tests |
| `generation/noise/router/chunk_density_function.rs` | 736 | Chunk-level density functions |
| `generation/noise/router/chunk_noise_router.rs` | 623 | Chunk noise router |
| `block/entities/furnace_like_block_entity.rs` | 613 | Furnace block entity |
| `chunk/palette.rs` | 606 | Block palette encoding |
| `chunk/mod.rs` | 611 | Chunk data structures |
| `generation/feature/placed_features.rs` | 572 | Placed feature deserialization |
| `poi/mod.rs` | 567 | Points of interest |
| `generation/structure/structures/mod.rs` | 558 | Structure generation dispatch |
| `chunk/format/linear.rs` | 548 | Linear format read/write |
| `chunk/format/mod.rs` | 517 | Chunk format abstraction |
| `item/mod.rs` | 480 | Item definitions |
| `generation/noise/router/proto_noise_router.rs` | 468 | Proto noise router |
| `generation/noise/perlin.rs` | 441 | Perlin noise implementation |
| `generation/noise/mod.rs` | 438 | Noise module root |

**Key pumpkin-world subsystems by line count:**

| Subsystem | Lines (approx) |
|-----------|------:|
| generation/ (all) | ~22,000 |
| chunk/ (all) | ~3,600 |
| block/ (all) | ~3,800 |
| noise/ (all) | ~5,700 |
| structure/ (all) | ~4,200 |
| feature/ (all) | ~3,300 |
| surface/ | ~750 |
| biome/ | ~460 |

### pumpkin/src/world/ (15 files, 6,829 lines)

| File | Lines | Purpose |
|------|------:|---------|
| `mod.rs` | 3,752 | World tick loop, player management, chunk sending |
| `portal/nether.rs` | 794 | Nether portal logic |
| `natural_spawner.rs` | 606 | Mob natural spawning |
| `custom_bossbar.rs` | 403 | Custom boss bar management |
| `loot.rs` | 248 | Loot table handling |
| `weather.rs` | 181 | Weather state management |
| `border.rs` | 139 | World border |
| `scoreboard.rs` | 122 | Scoreboard system |
| `bossbar.rs` | 118 | Boss bar rendering |
| `chunker.rs` | 106 | Chunk delivery |
| `explosion.rs` | 109 | Explosion mechanics |
| `portal/end.rs` | 103 | End portal logic |
| `time.rs` | 73 | Day/night cycle |
| `portal/mod.rs` | 73 | Portal module root |
| `portal/poi.rs` | 2 | POI stub |

### pumpkin/src/block/ (138 files, 21,143 lines)

Top files by size:

| File | Lines | Purpose |
|------|------:|---------|
| `registry.rs` | 919 | Block registry and behavior dispatch |
| `blocks/redstone/redstone_wire.rs` | 861 | Redstone wire logic |
| `blocks/signs.rs` | 645 | Sign block behavior |
| `blocks/chests.rs` | 643 | Chest block behavior |
| `blocks/redstone/rails/activator_rail.rs` | 568 | Activator rail |
| `blocks/redstone/rails/powered_rail.rs` | 569 | Powered rail |
| `fluid/flowing_trait.rs` | 547 | Fluid flow mechanics |
| `blocks/piston/piston.rs` | 540 | Piston mechanics |
| `blocks/fire/fire.rs` | 421 | Fire spread logic |
| `blocks/redstone/turbo.rs` | 421 | Turbo redstone engine |
| `mod.rs` | 413 | Block module root |

---

## Gap Analysis

### Spec vs Implementation Coverage

| Domain | Spec Data | Pumpkin Status | Gap |
|--------|-----------|---------------|-----|
| **Block types** | 1,095 blocks / 245 types | 138 block .rs files in `pumpkin/src/block/` | Many block types share handlers; not 1:1 mapping |
| **Biomes** | 138 biome definitions | `biome/` module ~460 lines, `biome.rs` 204 lines | Biome data loaded from registry; generation logic minimal |
| **Configured features** | 205 feature configs | `configured_features.rs` 375 lines + stubs | Most features are 6-line stubs (unimplemented `todo!()`) |
| **Placed features** | 239 placement rules | `placed_features.rs` 572 lines | Deserialization exists; placement logic partial |
| **Structures** | 49 structures | Stronghold, desert_pyramid, swamp_hut, igloo, jungle_temple, buried_treasure, nether_fortress | ~7 of 49 structures implemented |
| **Density functions** | 42 definitions | `density_function/` ~1,900 lines | Core math/noise/spline present |
| **Template pools** | 245 jigsaw pools | Not implemented | No jigsaw assembler -- villages/bastions/etc. cannot generate |
| **Processor lists** | 41 processors | Not implemented | Required for jigsaw structure gen |
| **Noise settings** | 8 dimension configs | `noise_settings/` present via noise router | Overworld/nether/end covered |
| **Carvers** | 5 configured carvers | `carver/cave.rs` 166 lines | Cave carver exists; canyon carver stub |
| **Dimensions** | 4 dimension types | `dimension.rs` 28 lines | Basic enum only |
| **Mining speeds** | 17 material categories | Not in pumpkin-world (handled in pumpkin server) | Block breaking speed calculation needed |
| **Mob spawning** | Per-biome spawn tables | `natural_spawner.rs` 606 lines | Basic spawner exists |
| **Surface rules** | Complex condition tree | `surface/` ~750 lines | Partial implementation |

### Critical Gaps (Priority Order)

| Priority | Gap | Impact |
|----------|-----|--------|
| **P0** | Feature stubs (30+ features are `todo!()`) | Trees, ores, vegetation not generating |
| **P0** | Jigsaw structure assembly | No villages, bastions, ancient cities |
| **P1** | Canyon carver | Missing terrain feature |
| **P1** | Template pool / processor list deserialization | Prerequisite for jigsaw |
| **P1** | Mining speed calculation | Block breaking times wrong |
| **P2** | Remaining structures (42 of 49 missing) | Missing generated structures |
| **P2** | Surface rule completeness | Biome-specific surface blocks |
| **P3** | Flat world presets (12 presets) | Superflat customization |
| **P3** | World presets (10 presets) | World type selection |

### What Works

| Component | Status |
|-----------|--------|
| Chunk loading/saving (Anvil + Linear) | Functional (3,872 lines) |
| Noise generation (Perlin, aquifer, density) | Functional (~5,700 lines) |
| Proto-chunk generation pipeline | Functional (1,247 lines) |
| 7 structure types | Functional (~2,700 lines) |
| Ore feature generation | Functional (241 lines) |
| Block palette encoding | Functional (606 lines) |
| World tick loop | Functional (3,752 lines) |
| Natural mob spawning | Functional (606 lines) |
| Nether portal mechanics | Functional (794 lines) |
