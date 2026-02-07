# MC 1.21.4 | Items Agent Reference

Generated: 2026-02-07
Sources: `item_components.json`, `prismarine-1.21.4.zip`, `mcdata-1.21.4.zip`, Pumpkin codebase

---

## 1. Item Component Types

**Total items: 1,385 | Distinct component types: 41**

### 1.1 Universal Components (present on ALL 1,385 items)

| Component | Count | Notes |
|-----------|-------|-------|
| `minecraft:attribute_modifiers` | 1385 | Always present (may be empty `modifiers: []`) |
| `minecraft:enchantments` | 1385 | Always present (may be empty `levels: {}`) |
| `minecraft:item_model` | 1385 | Resource path for client model |
| `minecraft:item_name` | 1385 | JSON text component (translate key) |
| `minecraft:lore` | 1385 | Always present (usually empty `[]`) |
| `minecraft:max_stack_size` | 1385 | 1, 16, or 64 |
| `minecraft:rarity` | 1385 | common, uncommon, rare, epic |
| `minecraft:repair_cost` | 1385 | Anvil repair cost (default 0) |

### 1.2 Conditional Components (only on relevant items)

| Component | Count | Notes |
|-----------|-------|-------|
| `minecraft:damage` | 68 | Current damage value (damageable items) |
| `minecraft:max_damage` | 68 | Maximum durability |
| `minecraft:enchantable` | 61 | Enchantment value for enchanting table |
| `minecraft:repairable` | 59 | Tag reference for repair materials |
| `minecraft:equippable` | 56 | Armor/equipment slot + model + sound |
| `minecraft:consumable` | 43 | Eat/drink effects |
| `minecraft:food` | 40 | Nutrition + saturation values |
| `minecraft:tool` | 33 | Mining speed rules, damage_per_block |
| `minecraft:container` | 32 | Shulker boxes etc. |
| `minecraft:jukebox_playable` | 19 | Music disc data |
| `minecraft:banner_patterns` | 17 | Banner pattern layers |
| `minecraft:bundle_contents` | 17 | Bundle item storage |
| `minecraft:damage_resistant` | 14 | Fire-resistant netherite items |
| `minecraft:enchantment_glint_override` | 7 | Force enchant glint (e.g. enchanted golden apple) |
| `minecraft:use_remainder` | 7 | Item left after use (e.g. bowl from stew) |
| `minecraft:bucket_entity_data` | 6 | Fish bucket entity data |
| `minecraft:potion_contents` | 4 | Potion/arrow effect data |
| `minecraft:use_cooldown` | 3 | Cooldown after use (ender pearl, chorus fruit, goat horn) |
| `minecraft:bees` | 2 | Bee nest/beehive contents |
| `minecraft:block_state` | 1 | Light block state |
| `minecraft:charged_projectiles` | 1 | Crossbow loaded ammo |
| `minecraft:death_protection` | 1 | Totem of undying |
| `minecraft:debug_stick_state` | 1 | Debug stick state storage |
| `minecraft:fireworks` | 1 | Firework rocket data |
| `minecraft:glider` | 1 | Elytra glide capability |
| `minecraft:map_color` | 1 | Filled map color |
| `minecraft:map_decorations` | 1 | Map decoration markers |
| `minecraft:ominous_bottle_amplifier` | 1 | Ominous bottle bad omen level |
| `minecraft:pot_decorations` | 1 | Decorated pot sherd data |
| `minecraft:recipes` | 1 | Knowledge book recipes |
| `minecraft:stored_enchantments` | 1 | Enchanted book stored enchantments |
| `minecraft:suspicious_stew_effects` | 1 | Suspicious stew potion effects |
| `minecraft:writable_book_content` | 1 | Book and quill pages |

---

## 2. Enchantments

**Total: 42 enchantments across 18 categories**

| Name | Max Level | Category |
|------|-----------|----------|
| aqua_affinity | 1 | head_armor |
| bane_of_arthropods | 5 | weapon |
| binding_curse | 1 | equippable |
| blast_protection | 4 | armor |
| breach | 4 | mace |
| channeling | 1 | trident |
| density | 5 | mace |
| depth_strider | 3 | foot_armor |
| efficiency | 5 | mining |
| feather_falling | 4 | foot_armor |
| fire_aspect | 2 | fire_aspect |
| fire_protection | 4 | armor |
| flame | 1 | bow |
| fortune | 3 | mining_loot |
| frost_walker | 2 | foot_armor |
| impaling | 5 | trident |
| infinity | 1 | bow |
| knockback | 2 | sword |
| looting | 3 | sword |
| loyalty | 3 | trident |
| luck_of_the_sea | 3 | fishing |
| lure | 3 | fishing |
| mending | 1 | durability |
| multishot | 1 | crossbow |
| piercing | 4 | crossbow |
| power | 5 | bow |
| projectile_protection | 4 | armor |
| protection | 4 | armor |
| punch | 2 | bow |
| quick_charge | 3 | crossbow |
| respiration | 3 | head_armor |
| riptide | 3 | trident |
| sharpness | 5 | sharp_weapon |
| silk_touch | 1 | mining_loot |
| smite | 5 | weapon |
| soul_speed | 3 | foot_armor |
| sweeping_edge | 3 | sword |
| swift_sneak | 3 | leg_armor |
| thorns | 3 | armor |
| unbreaking | 3 | durability |
| vanishing_curse | 1 | vanishing |
| wind_burst | 3 | mace |

### Enchantments by Category

| Category | Count | Enchantments |
|----------|-------|-------------|
| armor | 5 | blast_protection, fire_protection, projectile_protection, protection, thorns |
| bow | 4 | flame, infinity, power, punch |
| crossbow | 3 | multishot, piercing, quick_charge |
| durability | 2 | mending, unbreaking |
| equippable | 1 | binding_curse |
| fire_aspect | 1 | fire_aspect |
| fishing | 2 | luck_of_the_sea, lure |
| foot_armor | 4 | depth_strider, feather_falling, frost_walker, soul_speed |
| head_armor | 2 | aqua_affinity, respiration |
| leg_armor | 1 | swift_sneak |
| mace | 3 | breach, density, wind_burst |
| mining | 1 | efficiency |
| mining_loot | 2 | fortune, silk_touch |
| sharp_weapon | 1 | sharpness |
| sword | 3 | knockback, looting, sweeping_edge |
| trident | 4 | channeling, impaling, loyalty, riptide |
| vanishing | 1 | vanishing_curse |
| weapon | 2 | bane_of_arthropods, smite |

---

## 3. Recipe Inventory

**Total recipe files: 1,370**

### 3.1 Recipes by Type

| Type | Count |
|------|-------|
| `minecraft:crafting_shaped` | 655 |
| `minecraft:crafting_shapeless` | 277 |
| `minecraft:stonecutting` | 254 |
| `minecraft:smelting` | 71 |
| `minecraft:crafting_transmute` | 32 |
| `minecraft:blasting` | 24 |
| `minecraft:smithing_trim` | 18 |
| `minecraft:smithing_transform` | 9 |
| `minecraft:campfire_cooking` | 9 |
| `minecraft:smoking` | 9 |
| `minecraft:crafting_special_*` | 12 (hardcoded special recipes) |

Special crafting recipes (hardcoded logic, no data-driven ingredients):
`armordye`, `bannerduplicate`, `bookcloning`, `decorated_pot`, `firework_rocket`, `firework_star`, `firework_star_fade`, `mapcloning`, `mapextending`, `repairitem`, `shielddecoration`, `tippedarrow`

### 3.2 Example: Shaped Recipe (lever)

```json
{
  "type": "minecraft:crafting_shaped",
  "category": "redstone",
  "key": {
    "#": "minecraft:cobblestone",
    "X": "minecraft:stick"
  },
  "pattern": [
    "X",
    "#"
  ],
  "result": {
    "count": 1,
    "id": "minecraft:lever"
  }
}
```

### 3.3 Example: Shapeless Recipe (blaze_powder)

```json
{
  "type": "minecraft:crafting_shapeless",
  "category": "misc",
  "ingredients": [
    "minecraft:blaze_rod"
  ],
  "result": {
    "count": 2,
    "id": "minecraft:blaze_powder"
  }
}
```

---

## 4. Loot Table Inventory

**Total loot table files: 1,237**

### 4.1 Loot Tables by Category

| Category | Count | Description |
|----------|-------|-------------|
| blocks | 1,015 | Block drop tables |
| entities | 101 | Mob/entity death drops |
| chests | 56 | Structure chest loot (dungeons, temples, etc.) |
| gameplay | 25 | Fishing, bartering, cat gifts, etc. |
| shearing | 22 | Sheep wool, mooshroom, bogged, snow golem |
| archaeology | 6 | Suspicious sand/gravel |
| spawners | 5 | Trial chamber spawner drops |
| equipment | 3 | Trial chamber mob equipment |
| dispensers | 3 | Trial chamber dispenser items |
| pots | 1 | Decorated pot loot |

### 4.2 Example: Gameplay Loot Table (cat_morning_gift)

```json
{
  "type": "minecraft:gift",
  "pools": [
    {
      "bonus_rolls": 0.0,
      "entries": [
        { "type": "minecraft:item", "name": "minecraft:rabbit_hide", "weight": 10 },
        { "type": "minecraft:item", "name": "minecraft:rabbit_foot", "weight": 10 },
        { "type": "minecraft:item", "name": "minecraft:chicken", "weight": 10 },
        { "type": "minecraft:item", "name": "minecraft:feather", "weight": 10 },
        { "type": "minecraft:item", "name": "minecraft:rotten_flesh", "weight": 10 },
        { "type": "minecraft:item", "name": "minecraft:string", "weight": 10 },
        { "type": "minecraft:item", "name": "minecraft:phantom_membrane", "weight": 2 }
      ],
      "rolls": 1.0
    }
  ],
  "random_sequence": "minecraft:gameplay/cat_morning_gift"
}
```

---

## 5. Current Pumpkin Implementation

### 5.1 pumpkin-inventory/src/ (5,471 lines, 24 files)

| File | Lines | Purpose |
|------|-------|---------|
| `screen_handler.rs` | 1,090 | Core inventory screen handling |
| `crafting/crafting_screen_handler.rs` | 687 | Crafting table UI + recipe matching |
| `player/player_inventory.rs` | 434 | Player inventory (36 slots + offhand) |
| `smithing.rs` | 426 | Smithing table handler |
| `stonecutter.rs` | 376 | Stonecutter handler |
| `slot.rs` | 358 | Slot abstraction (normal, result, armor, etc.) |
| `container_click.rs` | 354 | Click action processing |
| `player/player_screen_handler.rs` | 217 | Player screen handler |
| `generic_container_screen_handler.rs` | 198 | Chest/barrel/shulker handler |
| `crafting/crafting_inventory.rs` | 192 | Crafting grid inventory |
| `furnace_like/furnace_like_screen_handler.rs` | 173 | Furnace/blast furnace/smoker handler |
| `drag_handler.rs` | 171 | Drag-to-distribute logic |
| `sync_handler.rs` | 159 | Client sync protocol |
| `furnace_like/furnace_like_slot.rs` | 132 | Furnace slot validation |
| `player/ender_chest_inventory.rs` | 119 | Ender chest (per-player) |
| `window_property.rs` | 119 | Window property tracking |
| `double.rs` | 115 | Double chest handler |
| `entity_equipment.rs` | 59 | Equipment slot mapping |
| `lib.rs` | 47 | Module declarations + equipment slot builder |
| `error.rs` | 19 | InventoryError type |
| `crafting/recipes.rs` | 18 | Stub: RecipeMatcher, RecipeFinder traits |
| `crafting/mod.rs` | 3 | Module declarations |
| `furnace_like/mod.rs` | 2 | Module declarations |
| `player/mod.rs` | 3 | Module declarations |

### 5.2 pumpkin/src/item/ (2,285 lines, 26 files)

| File | Lines | Purpose |
|------|-------|---------|
| `items/bucket.rs` | 319 | Bucket fill/place logic |
| `items/axe.rs` | 221 | Axe stripping/scraping |
| `items/honeycomb.rs` | 172 | Waxing copper blocks |
| `items/hoe.rs` | 108 | Hoe tilling |
| `items/armor_stand.rs` | 102 | Armor stand placement |
| `items/shovel.rs` | 102 | Shovel pathing/campfire |
| `items/minecart.rs` | 92 | Minecart placement |
| `items/ender_eye.rs` | 90 | Eye of ender / end portal |
| `items/mod.rs` | 81 | Item registration hub |
| `mod.rs` | 75 | ItemBehaviour trait definition |
| `registry.rs` | 79 | ItemRegistry (HashMap lookup) |
| `items/spawn_egg.rs` | 74 | Spawn egg entity spawning |
| `items/firework_rocket.rs` | 75 | Firework rocket launch |
| `items/wind_charge.rs` | 70 | Wind charge projectile |
| `items/ignite/ignition.rs` | 68 | Shared ignition logic |
| `items/ignite/fire_charge.rs` | 62 | Fire charge use |
| `items/ignite/flint_and_steel.rs` | 60 | Flint and steel use |
| `items/egg.rs` | 60 | Egg projectile |
| `items/snowball.rs` | 57 | Snowball projectile |
| `items/dye.rs` | 56 | Dye on sheep/signs |
| `items/end_crystal.rs` | 54 | End crystal placement |
| `items/ink_sac.rs` | 48 | Ink sac on signs |
| `items/glowing_ink_sac.rs` | 48 | Glowing ink sac on signs |
| `items/name_tag.rs` | 41 | Name tag application |
| `items/trident.rs` | 25 | Trident throw (stub) |
| `items/swords.rs` | 22 | Sword mining check |
| `items/mace.rs` | 21 | Mace mining check |
| `items/ignite/mod.rs` | 3 | Module declarations |

### 5.3 Registered Item Behaviours (23 total)

```
ArmorStandItem, AxeItem, DyeItem, EggItem, EmptyBucketItem,
EndCrystalItem, EnderEyeItem, FilledBucketItem, FireChargeItem,
FireworkRocketItem, FlintAndSteelItem, GlowingInkSacItem, HoeItem,
HoneyCombItem, InkSacItem, MaceItem, MinecartItem, NameTagItem,
ShovelItem, SnowBallItem, SpawnEggItem, SwordItem, TridentItem,
WindChargeItem
```

### 5.4 Generated Data (pumpkin-data/)

| Generated File | Lines | Content |
|----------------|-------|---------|
| `item.rs` | 36,196 | All 1,385 items with components |
| `tag.rs` | 28,751 | Tag system (item tags, block tags, etc.) |
| `recipes.rs` | 19,029 | Crafting/smelting/smithing recipes |
| `enchantment.rs` | 677 | All 42 enchantments |
| `data_component.rs` | 443 | Component type enum |

---

## 6. Gap Analysis

### 6.1 Implementation Coverage Summary

| System | Status | Spec Count | Implemented | Coverage |
|--------|--------|------------|-------------|----------|
| Item definitions | Complete | 1,385 | 1,385 | 100% |
| Item behaviours | Partial | ~100 unique | 23 | ~23% |
| Enchantment data | Complete | 42 | 42 | 100% |
| Recipe data | Complete | 1,370 | 1,370 | 100% |
| Loot table data | Complete | 1,237 | 1,237 | 100% |
| Crafting (shaped/shapeless) | Implemented | 932 | 932 | 100% |
| Stonecutting | Implemented | 254 | 254 | 100% |
| Smithing | Implemented | 27 | 27 | 100% |
| Smelting/cooking | Partial | 113 | handler exists | ~50% |
| Loot table evaluation | Partial | 1,237 | engine exists, weight TODO | ~30% |
| Screen handlers | Partial | ~15 types | 7 types | ~47% |

### 6.2 Missing Screen Handlers

| Handler | Status | Impact |
|---------|--------|--------|
| Anvil | Not started | Blocks repair/rename/enchant combine |
| Enchanting table | Not started | Blocks player enchanting (42 enchantments) |
| Brewing stand | Not started | Blocks potion creation |
| Grindstone | Not started | Blocks enchantment removal |
| Loom | Not started | Blocks banner pattern creation |
| Cartography table | Not started | Blocks map modification |
| Beacon | Not started | Blocks beacon effect selection |
| Creative inventory | Not started | Blocks creative mode item access |

### 6.3 Missing Runtime Systems

| System | Spec Data Available | Status |
|--------|-------------------|--------|
| Consumable effect application | 43 items with `consumable` component | Not wired |
| Potion system | 4 items with `potion_contents` | Not started |
| Durability (complete) | 68 items with `max_damage` | Partial (some tools only) |
| Attribute modifier application | All 1,385 items | Partial |
| Equipment equip/unequip | 56 items with `equippable` | Partial |
| Bundle interaction | 17 items with `bundle_contents` | Not started |
| Block drop evaluation | 1,015 block loot tables | Partial |
| Entity drop evaluation | 101 entity loot tables | Partial |
| Chest loot generation | 56 chest loot tables | Not started |
| Fishing loot | gameplay loot tables | Not started |
| Villager trading | Not in spec data | Not started |

### 6.4 Priority Matrix

| Priority | Task | Data Source | Blocking |
|----------|------|-------------|----------|
| P0 | Consumable effect application | `consumable` component (43 items) | Golden apples, food effects |
| P0 | Loot table weight selection fix | All 1,237 loot tables | Correct drop rates |
| P1 | Anvil screen handler | `repair_cost` + `repairable` (59 items) | Item repair/rename |
| P1 | Enchanting table handler | `enchantable` (61 items) + 42 enchantments | Player enchanting |
| P1 | Creative inventory tabs | All 1,385 items | Creative mode usability |
| P2 | Brewing stand handler | `potion_contents` (4 items) | Potion creation |
| P2 | Grindstone handler | Enchantment data | Enchantment removal |
| P2 | Complete durability system | `max_damage` (68 items) | Tool/armor wear |
| P3 | Loom/Cartography handlers | `banner_patterns` (17), map components | Decorative features |
| P3 | Bundle interaction | `bundle_contents` (17 items) | Item organization |
| P3 | Beacon handler | Effect selection data | Beacon effects |
