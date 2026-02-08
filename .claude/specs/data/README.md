# Minecraft 1.21.4 Reference Data

Three data sources for agent development:

## 1. Mojang Vanilla Data (misode/mcmeta)
**Location:** `mcdata-1.21.4.zip` + `1.21.4/summary/`
- 1370 recipes, 1237 loot tables, 42 enchantments, 49 damage types
- Worldgen: 65 biomes, structures, features, density functions
- Tags: block, item, entity_type, enchantment, worldgen
- Summary JSONs: blocks.json (1095), registries.json, item_components.json, block_definitions.json, commands.json

## 2. PrismarineJS Behavioral Data
**Location:** `1.21.4/prismarine/`
- `entities.json` — 149 entities with hitbox (width/height), category, metadata keys
- `foods.json` — hunger/saturation values for all food items
- `materials.json` — tool-to-block mining speed relationships
- `effects.json` — all status effects with ids
- `prismarine-1.21.4.zip` — FULL set: blocks, items, recipes, enchantments, biomes, particles, tints, instruments (1.5MB)

Source: github.com/PrismarineJS/minecraft-data

## 3. Bukkit/Spigot API Reference
**Location:** `bukkit-api/`
- `BUKKIT-API-REFERENCE.md` — Curated summary: 318 events across 11 packages, key interfaces (Player, World, Block, ItemStack), plugin lifecycle, scheduler, commands
- `bukkit-api-ref.zip` — Scraped Javadoc pages: Material (8333 lines), Player, World, Bukkit, LivingEntity, EntityType, AllClasses index, etc.

Live Javadoc: https://hub.spigotmc.org/javadocs/bukkit/

## Agent → Data Mapping

| Agent | Mojang (misode) | PrismarineJS | Bukkit API |
|-------|----------------|--------------|------------|
| Entity | loot_table/entities/, damage_type/ | **entities.json** (hitbox, metadata) | entity.*, event.entity.* |
| Items | recipe/, tags/item/, item_components | items.json, recipes.json, foods.json | Material, ItemStack, event.inventory.* |
| WorldGen | worldgen/, dimension/ | biomes.json, tints.json | World, generator.*, event.world.* |
| Storage | block_definitions, loot_table/blocks/ | blocks.json, materials.json | Block, Inventory, event.block.* |
| Redstone | tags/block/ | blocks.json | event.block.BlockRedstone/Piston* |
| Net | — | — | event.player.*, Server |
| Core | registries.json | — | plugin.*, scheduler.*, command.* |
