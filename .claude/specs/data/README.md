# Minecraft 1.21.4 Vanilla Data

Source: [misode/mcmeta](https://github.com/misode/mcmeta) tag `1.21.4-data` and `1.21.4-summary`

## Quick Reference (summary/)

- `blocks.json` - 1095 block types with states and properties
- `registries.json` - ALL registry keys (entities, items, biomes, etc.)
- `item_components.json` - Item component definitions
- `block_definitions.json` - Block definitions
- `commands.json` - Command tree

## Full Data (mcdata-1.21.4.zip)

Extract to get individual JSON files:
```
unzip mcdata-1.21.4.zip
```

Contains:
- `recipe/` - 1370 recipes (crafting, smelting, smithing, stonecutting)
- `loot_table/` - 1237 loot tables (blocks/, entities/, chests/, gameplay/)
- `worldgen/` - Biomes, features, noise settings, structures
- `tags/` - Block tags, item tags, entity_type tags, biome tags
- `enchantment/` - 42 enchantments
- `damage_type/` - 49 damage types
- `dimension/` - Overworld, Nether, End configs

## Agent Usage

| Agent | Primary Data |
|-------|-------------|
| Items | recipe/, tags/item/, item_components.json |
| Entity | loot_table/entities/, tags/entity_type/ |
| WorldGen | worldgen/, tags/worldgen/ |
| Storage | block_definitions.json, loot_table/blocks/ |
| Redstone | tags/block/, blocks.json |
| Core | registries.json |
