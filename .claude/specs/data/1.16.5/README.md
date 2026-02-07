# MC 1.16.5 PrismarineJS Data

> **Nether Update** — Protocol 754. The version that introduced the modern dimension codec.
> Data sourced from PrismarineJS minecraft-data (inheritance chain: 1.16 → 1.16.1 → 1.16.2 → 1.16.4 → 1.16.5)

## File Inventory

| File | Source Version | Entries | Size |
|------|---------------|---------|------|
| entities.json | 1.16.2 | 108 | 20KB |
| blocks.json | 1.16.2 | 763 | 499KB |
| items.json | 1.16.2 | 975 | 120KB |
| recipes.json | 1.16.2 | 562 | 297KB |
| biomes.json | 1.16.2 | 79 | 22KB |
| enchantments.json | 1.16.4 | 38 | 15KB |
| effects.json | 1.16.1 | 33 | 3KB |
| foods.json | 1.16.1 | 40 | 8KB |
| materials.json | 1.16.2 | 8 categories | 1KB |
| blockLoot.json | 1.16.2 | 701 | 158KB |
| entityLoot.json | 1.16.2 | 72 | 18KB |
| attributes.json | 1.16 | 13 | 2KB |
| particles.json | 1.16 | 72 | 3KB |
| mapIcons.json | 1.16 | 27 | 3KB |
| sounds.json | 1.16 | 985 | 60KB |
| tints.json | 1.16.2 | 5 | 9KB |

## Version Delta: 1.16.5 → 1.21.4

| Category | 1.16.5 | 1.21.4 | Added | Removed |
|----------|--------|--------|-------|---------|
| **Entities** | 108 | 149 | +42 | -1 (boat → split into wood-specific boats) |
| **Blocks** | 763 | 1095 | +334 | -2 (grass→short_grass, grass_path→dirt_path) |
| **Items** | 975 | 1385 | +413 | ~3 renames |
| **Effects** | 33 | 39 | +8 | 0 |
| **Enchantments** | 38 | 42 | +4 | 0 |
| **Biomes** | 79 | 135 | +56 | 0 |
| **Recipes** | 562 | 1370 | +808 | — |

### Key New Content (1.17–1.21)
- **1.17 (Caves & Cliffs I):** Axolotl, goat, glow squid, amethyst, copper, deepslate, dripstone, tinted glass
- **1.18 (Caves & Cliffs II):** World height -64→320, new cave biomes (lush/dripstone/deep dark)
- **1.19 (Wild Update):** Warden, allay, frog, sculk, mangrove, mud, ancient city
- **1.20 (Trails & Tales):** Camel, sniffer, cherry grove, bamboo wood, armor trims, pottery
- **1.21 (Tricky Trials):** Breeze, bogged, wind charge, trial spawner, vault, mace, armadillo

### Notable Entity Changes
- `boat` removed → split into `acacia_boat`, `birch_boat`, `cherry_boat`, etc. (6 wood types × 2 variants)
- 42 new entities including: allay, armadillo, axolotl, breeze, camel, creaking, frog, goat, sniffer, warden

### Notable Block Changes
- 334 new blocks (44% increase)
- Major additions: hanging signs, candles, sculk family, copper family, bamboo wood, cherry wood
- 2 renames: `grass` → `short_grass`, `grass_path` → `dirt_path`

### Notable Effect Changes
- +8 new effects: Darkness, Infested, Oozing, RaidOmen, TrialOmen, Unluck, Weaving, WindCharged

## Multi-Version Strategy Notes

This data represents the **Tier 3** baseline from the sprint index. Key architectural differences:
- 1.16.5 uses the pre-component item system (NBT-based, not item components)
- 1.16.5 chunk format is pre-1.18 (0→255 height range, different section encoding)
- 1.16.5 introduced the dimension codec but uses a simpler format than 1.21.4
- Protocol 754 vs 774 — extensive packet changes across 5 major versions
